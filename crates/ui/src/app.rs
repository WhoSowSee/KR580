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
    pub(crate) memory_address_input: String,
    pub(crate) memory_value_input: String,
    pub(crate) memory_inline_value_input: String,
    pub(crate) opcode_dropdown_address: Option<u16>,
    pub(crate) opcode_search_input: String,
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
    MemoryScrolled(f32),
    JumpMemoryAddress,
    MemoryAddressChanged(String),
    MemoryValueChanged(String),
    InlineMemoryValueChanged(u16, String),
    ApplyInlineMemoryValue(u16),
    OpcodeDropdownToggled(u16),
    OpcodeSearchChanged(String),
    OpcodeSelected(u16, u8),
    HideOpcodeDropdown,
    ApplyMemory,
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
                memory_address_input: "0000".to_owned(),
                memory_value_input: "00".to_owned(),
                memory_inline_value_input: "00".to_owned(),
                opcode_dropdown_address: None,
                opcode_search_input: String::new(),
                startup_frames_seen: 0,
            },
            Task::none(),
        )
    }

    pub(crate) fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => self.pull_events(),
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
            Message::RegisterNameChanged(value) => self.change_register_name(value),
            Message::RegisterPrevious => self.step_register(-1),
            Message::RegisterNext => self.step_register(1),
            Message::RegisterValueChanged(value) => self.change_register_value(value),
            Message::ApplyRegister => self.apply_register(),
            Message::MemorySelected(address) => self.select_memory(address),
            Message::MemoryAddressPrevious => self.step_memory_address(-1),
            Message::MemoryAddressNext => self.step_memory_address(1),
            Message::MemoryScrolled(offset) => self.scroll_memory(offset),
            Message::JumpMemoryAddress => return self.jump_memory_address(),
            Message::MemoryAddressChanged(value) => self.change_memory_address(value),
            Message::MemoryValueChanged(value) => self.change_memory_value(value),
            Message::InlineMemoryValueChanged(address, value) => {
                self.change_inline_memory_value(address, value)
            }
            Message::ApplyInlineMemoryValue(address) => self.apply_inline_memory_value(address),
            Message::OpcodeDropdownToggled(address) => self.toggle_opcode_dropdown(address),
            Message::OpcodeSearchChanged(value) => self.opcode_search_input = value,
            Message::OpcodeSelected(address, value) => self.select_opcode(address, value),
            Message::HideOpcodeDropdown => self.hide_opcode_dropdown(),
            Message::ApplyMemory => return self.apply_memory(),
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
            event::listen_with(|event, _status, _window| match event {
                iced::Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::Escape),
                    ..
                }) => Some(Message::HideOpcodeDropdown),
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
