use iced::{Subscription, Task, Theme};
use iced::{event, keyboard, time};
use k580_app::{AppSnapshot, EmulatorHandle, initial_snapshot, spawn_emulator};
use k580_core::RegisterName;
use std::time::Duration;

use crate::platform;

pub(crate) const MEMORY_ADDRESS_COUNT: usize = 0x1_0000;
pub(crate) const MEMORY_OVERSCAN_ROWS: usize = 12;
pub(crate) const MEMORY_RENDER_ROWS: usize = 96;
pub(crate) const MEMORY_ROW_HEIGHT: f32 = 28.0;
pub(crate) const MEMORY_SCROLL_ID: &str = "memory-scroll";

/// Stable widget identifiers for every text input we want to drive with
/// keyboard navigation. They define isolated focus rings so that Tab/Shift+Tab
/// only cycles inside the panel that currently owns the focus instead of
/// walking through every focusable widget in the application.
pub(crate) const MEMORY_ADDRESS_INPUT_ID: &str = "memory-address-input";
pub(crate) const MEMORY_VALUE_INPUT_ID: &str = "memory-value-input";
pub(crate) const REGISTER_NAME_INPUT_ID: &str = "register-name-input";
pub(crate) const REGISTER_VALUE_INPUT_ID: &str = "register-value-input";
/// The inline value editor inside the memory list. Only one such input is
/// rendered at a time (for the currently selected address), so a single ID
/// keeps focus continuity when the user steps from one row to the next.
pub(crate) const MEMORY_INLINE_INPUT_ID: &str = "memory-inline-input";

/// Number of 100 ms ticks the memory scrollbar stays visible after the last
/// scroll event. 12 ticks ≈ 1.2 seconds.
pub(crate) const MEMORY_SCROLL_VISIBLE_TICKS: u8 = 12;

pub(crate) const REGISTER_ORDER: [RegisterName; 7] = [
    RegisterName::A,
    RegisterName::B,
    RegisterName::C,
    RegisterName::D,
    RegisterName::E,
    RegisterName::H,
    RegisterName::L,
];

pub(crate) fn register_name(register: RegisterName) -> &'static str {
    match register {
        RegisterName::A => "A",
        RegisterName::B => "B",
        RegisterName::C => "C",
        RegisterName::D => "D",
        RegisterName::E => "E",
        RegisterName::H => "H",
        RegisterName::L => "L",
    }
}

pub(crate) fn parse_register_name(input: &str) -> Option<RegisterName> {
    REGISTER_ORDER
        .iter()
        .copied()
        .find(|register| register_name(*register).eq_ignore_ascii_case(input.trim()))
}

pub(crate) struct DesktopApp {
    pub(crate) handle: EmulatorHandle,
    pub(crate) snapshot: AppSnapshot,
    pub(crate) status: String,
    pub(crate) selected_register: RegisterName,
    pub(crate) register_name_input: String,
    pub(crate) register_value_input: String,
    pub(crate) memory_scroll_first_row: u16,
    pub(crate) memory_scroll_offset: f32,
    pub(crate) memory_viewport_height: f32,
    pub(crate) memory_scroll_visible_ticks: u8,
    pub(crate) opcode_scroll_visible_ticks: u8,
    pub(crate) memory_address_input: String,
    pub(crate) memory_value_input: String,
    pub(crate) memory_inline_value_input: String,
    pub(crate) opcode_dropdown_address: Option<u16>,
    pub(crate) opcode_search_input: String,
    /// Cached substring pattern for the address-search workflow. Stored
    /// separately from `memory_address_input` because every successful
    /// match overwrites the input with the matched 4-digit address; without
    /// this cache the second Ctrl+Enter would search for the matched
    /// address itself instead of the original pattern.
    pub(crate) memory_search_pattern: Option<String>,
    /// Latest known state of the keyboard modifiers. Used to disambiguate
    /// `Enter` (apply memory write) from `Ctrl+Enter` (find next match) which
    /// the text input cannot tell apart on its own.
    pub(crate) keyboard_modifiers: keyboard::Modifiers,
    /// Identifier of the text input that the user has most recently
    /// interacted with, used purely to drive cosmetic focus styling on the
    /// spinner shells. Iced 0.14 has no `on_focus`/`on_blur` callbacks, so
    /// we sync this from any signal that implies focus (typing, Tab
    /// navigation, explicit focus tasks).
    pub(crate) focused_input: Option<&'static str>,
    /// Tracks how many frames iced has rendered since startup. We keep the
    /// window cloaked (DWM-hidden on Windows) until the second frame so the
    /// OS never gets a chance to flash its default white client area.
    pub(crate) startup_frames_seen: u8,
}

