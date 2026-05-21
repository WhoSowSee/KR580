//! Application shell: the iced state container, message routing, theme
//! selection, and the keyboard subscription.
//!
//! The two heaviest sub-pieces live in dedicated modules:
//!
//! - `messages` owns the `Message` enum (it grows often and would crowd
//!   the state container otherwise).
//! - `constants` owns the widget identifiers, the register order, and a
//!   couple of register-name helpers. They are re-exported from this
//!   module so the rest of the crate can keep importing them as
//!   `crate::app::FOO`.

mod constants;
mod messages;

pub(crate) use constants::{
    MEMORY_ADDRESS_COUNT, MEMORY_ADDRESS_INPUT_ID, MEMORY_INLINE_INPUT_ID, MEMORY_OVERSCAN_ROWS,
    MEMORY_RENDER_ROWS, MEMORY_ROW_HEIGHT, MEMORY_SCROLL_ID, MEMORY_SCROLL_VISIBLE_TICKS,
    MEMORY_VALUE_INPUT_ID, REGISTER_NAME_INPUT_ID, REGISTER_ORDER, REGISTER_VALUE_INPUT_ID,
    parse_register_name, register_name,
};
pub(crate) use messages::Message;

use iced::{Point, event, keyboard, mouse, time};
use iced::{Subscription, Task, Theme};
use k580_app::{AppSnapshot, EmulatorHandle, initial_snapshot, spawn_emulator};
use k580_core::RegisterName;
use std::time::Duration;

