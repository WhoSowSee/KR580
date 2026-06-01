use iced::Task;

use super::constants::{
    MEMORY_ADDRESS_INPUT_ID, MEMORY_INLINE_INPUT_ID, MEMORY_SCROLL_VISIBLE_TICKS,
    MEMORY_VALUE_INPUT_ID, OPCODE_SEARCH_INPUT_ID, REGISTER_INLINE_INPUT_ID,
    REGISTER_NAME_INPUT_ID, REGISTER_VALUE_INPUT_ID,
};
use super::messages::Message;
use super::state::{DesktopApp, PendingAction};
use crate::platform;

impl DesktopApp {
    pub(crate) fn update(&mut self, message: Message) -> Task<Message> {
        if let Some(task) = self.route_discard_modal_message(&message) {
            return task;
        }
        if let Some(task) = self.route_settings_modal_message(&message) {
            return task;
        }
        if let Some(task) = self.dispatch_settings_message(message.clone()) {
            return task;
        }

        match message {
            Message::Tick => return self.handle_tick(),
            Message::CursorMoved(point) => {
                self.latest_cursor_position = point;
            }
            Message::MousePressed => {
                return iced::advanced::widget::operate(crate::runtime::find_focusable_at(
                    self.latest_cursor_position,
                ))
                .map(Message::FocusReconciled);
            }
            Message::FocusReconciled(hit) => return self.handle_focus_reconciled(hit),
            Message::ResolveFocusedTracker(None) => {
                self.focused_input = None;
            }
            Message::ResolveFocusedTracker(Some(_)) => {}
            Message::StepInstruction => return self.step_instruction_and_advance(),
            Message::RestartProgram => self.restart_program(),
            Message::StepTact => return self.step_tact_and_maybe_advance(),
            Message::ToggleRun => self.toggle_run(),
            Message::ResetCpu => {
                self.run_blocked_after_halt = false;
                self.dispatch_with_undo(k580_app::AppCommand::ResetCpu);
            }
            Message::ResetRam => {
                self.run_blocked_after_halt = false;
                self.dispatch_with_undo(k580_app::AppCommand::ResetRam);
            }
            Message::ClearHalt => {
                // Shortcut bypasses the menu's enabled gate; guard
                // against pushing an empty undo entry.
                if !self.snapshot.cpu.halted {
                    return Task::none();
                }
                self.run_blocked_after_halt = false;
                self.dispatch_with_undo(k580_app::AppCommand::ClearHalt);
            }
            Message::ToggleHalt => {
                if self.snapshot.cpu.halted {
                    self.run_blocked_after_halt = false;
                    self.dispatch_with_undo(k580_app::AppCommand::ClearHalt);
                } else {
                    self.dispatch_with_undo(k580_app::AppCommand::SetHalted(true));
                }
            }
            Message::OpenSnapshot => {
                if self.dirty {
                    self.open_discard_modal(PendingAction::OpenSnapshot);
                } else {
                    self.open_snapshot();
                }
            }
            Message::LoadSnapshotFromPath(path) => self.load_snapshot_from_path(path),
            Message::SaveSnapshot => self.save_snapshot(),
            Message::SaveSnapshotAs => self.save_snapshot_as(),
            Message::SaveLegacySnapshot => self.save_legacy_snapshot(),
            Message::OpenLegacySnapshot => {
                if self.dirty {
                    self.open_discard_modal(PendingAction::OpenLegacySnapshot);
                } else {
                    self.open_legacy_snapshot();
                }
            }
            Message::NewFile => {
                if self.dirty {
                    self.open_discard_modal(PendingAction::NewFile);
                } else {
                    self.run_new_file();
                }
            }
            Message::Export => self.export_file(),
            Message::Import => {
                if self.dirty {
                    self.open_discard_modal(PendingAction::Import);
                } else {
                    self.import_file();
                }
            }
            Message::RegisterNameChanged(value) => {
                self.change_register_name(value);
                self.active_register_target = None;
                self.inline_register_target = None;
                self.focused_input = Some(REGISTER_NAME_INPUT_ID);
            }
            Message::RegisterPrevious => self.step_register(-1),
            Message::RegisterNext => self.step_register(1),
            Message::RegisterValueChanged(value) => {
                // No focus op: queued ops race later clicks and steal focus.
                self.change_register_value(value);
                self.active_register_target = None;
                self.inline_register_target = None;
                self.focused_input = Some(REGISTER_VALUE_INPUT_ID);
            }
            Message::ApplyRegister => {
                if self.keyboard_modifiers.command() {
                    return self
                        .find_next_memory_address_in_direction(self.keyboard_modifiers.shift());
                }
                return self.apply_register_and_step(self.keyboard_modifiers.shift());
            }
            Message::RegisterSelected(target) => self.select_register_target(target),
            Message::RegisterEnter(target) => {
                self.enter_inline_register(target);
                self.focused_input = Some(REGISTER_INLINE_INPUT_ID);
                return Task::done(Message::RefocusInlineRegister);
            }
            Message::RefocusInlineRegister => {
                return iced::widget::operation::focus(REGISTER_INLINE_INPUT_ID);
            }
            Message::InlineRegisterValueChanged(target, value) => {
                self.change_inline_register_value(target, value);
                self.focused_input = Some(REGISTER_INLINE_INPUT_ID);
            }
            Message::ApplyInlineRegisterValue(target) => {
                return self.apply_inline_register_value(target, self.keyboard_modifiers.shift());
            }
            Message::RegisterHoverStarted(target) => {
                self.hovered_register_target = Some(target);
            }
            Message::RegisterHoverEnded(target) if self.hovered_register_target == Some(target) => {
                self.hovered_register_target = None;
            }
            Message::RegisterHoverEnded(_) => {}
            Message::MemorySelected(address) => self.select_memory(address),
            Message::MemoryEnter(address) => {
                // Defer focus — `MousePressed` reconcile would clear
                // focus on widgets whose bounds miss the click point.
                self.select_memory(address);
                self.focused_input = Some(MEMORY_INLINE_INPUT_ID);
                return Task::done(Message::RefocusInline);
            }
            Message::RefocusInline => {
                return iced::widget::operation::focus(MEMORY_INLINE_INPUT_ID);
            }
            Message::MemoryAddressPrevious => return self.step_memory_address(-1),
            Message::MemoryAddressNext => return self.step_memory_address(1),
            Message::MemoryAddressPageUp => return self.step_memory_address(-16),
            Message::MemoryAddressPageDown => return self.step_memory_address(16),
            Message::ArrowKey(direction) => return self.handle_arrow_key(direction),
            Message::HorizontalArrowKey(direction) => {
                return self.handle_horizontal_arrow_key(direction);
            }
            Message::RegisterCtrlArrowKey(direction) => {
                return self.navigate_inline_register_target(direction);
            }
            Message::MemoryScrolled(offset, viewport_height) => {
                self.memory_viewport_height = viewport_height;
                self.scroll_memory(offset);
                self.memory_scroll_visible_ticks = MEMORY_SCROLL_VISIBLE_TICKS;
            }
            Message::JumpMemoryAddress => {
                if self.keyboard_modifiers.command() {
                    return self
                        .find_next_memory_address_in_direction(self.keyboard_modifiers.shift());
                }
                if self.keyboard_modifiers.alt() {
                    return self.jump_memory_address();
                }
                return self.advance_memory_address(self.keyboard_modifiers.shift());
            }
            Message::MemoryAddressChanged(value) => {
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
                return step.chain(iced::widget::operation::focus(MEMORY_INLINE_INPUT_ID));
            }
            Message::OpcodeDropdownToggled(address) => self.toggle_opcode_dropdown(address),
            Message::OpcodeSearchChanged(value) => self.opcode_search_input = value,
            Message::OpcodeSelected(address, value) => self.select_opcode(address, value),
            Message::OpcodeScrolled => {
                self.opcode_scroll_visible_ticks = MEMORY_SCROLL_VISIBLE_TICKS;
            }
            Message::HideOpcodeDropdown => self.hide_opcode_dropdown(),
            Message::DismissErrorNotice => self.clear_error_notice(),
            Message::DismissHaltNotice => self.clear_halt_notice(),
            Message::DismissInfoNotice => self.clear_info_notice(),
            Message::EscPressed => return self.handle_esc(),
            Message::EnterPressed => {
                if let Some(target) = self.active_register_target {
                    return Task::done(Message::RegisterEnter(target));
                }
                let Some(address) = self.selected_memory_address() else {
                    return Task::none();
                };
                return Task::done(Message::MemoryEnter(address));
            }
            Message::OpenOpcodePicker => {
                let Some(address) = self.selected_memory_address() else {
                    return Task::none();
                };
                self.toggle_opcode_dropdown(address);
                if self.opcode_dropdown_address.is_none() {
                    return Task::none();
                }
                self.focused_input = Some(OPCODE_SEARCH_INPUT_ID);
                return iced::widget::operation::focus(OPCODE_SEARCH_INPUT_ID);
            }
            Message::ApplyMemory => {
                if self.keyboard_modifiers.command() {
                    return self
                        .find_next_memory_address_in_direction(self.keyboard_modifiers.shift());
                }
                if self.keyboard_modifiers.alt() {
                    return self.apply_memory_and_jump();
                }
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
                use iced::advanced::widget::operation::focusable::find_focused;
                return iced::advanced::widget::operate(find_focused())
                    .map(move |focused| Message::FocusResolved { focused, backward });
            }
            Message::FocusResolved { focused, backward } => {
                return self.cycle_focus(focused, backward);
            }
            Message::WindowOpened(id) => {
                // Cloak through the first frame so DWM doesn't paint
                // the default white client area; uncloak after iced presents.
                self.window_id = Some(id);
                return Task::batch([
                    iced::window::run(id, |window| platform::cloak_window(window, true)).discard(),
                    iced::window::run(id, |window| platform::set_rounded_corners(window)).discard(),
                    iced::window::set_mode(id, iced::window::Mode::Windowed),
                    iced::window::is_maximized(id).map(Message::WindowMaximizedChanged),
                ]);
            }
            Message::WindowResized(width) => {
                self.window_width = width;
            }
            Message::FrameRendered => {
                if self.startup_frames_seen < u8::MAX {
                    self.startup_frames_seen = self.startup_frames_seen.saturating_add(1);
                }
                if self.startup_frames_seen == 2 {
                    return iced::window::latest()
                        .and_then(|id| {
                            iced::window::run(id, |window| platform::cloak_window(window, false))
                        })
                        .discard();
                }
            }
            Message::MenuToggled(menu) => {
                self.open_menu = if self.open_menu == Some(menu) {
                    None
                } else {
                    Some(menu)
                };
            }
            Message::MenuClosed => {
                self.open_menu = None;
            }
            Message::MenuCategoriesToggled => {
                self.menu_categories_visible = !self.menu_categories_visible;
                if !self.menu_categories_visible {
                    self.open_menu = None;
                }
            }
            Message::MenuBatch(messages) => {
                let tasks = messages.into_iter().map(Task::done).collect::<Vec<_>>();
                return Task::batch(tasks);
            }
            Message::OpenAbout => {
                self.open_menu = None;
                self.about_dialog_open = true;
            }
            Message::CloseAbout => {
                self.about_dialog_open = false;
            }
            Message::ShowHelpComingSoon => {
                self.open_menu = None;
                self.set_status_custom(self.lang.t(crate::i18n::Key::HelpComingSoon).to_owned());
            }
            Message::OpenUrl(url) => {
                if let Err(error) = open_external_url(url) {
                    tracing::warn!("failed to open url {url}: {error}");
                }
            }
            Message::SpeedTierChanged(tier) => {
                self.apply_speed_tier(tier);
            }
            Message::WindowDragStart => {
                if self.close_titlebar_popup_before_drag() {
                    return Task::none();
                }
                let Some(id) = self.window_id else {
                    return Task::none();
                };
                return iced::window::drag(id);
            }
            Message::WindowMinimize => {
                let Some(id) = self.window_id else {
                    return Task::none();
                };
                return iced::window::minimize(id, true);
            }
            Message::WindowToggleMaximize => {
                let Some(id) = self.window_id else {
                    return Task::none();
                };
                // Optimistic swap so the caption glyph updates this
                // frame; the poll reconciles if the WM refuses.
                self.window_maximized = !self.window_maximized;
                return Task::batch([
                    iced::window::toggle_maximize(id),
                    iced::window::is_maximized(id).map(Message::WindowMaximizedChanged),
                ]);
            }
            Message::WindowClose => {
                let Some(id) = self.window_id else {
                    return Task::none();
                };
                return iced::window::close(id);
            }
            Message::WindowMaximizedChanged(maximized) => {
                self.window_maximized = maximized;
            }
            Message::Undo => return self.apply_undo(),
            Message::Redo => return self.apply_redo(),
            Message::ConfirmDiscard => return self.confirm_discard(),
            Message::CancelDiscard => self.cancel_discard(),
            Message::WindowCloseRequested => {
                if self.dirty {
                    self.open_discard_modal(PendingAction::CloseWindow);
                } else {
                    return Task::done(Message::WindowClose);
                }
            }
            Message::OpenMonitor => {
                self.open_menu = None;
                self.hide_opcode_dropdown();
                self.monitor_open = true;
            }
            Message::CloseMonitor => {
                self.monitor_open = false;
                self.monitor_hex_popup = false;
            }
            Message::ToggleMonitorSplit => {
                self.monitor_split = !self.monitor_split;
            }
            Message::ToggleMonitorHexPopup => {
                self.monitor_hex_popup = !self.monitor_hex_popup;
                if self.monitor_hex_popup {
                    self.monitor_hex_scroll_visible_ticks = MEMORY_SCROLL_VISIBLE_TICKS;
                }
            }
            Message::CycleMonitorHexFilter => {
                self.monitor_hex_filter = self.monitor_hex_filter.next();
                self.monitor_hex_scroll_visible_ticks = MEMORY_SCROLL_VISIBLE_TICKS;
            }
            Message::MonitorHexScrolled => {
                self.monitor_hex_scroll_visible_ticks = MEMORY_SCROLL_VISIBLE_TICKS;
            }
            Message::ClearMonitorBuffer => {
                self.dispatch(k580_app::AppCommand::ClearMonitorBuffer);
            }
            Message::SaveMonitorImage => {
                self.save_monitor_image();
            }
            _ => {}
        }
        Task::none()
    }
}

#[cfg(target_os = "windows")]
fn open_external_url(url: &str) -> std::io::Result<()> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    Command::new("cmd")
        .args(["/C", "start", "", url])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn open_external_url(url: &str) -> std::io::Result<()> {
    std::process::Command::new("open").arg(url).spawn()?;
    Ok(())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn open_external_url(url: &str) -> std::io::Result<()> {
    std::process::Command::new("xdg-open").arg(url).spawn()?;
    Ok(())
}