#[derive(Clone, Debug)]
pub(crate) enum Message {
    Tick,
    StepInstruction,
    StepTact,
    Run,
    Stop,
    ResetCpu,
    ResetRam,
    OpenSnapshot,
    SaveSnapshot,
    ExportTxt,
    ExportXlsx,
    ExportDocx,
    RegisterSelected(RegisterName),
    RegisterNameChanged(String),
    RegisterPrevious,
    RegisterNext,
    RegisterValueChanged(String),
    ApplyRegister,
    MemorySelected(u16),
    MemoryAddressPrevious,
    MemoryAddressNext,
    MemoryAddressPageUp,
    MemoryAddressPageDown,
    MemoryScrolled(f32, f32),
    JumpMemoryAddress,
    MemoryAddressChanged(String),
    MemoryValueChanged(String),
    InlineMemoryValueChanged(u16, String),
    ApplyInlineMemoryValue(u16),
    OpcodeDropdownToggled(u16),
    OpcodeSearchChanged(String),
    OpcodeSelected(u16, u8),
    OpcodeScrolled,
    HideOpcodeDropdown,
    ApplyMemory,
    /// Latest keyboard modifier state, broadcast by iced whenever any of the
    /// modifier keys change. Cached so message handlers can disambiguate
    /// modified shortcuts (Ctrl+Enter, Alt+Enter) before the text input's
    /// own `on_submit` fires.
    ModifiersChanged(keyboard::Modifiers),
    /// Move keyboard focus inside the focus group of the currently focused
    /// input. `backward` swaps direction (Shift+Tab). Groups are isolated:
    /// the memory address/value pair, the register name/value pair, and the
    /// inline memory list cycle independently.
    FocusCycle {
        backward: bool,
    },
    /// Internal continuation of `FocusCycle`: carries the id of the widget
    /// that owned focus when Tab was pressed. We compute the destination in
    /// the `update` handler because only there can we tweak app state (e.g.
    /// shift the inline-edited address) before issuing the actual focus
    /// task.
    FocusResolved {
        focused: iced::widget::Id,
        backward: bool,
    },
    /// Result of the periodic `find_focused` poll. Carries the ids of any
    /// focused widgets iced reports — typically zero or one — so the UI can
    /// keep `DesktopApp::focused_input` in sync regardless of how the user
    /// reached the input (typing, Tab, mouse click).
    FocusPolled(Vec<iced::widget::Id>),
    /// Iced reports that a window has been opened. We respond by cloaking it
    /// via DWM on Windows so the launch flash never reaches the screen.
    WindowOpened(iced::window::Id),
    /// Iced has rendered a frame. After the second frame we know the wgpu
    /// surface is presenting our content, so we can safely uncloak.
    FrameRendered,
}

impl DesktopApp {
    pub(crate) fn new() -> (Self, Task<Message>) {
        let handle = spawn_emulator();
        (
            Self {
                handle,
                snapshot: initial_snapshot(),
                status: "Ready".to_owned(),
                selected_register: RegisterName::A,
                register_name_input: "A".to_owned(),
                register_value_input: "00".to_owned(),
                memory_scroll_first_row: 0,
                memory_scroll_offset: 0.0,
                memory_viewport_height: 0.0,
                memory_scroll_visible_ticks: 0,
                opcode_scroll_visible_ticks: 0,
                memory_address_input: "0000".to_owned(),
                memory_value_input: "00".to_owned(),
                memory_inline_value_input: "00".to_owned(),
                opcode_dropdown_address: None,
                opcode_search_input: String::new(),
                memory_search_pattern: None,
                keyboard_modifiers: keyboard::Modifiers::default(),
                focused_input: None,
                startup_frames_seen: 0,
            },
            Task::none(),
        )
    }