use crate::platform;

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
    /// Latest known cursor position, refreshed on every
    /// `mouse::Event::CursorMoved` from the global event listener. The
    /// `MousePressed` handler uses this to reconcile focus state against
    /// the click coordinates, because `mouse::Event::ButtonPressed` only
    /// carries the button identity. Defaults to the origin until iced
    /// reports the first cursor movement; in practice the user has to
    /// move the cursor before they can click anything, so the default
    /// is never observed.
    pub(crate) latest_cursor_position: Point,
    /// Visual "armed" state of the action panel's run/pause toggle.
    /// Decoupled from `AppCommand::Run` dispatch (see `Message::ToggleRun`)
    /// so empty pages never burn 100k T-states on a stray click.
    pub(crate) running: bool,
    /// Set on `TactAdvanced { instruction_boundary: true }`; cleared by
    /// the step-tact handler. PC mutates on the first tact in core, so
    /// before/after comparison would teleport — the handler waits for
    /// this flag instead.
    pub(crate) last_tact_was_boundary: bool,
    /// Tracks how many frames iced has rendered since startup. We keep the
    /// window cloaked (DWM-hidden on Windows) until the second frame so the
    /// OS never gets a chance to flash its default white client area.
    pub(crate) startup_frames_seen: u8,
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
                latest_cursor_position: Point::ORIGIN,
                running: false,
                last_tact_was_boundary: false,
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
            }
            Message::CursorMoved(point) => {
                // Cache the latest cursor position so the next
                // `MousePressed` knows where the click landed. The
                // mouse::Event::ButtonPressed variant carries only
                // the button identity, not the coordinates, so we
                // have to track them ourselves.
                self.latest_cursor_position = point;
            }
            Message::MousePressed => {
                // Authoritative focus reconciliation, in two passes.
                //
                // Pass 1 (`find_focusable_at`) is read-only: it walks
                // the widget tree and returns the id of the focusable
                // whose bounds contain the click point, or `None` if
                // the click missed every focusable.
                //
                // Pass 2 (`unfocus_except`) is the mutation: given a
                // confirmed hit id, it walks the tree again and
                // clears `state.is_focused` on every focusable that
                // is *not* the hit. This is what fixes the
                // column→stack capture race described in
                // `runtime::focus_ops` — text_inputs in sibling
                // panels never see the click, so without this pass
                // they would keep stale `Some(_)` flags from earlier
                // typing.
                //
                // Pass 2 only runs when pass 1 found a hit. A `None`
                // result is treated as "leave focus alone" instead
                // of "clear everything" because of a layout race:
                // iced processes the click in the freshly-clicked
                // input's `update` *before* draining the operation
                // queue, and the layout may shift by a pixel or two
                // in between, making the input's reported bounds
                // miss the click point. A single-pass operation
                // would then unfocus the input that just processed
                // the click, dropping the caret mid-edit. Splitting
                // the work and bailing out on `None` keeps repeat
                // clicks inside an already-focused input safe.
                return iced::advanced::widget::operate(crate::runtime::find_focusable_at(
                    self.latest_cursor_position,
                ))
                .map(Message::FocusReconciled);
            }
            Message::FocusReconciled(hit) => {
                const TRACKED: [&str; 5] = [
                    MEMORY_ADDRESS_INPUT_ID,
                    MEMORY_VALUE_INPUT_ID,
                    REGISTER_NAME_INPUT_ID,
                    REGISTER_VALUE_INPUT_ID,
                    MEMORY_INLINE_INPUT_ID,
                ];

                // Map the bare `Id` back to one of our static string
                // identifiers so the cosmetic shell border can index
                // into its own table. Untracked focusables (the
                // opcode-search input, for example, which is unkeyed)
                // resolve to `None` and clear the indicator entirely
                // — the user clicked into a region we don't decorate
                // with a focus ring.
                let resolved = hit.as_ref().and_then(|id| {
                    TRACKED
                        .into_iter()
                        .find(|known| *id == iced::widget::Id::new(known))
                });

                // Update the cosmetic tracker first so the focus ring
                // matches the new state on the same frame. Two cases
                // here:
                //
                // * `hit = Some(id)` — pass 1 found a focusable
                //   under the click. Update the ring and chain pass
                //   2 (`unfocus_except`) to clear stale focus on
                //   every *other* focusable. We deliberately do not
                //   touch the hit widget's state: iced's
                //   `text_input::update` has already set
                //   `is_focused = Some(_)` for it, and calling
                //   `state.focus()` ourselves would snap the caret
                //   to the end via `move_cursor_to_end`.
                //
                // * `hit = None` — pass 1 found nothing. Either the
                //   click landed in dead space (panel border, label,
                //   gap between widgets) or a layout race left the
                //   focused input's bounds momentarily not matching
                //   the click point. In neither case is wiping all
                //   focus the right move: dead-space clicks should
                //   leave focus alone (otherwise the user can never
                //   keep typing after clicking the surrounding
                //   chrome), and races are exactly the scenario the
                //   split is designed to absorb. So we simply do not
                //   issue a pass 2 here, leaving every focusable's
                //   state untouched.
                if let Some(id) = hit {
                    self.focused_input = resolved;
                    return iced::advanced::widget::operate(crate::runtime::unfocus_except(id))
                        .discard();
                }
            }
            Message::StepInstruction => return self.step_instruction_and_advance(),
            Message::RestartProgram => self.restart_program(),
            Message::StepTact => return self.step_tact_and_maybe_advance(),
            Message::Run => self.dispatch(k580_app::AppCommand::Run),
            Message::Stop => self.dispatch(k580_app::AppCommand::Stop),
            Message::ToggleRun => self.toggle_run(),
            Message::ResetCpu => self.dispatch(k580_app::AppCommand::ResetCpu),
            Message::ResetRam => self.dispatch(k580_app::AppCommand::ResetRam),
            Message::OpenSnapshot => self.open_snapshot(),
            Message::SaveSnapshot => self.save_snapshot(),
            Message::ExportTxt => self.export_txt(),
            Message::ExportXlsx => self.export_xlsx(),
            Message::ExportDocx => self.export_docx(),
            Message::RegisterSelected(register) => self.select_register(register),
            Message::RegisterNameChanged(value) => {
                // Mirror focus into our cosmetic tracker so the shell
                // border updates the same frame the user starts typing.
                // `MousePressed` -> `reconcile_focus_at` already does
                // this on click; this write covers the case where the
                // user reaches the field via Tab and starts typing
                // before the next click event arrives.
                self.change_register_name(value);
                self.focused_input = Some(REGISTER_NAME_INPUT_ID);
            }
            Message::RegisterPrevious => self.step_register(-1),
            Message::RegisterNext => self.step_register(1),
            Message::RegisterValueChanged(value) => {
                // See RegisterNameChanged — same rationale. We
                // deliberately do NOT return any focus operation here:
                // operations from `*Changed` handlers are queued and
                // can drain after the user has clicked into a different
                // panel, which would steal focus from the freshly
                // clicked input. The authoritative focus mutation
                // happens in `MousePressed` -> `reconcile_focus_at`.
                self.change_register_value(value);
                self.focused_input = Some(REGISTER_VALUE_INPUT_ID);
            }
            Message::ApplyRegister => {
                if self.keyboard_modifiers.command() {
                    return self
                        .find_next_memory_address_in_direction(self.keyboard_modifiers.shift());
                }
                return self.apply_register_and_step(self.keyboard_modifiers.shift());
            }
            Message::MemorySelected(address) => {
                // Single-click on the row: only move the highlight.
                // Focus stays where it was, so the user does not get
                // dropped into editing mode by an accidental click on
                // the address or command columns. To start editing,
                // they have to click the value column directly or
                // double-click the row.
                self.select_memory(address);
            }
            Message::MemoryEnter(address) => {
                // Either a double-click on the row or a single-click on
                // the value cell — both gestures unambiguously mean
                // "I want to type a new byte here".
                //
                // We can't focus the inline editor synchronously: the
                // very same `ButtonPressed` that triggered this message
                // also fires `Message::MousePressed` from the global
                // `event::listen_with` subscription, which dispatches
                // `reconcile_focus_at(cursor)` and clears focus from
                // every focusable whose bounds don't contain the click
                // point. For double-clicks on the address or command
                // columns the click point is *outside* the inline
                // editor's bounds, so a synchronously-issued
                // `operation::focus` would be promptly undone by the
                // reconcile pass.
                //
                // Bouncing through `RefocusInline` defers the focus to
                // the next update tick, well after the reconcile has
                // run. The cosmetic tracker is set immediately so the
                // shell border updates the same frame.
                self.select_memory(address);
                self.focused_input = Some(MEMORY_INLINE_INPUT_ID);
                return Task::done(Message::RefocusInline);
            }
            Message::RefocusInline => {
                // Deferred follow-up to ArrowUp/ArrowDown inside the
                // inline editor: by the time this message lands the
                // row at the new address has been laid out, so the
                // freshly-spawned `text_input` is in the tree and the
                // focus operation can target it. The cosmetic tracker
                // is already correct since we never changed it during
                // the step.
                return iced::widget::operation::focus(MEMORY_INLINE_INPUT_ID);
            }
            Message::MemoryAddressPrevious => return self.step_memory_address(-1),
            Message::MemoryAddressNext => return self.step_memory_address(1),
            Message::MemoryAddressPageUp => return self.step_memory_address(-16),
            Message::MemoryAddressPageDown => return self.step_memory_address(16),
            Message::ArrowKey(direction) => return self.handle_arrow_key(direction),
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
                // See RegisterNameChanged — same rationale. Mirror
                // focus for cosmetic styling, but do not return any
                // focus operation: queued ops would race with later
                // clicks and steal focus from the freshly clicked
                // input.
                self.change_memory_address(value);
                self.focused_input = Some(MEMORY_ADDRESS_INPUT_ID);
            }
            Message::MemoryValueChanged(value) => {
                self.change_memory_value(value);
                self.focused_input = Some(MEMORY_VALUE_INPUT_ID);
            }
            Message::InlineMemoryValueChanged(address, value) => {
                self.change_inline_memory_value(address, value);
                self.focused_input = Some(MEMORY_INLINE_INPUT_ID);
            }
            Message::ApplyInlineMemoryValue(address) => {
                let backward = self.keyboard_modifiers.shift();
                self.apply_inline_memory_value(address);
                let step = self.step_memory_address(if backward { -1 } else { 1 });
                // The inline editor widget is rebuilt against the new
                // address, which would normally drop focus. Re-focus it
                // here so the user can keep typing the next byte without
                // reaching for the mouse.
                return step.chain(iced::widget::operation::focus(MEMORY_INLINE_INPUT_ID));
            }
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
                    // ArrowUp/ArrowDown are routed by the message handler:
                    // the destination depends on which input owns focus and
                    // we don't want to read app state from inside the
                    // (Fn, not FnMut) listener closure.
                    keyboard::Key::Named(keyboard::key::Named::ArrowUp) => {
                        Some(Message::ArrowKey(1))
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowDown) => {
                        Some(Message::ArrowKey(-1))
                    }
                    keyboard::Key::Named(keyboard::key::Named::PageUp) => {
                        Some(Message::MemoryAddressPageUp)
                    }
                    keyboard::Key::Named(keyboard::key::Named::PageDown) => {
                        Some(Message::MemoryAddressPageDown)
                    }
                    _ => None,
                },
                // Track the cursor on every move regardless of whether
                // a widget captured the event — we need the latest
                // position cached so the next `ButtonPressed` knows
                // where the click landed. CursorMoved events fire
                // continuously during dragging, but the message
                // handler is a single field write so the cost is
                // negligible.
                (iced::Event::Mouse(mouse::Event::CursorMoved { position }), _) => {
                    Some(Message::CursorMoved(position))
                }
                // Fire reconciliation on every left mouse press,
                // regardless of capture status. Listening to captured
                // presses is the whole point: when text_input::update
                // captures a press inside panel A's input, the column
                // still propagates to panel B's stack, but B's stack
                // bails out and B's text_inputs never see the click.
                // The reconcile pass walks the tree from the outside
                // and clears every focusable not under the cursor,
                // fixing whatever stale state the broken propagation
                // left behind.
                (iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)), _) => {
                    Some(Message::MousePressed)
                }
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

    /// Routes ArrowUp/ArrowDown to whichever editor currently owns focus.
    /// `direction` is `+1` for ArrowUp and `-1` for ArrowDown, matching
    /// the convention "up increments, down decrements" used by numeric
    /// byte fields. With nothing tracked focused we fall back to memory
    /// list navigation, which is the legacy app-wide shortcut.
    fn handle_arrow_key(&mut self, direction: i32) -> Task<Message> {
        match self.focused_input {
            Some(REGISTER_NAME_INPUT_ID) => {
                // ArrowUp moves to the register listed *above* the current
                // one in `REGISTER_ORDER`, which means stepping by `-1`.
                self.step_register(-direction);
                Task::none()
            }
            Some(REGISTER_VALUE_INPUT_ID) => {
                self.step_register_value_input(direction);
                Task::none()
            }
            Some(MEMORY_VALUE_INPUT_ID) => {
                self.step_memory_value_input(direction);
                Task::none()
            }
            Some(MEMORY_INLINE_INPUT_ID) => {
                // Stepping the memory address moves the highlight onto a
                // different row, which means iced drops the inline
                // `text_input` from the row that was selected and
                // spawns a fresh one with the same id under the new
                // row. Chaining `operation::focus` directly here would
                // run before the rebuild, so the focus would land on
                // the widget that is about to disappear and the caret
                // would vanish.
                //
                // Bouncing through a `RefocusInline` message defers the
                // focus operation to the next update tick: by then the
                // new row is laid out, the new `text_input` is in the
                // tree, and `operation::focus(MEMORY_INLINE_INPUT_ID)`
                // hits it. The cosmetic `focused_input` tracker is
                // already pointing at this id, so we leave it alone.
                //
                // We also call `step_memory_address_browse` instead of
                // `step_memory_address`: the latter goes through
                // `select_memory -> sync_pc_to_cursor -> dispatch_sync`,
                // which blocks on a worker round-trip. The blocking
                // path was eating focus on the inline editor (the
                // `StateChanged` event came back synchronously in the
                // middle of the handler and the resulting view rebuild
                // landed before our `Task::done(RefocusInline)` made it
                // out the door). The browse-mode step keeps PC
                // untouched and updates only the spinner / inline
                // value so the row swap is purely cosmetic.
                let scroll = self.step_memory_address_browse(-direction);
                scroll.chain(Task::done(Message::RefocusInline))
            }
            // Memory address field and "no focus" both fall through to
            // memory navigation: stepping the address there *is* what the
            // user wants, and the unfocused case keeps the legacy global
            // shortcut.
            _ => self.step_memory_address(-direction),
        }
    }
}