    pub(crate) fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                self.pull_events();
                self.memory_scroll_visible_ticks =
                    self.memory_scroll_visible_ticks.saturating_sub(1);
                self.opcode_scroll_visible_ticks =
                    self.opcode_scroll_visible_ticks.saturating_sub(1);
                // Poll iced for the currently focused widget. `find_focused`
                // emits at most one id, but `collect()` always finishes with
                // a `Vec` (possibly empty), giving us a single deterministic
                // `FocusPolled` message every tick.
                use iced::advanced::widget::operation::focusable::find_focused;
                return iced::advanced::widget::operate(find_focused())
                    .collect()
                    .map(Message::FocusPolled);
            }
            Message::StepInstruction => self.dispatch(k580_app::AppCommand::StepInstruction),
            Message::StepTact => self.dispatch(k580_app::AppCommand::StepTact),
            Message::Run => self.dispatch(k580_app::AppCommand::Run),
            Message::Stop => self.dispatch(k580_app::AppCommand::Stop),
            Message::ResetCpu => self.dispatch(k580_app::AppCommand::ResetCpu),
            Message::ResetRam => self.dispatch(k580_app::AppCommand::ResetRam),
            Message::OpenSnapshot => self.open_snapshot(),
            Message::SaveSnapshot => self.save_snapshot(),
            Message::ExportTxt => self.export_txt(),
            Message::ExportXlsx => self.export_xlsx(),
            Message::ExportDocx => self.export_docx(),
            Message::RegisterSelected(register) => self.select_register(register),
            Message::RegisterNameChanged(value) => {
                self.focused_input = Some(REGISTER_NAME_INPUT_ID);
                self.change_register_name(value);
            }
            Message::RegisterPrevious => self.step_register(-1),
            Message::RegisterNext => self.step_register(1),
            Message::RegisterValueChanged(value) => {
                self.focused_input = Some(REGISTER_VALUE_INPUT_ID);
                self.change_register_value(value);
            }
            Message::ApplyRegister => {
                if self.keyboard_modifiers.command() {
                    return self
                        .find_next_memory_address_in_direction(self.keyboard_modifiers.shift());
                }
                return self.apply_register_and_step(self.keyboard_modifiers.shift());
            }
            Message::MemorySelected(address) => {
                self.focused_input = Some(MEMORY_INLINE_INPUT_ID);
                self.select_memory(address);
            }
            Message::MemoryAddressPrevious => return self.step_memory_address(-1),
            Message::MemoryAddressNext => return self.step_memory_address(1),
            Message::MemoryAddressPageUp => return self.step_memory_address(-16),
            Message::MemoryAddressPageDown => return self.step_memory_address(16),
            Message::MemoryScrolled(offset, viewport_height) => {
                self.memory_viewport_height = viewport_height;
                self.scroll_memory(offset);
                self.memory_scroll_visible_ticks = MEMORY_SCROLL_VISIBLE_TICKS;
            }
            Message::JumpMemoryAddress => {
                if self.keyboard_modifiers.command() {
                    // Ctrl+Enter forward search, Ctrl+Shift+Enter backward.
                    return self
                        .find_next_memory_address_in_direction(self.keyboard_modifiers.shift());
                }
                if self.keyboard_modifiers.alt() {
                    // Alt+Enter from the address field commits the typed
                    // address and jumps the memory list to it (the visible
                    // scroll target).
                    return self.jump_memory_address();
                }
                // Plain Enter / Shift+Enter: stay in the editor, advance or
                // step back the address in the input itself, without
                // scrolling the memory list.
                return self.advance_memory_address(self.keyboard_modifiers.shift());
            }
            Message::MemoryAddressChanged(value) => {
                self.focused_input = Some(MEMORY_ADDRESS_INPUT_ID);
                self.change_memory_address(value);
            }
            Message::MemoryValueChanged(value) => {
                self.focused_input = Some(MEMORY_VALUE_INPUT_ID);
                self.change_memory_value(value);
            }
            Message::InlineMemoryValueChanged(address, value) => {
                self.focused_input = Some(MEMORY_INLINE_INPUT_ID);
                self.change_inline_memory_value(address, value)
            }
            Message::ApplyInlineMemoryValue(address) => self.apply_inline_memory_value(address),
            Message::OpcodeDropdownToggled(address) => self.toggle_opcode_dropdown(address),
            Message::OpcodeSearchChanged(value) => self.opcode_search_input = value,
            Message::OpcodeSelected(address, value) => self.select_opcode(address, value),
            Message::OpcodeScrolled => {
                self.opcode_scroll_visible_ticks = MEMORY_SCROLL_VISIBLE_TICKS;
            }
            Message::HideOpcodeDropdown => self.hide_opcode_dropdown(),
            Message::ApplyMemory => {
                if self.keyboard_modifiers.command() {
                    // Ctrl+Enter forward search, Ctrl+Shift+Enter backward.
                    return self
                        .find_next_memory_address_in_direction(self.keyboard_modifiers.shift());
                }
                if self.keyboard_modifiers.alt() {
                    // Alt+Enter from the value field writes the byte and
                    // jumps the memory list to the same address.
                    return self.apply_memory_and_jump();
                }
                // Plain Enter / Shift+Enter: behaviour depends on which
                // memory-editor field the user is working in. From the
                // address field we just step the address; from the value
                // field we also commit the byte. Either way focus stays
                // where it was.
                let from_address = self.focused_input == Some(MEMORY_ADDRESS_INPUT_ID);
                let backward = self.keyboard_modifiers.shift();
                if from_address {
                    return self.advance_memory_address(backward);
                }
                return self.apply_memory_and_step(backward);
            }
            Message::ModifiersChanged(modifiers) => {
                self.keyboard_modifiers = modifiers;
            }
            Message::FocusCycle { backward } => {
                // Ask iced for the id of the currently focused widget. If
                // nothing is focused, this resolves to no value and the
                // continuation never fires—exactly what we want, because
                // focusing "the next widget" is meaningless without a
                // starting point.
                use iced::advanced::widget::operation::focusable::find_focused;
                return iced::advanced::widget::operate(find_focused())
                    .map(move |focused| Message::FocusResolved { focused, backward });
            }
            Message::FocusResolved { focused, backward } => {
                return self.cycle_focus(focused, backward);
            }
            Message::FocusPolled(ids) => {
                use crate::app::{
                    MEMORY_ADDRESS_INPUT_ID, MEMORY_INLINE_INPUT_ID, MEMORY_VALUE_INPUT_ID,
                    REGISTER_NAME_INPUT_ID, REGISTER_VALUE_INPUT_ID,
                };
                const TRACKED: [&str; 5] = [
                    MEMORY_ADDRESS_INPUT_ID,
                    MEMORY_VALUE_INPUT_ID,
                    REGISTER_NAME_INPUT_ID,
                    REGISTER_VALUE_INPUT_ID,
                    MEMORY_INLINE_INPUT_ID,
                ];
                let resolved = ids.into_iter().find_map(|id| {
                    TRACKED
                        .into_iter()
                        .find(|known| id == iced::widget::Id::new(known))
                });
                // Only refresh the cache when the poll points at one of
                // *our* tracked inputs. If iced reports nothing or focus
                // landed on something else (a spinner button, the apply
                // button, etc.), keep the last known input — otherwise
                // the brief blur from a button click would race the next
                // Enter handler and forget which field the user was in.
                if let Some(id) = resolved {
                    self.focused_input = Some(id);
                }
            }
            Message::WindowOpened(id) => {
                // Cloak immediately, then unhide the window. Because the
                // window is cloaked, DWM never composites the white client
                // area; the user only sees the window once we uncloak it
                // after iced has presented its first real frame.
                return Task::batch([
                    iced::window::run(id, |window| platform::cloak_window(window, true)).discard(),
                    iced::window::set_mode(id, iced::window::Mode::Windowed),
                ]);
            }
            Message::FrameRendered => {
                if self.startup_frames_seen < u8::MAX {
                    self.startup_frames_seen = self.startup_frames_seen.saturating_add(1);
                }
                // Wait for the second frame so we are certain the wgpu
                // swapchain has produced and presented our content before
                // exposing the window.
                if self.startup_frames_seen == 2 {
                    return iced::window::latest()
                        .and_then(|id| {
                            iced::window::run(id, |window| platform::cloak_window(window, false))
                        })
                        .discard();
                }
            }
        }
        Task::none()
    }

    pub(crate) fn theme(&self) -> Theme {
        Theme::TokyoNight
    }

    pub(crate) fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = vec![
            time::every(Duration::from_millis(100)).map(|_| Message::Tick),
            iced::window::open_events().map(Message::WindowOpened),
            event::listen_with(|event, status, _window| match (event, status) {
                (iced::Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)), _) => {
                    Some(Message::ModifiersChanged(modifiers))
                }
                (
                    iced::Event::Keyboard(keyboard::Event::KeyPressed {
                        key: keyboard::Key::Named(keyboard::key::Named::Escape),
                        ..
                    }),
                    _,
                ) => Some(Message::HideOpcodeDropdown),
                (
                    iced::Event::Keyboard(keyboard::Event::KeyPressed {
                        key: keyboard::Key::Named(keyboard::key::Named::Tab),
                        modifiers,
                        ..
                    }),
                    iced::event::Status::Ignored,
                ) => Some(Message::FocusCycle {
                    backward: modifiers.shift(),
                }),
                (
                    iced::Event::Keyboard(keyboard::Event::KeyPressed { key, .. }),
                    iced::event::Status::Ignored,
                ) => match key {
                    keyboard::Key::Named(keyboard::key::Named::ArrowUp) => {
                        Some(Message::MemoryAddressPrevious)
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowDown) => {
                        Some(Message::MemoryAddressNext)
                    }
                    keyboard::Key::Named(keyboard::key::Named::PageUp) => {
                        Some(Message::MemoryAddressPageUp)
                    }
                    keyboard::Key::Named(keyboard::key::Named::PageDown) => {
                        Some(Message::MemoryAddressPageDown)
                    }
                    _ => None,
                },
                _ => None,
            }),
        ];

        // Only listen to frame events while we are still cloaked. Once the
        // window is uncloaked there is nothing more to do, and iced docs warn
        // that the rate of `frames()` matches the display refresh rate.
        if self.startup_frames_seen < 2 {
            subscriptions.push(iced::window::frames().map(|_| Message::FrameRendered));
        }

        Subscription::batch(subscriptions)
    }
}
