use iced::Task;

use super::constants::{
    MEMORY_ADDRESS_INPUT_ID, MEMORY_INLINE_INPUT_ID, MEMORY_SCROLL_VISIBLE_TICKS,
    MEMORY_VALUE_INPUT_ID, OPCODE_SEARCH_INPUT_ID, REGISTER_INLINE_INPUT_ID,
    REGISTER_NAME_INPUT_ID, REGISTER_VALUE_INPUT_ID, STACK_VIEW_START,
};
use super::messages::{Message, RegisterInlineTarget};
use super::state::{DesktopApp, PendingAction};

impl DesktopApp {
    pub(crate) fn update(&mut self, message: Message) -> Task<Message> {
        let message = match message {
            Message::RuntimeEvent {
                event,
                status,
                window,
            } => return self.handle_runtime_event(event, status, window),
            message => message,
        };
        if let Some(task) = self.dispatch_window_message(&message) {
            return task;
        }
        if let Some(task) = self.route_discard_modal_message(&message) {
            return task;
        }
        if let Some(task) = self.route_import_modal_message(&message) {
            return task;
        }
        if let Some(task) = self.route_export_modal_message(&message) {
            return task;
        }
        if let Some(task) = self.route_help_dialog_message(&message) {
            return task;
        }
        if let Some(task) = self.route_settings_modal_message(&message) {
            return task;
        }
        if let Some(task) = self.dispatch_settings_message(message.clone()) {
            return task;
        }
        if let Some(task) = self.dispatch_overlay_message(&message) {
            return task;
        }

        match message {
            Message::Tick => return self.handle_tick(),
            Message::CursorMoved(point) => {
                self.latest_cursor_position = point;
            }
            Message::MousePressed | Message::MousePressedIgnored => {
                self.mouse_press_generation = self.mouse_press_generation.wrapping_add(1);
                let generation = self.mouse_press_generation;
                self.handle_replacement_double_click(generation);
                return iced::advanced::widget::operate(crate::runtime::find_focusable_at(
                    self.latest_cursor_position,
                ))
                .map(move |hit| Message::FocusReconciled { generation, hit });
            }
            Message::FocusReconciled { generation, hit } => {
                return self.handle_focus_reconciled(generation, hit);
            }
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
                self.dispatch_with_undo(crate::backend::AppCommand::ResetCpu);
                self.pending_follow_pc = true;
            }
            Message::ResetRam => {
                self.run_blocked_after_halt = false;
                self.dispatch_with_undo(crate::backend::AppCommand::ResetRam);
            }
            Message::ClearHalt => {
                if !self.snapshot.cpu.halted {
                    return Task::none();
                }
                self.run_blocked_after_halt = false;
                self.dispatch_with_undo(crate::backend::AppCommand::ClearHalt);
                self.pending_follow_pc = false;
            }
            Message::ToggleHalt => {
                if self.snapshot.cpu.halted {
                    self.run_blocked_after_halt = false;
                    self.dispatch_with_undo(crate::backend::AppCommand::ClearHalt);
                } else {
                    self.dispatch_with_undo(crate::backend::AppCommand::SetHalted(true));
                }
                self.pending_follow_pc = false;
            }
            Message::OpenSnapshot => {
                if self.dirty {
                    self.open_discard_modal(PendingAction::OpenSnapshot);
                } else {
                    self.open_program();
                }
            }
            Message::LoadSnapshotFromPath(path) => self.load_program_from_path(path),
            Message::SaveSnapshot => self.save_program(),
            Message::SaveSnapshotAs => self.save_program_as(),
            Message::NewFile => {
                if self.dirty {
                    self.open_discard_modal(PendingAction::NewFile);
                } else {
                    self.run_new_file();
                }
            }
            Message::Export => self.open_export_modal(),
            Message::Import => {
                if self.dirty {
                    self.open_discard_modal(PendingAction::Import);
                } else {
                    self.open_import_modal();
                }
            }
            Message::RegisterNameChanged(value) if !self.running => {
                self.active_register_target = None;
                self.inline_register_target = None;
                self.change_register_name(value);
                self.focused_input = Some(REGISTER_NAME_INPUT_ID);
            }
            Message::RegisterPrevious if !self.running => {
                if self.register_name_input.is_empty() {
                    self.select_register_target(RegisterInlineTarget::for_register(
                        k580_core::RegisterName::A,
                    ));
                } else {
                    self.step_register(-1);
                }
            }
            Message::RegisterNext if !self.running => {
                if self.register_name_input.is_empty() {
                    self.select_register_target(RegisterInlineTarget::for_register(
                        k580_core::RegisterName::A,
                    ));
                } else {
                    self.step_register(1);
                }
            }
            Message::RegisterValueChanged(value) if !self.running => {
                self.change_register_value(value);
                self.active_register_target = None;
                self.inline_register_target = None;
                self.focused_input = Some(REGISTER_VALUE_INPUT_ID);
            }
            Message::ApplyRegister if !self.running => {
                if self.keyboard_modifiers.command() {
                    return self
                        .find_next_memory_address_in_direction(self.keyboard_modifiers.shift());
                }
                return self.apply_register_and_step(self.keyboard_modifiers.shift());
            }
            Message::RegisterSelected(target) if !self.running => {
                self.select_register_target(target)
            }
            Message::RegisterEnter(target) if !self.running => {
                self.enter_inline_register(target);
                self.focused_input = Some(REGISTER_INLINE_INPUT_ID);
                return iced::widget::operation::focus(REGISTER_INLINE_INPUT_ID);
            }
            Message::RegisterReplace(target) if !self.running => {
                self.enter_inline_register_replacing(target);
                self.focused_input = Some(REGISTER_INLINE_INPUT_ID);
                return iced::widget::operation::focus(REGISTER_INLINE_INPUT_ID);
            }
            Message::InlineRegisterValueChanged(target, value) if !self.running => {
                self.change_inline_register_value(target, value);
                self.focused_input = Some(REGISTER_INLINE_INPUT_ID);
            }
            Message::ApplyInlineRegisterValue(target) if !self.running => {
                return self.apply_inline_register_value(target, self.keyboard_modifiers.shift());
            }
            Message::RegisterHoverStarted(target) => {
                self.hovered_register_target = Some(target);
            }
            Message::RegisterHoverEnded(target) if self.hovered_register_target == Some(target) => {
                self.hovered_register_target = None;
            }
            Message::RegisterHoverEnded(_) => {}
            Message::MemorySelected(address) if !self.running => self.select_memory(address),
            Message::MemoryEnter(address) if !self.running => {
                self.select_memory(address);
                self.focused_input = Some(MEMORY_INLINE_INPUT_ID);
                return Task::done(Message::RefocusInline);
            }
            Message::MemoryReplace(address) if !self.running => {
                self.enter_inline_memory_replacing(address);
                self.focused_input = Some(MEMORY_INLINE_INPUT_ID);
                return Task::done(Message::RefocusInline);
            }
            Message::RefocusInline => {
                return iced::widget::operation::focus(MEMORY_INLINE_INPUT_ID);
            }
            Message::MemoryAddressPrevious if !self.running => return self.step_memory_address(-1),
            Message::MemoryAddressNext if !self.running => return self.step_memory_address(1),
            Message::MemoryAddressPageUp if !self.running => return self.step_memory_address(-16),
            Message::MemoryAddressPageDown if !self.running => return self.step_memory_address(16),
            Message::ArrowKey(direction) => return self.handle_arrow_key(direction),
            Message::HorizontalArrowKey(direction) => {
                return self.handle_horizontal_arrow_key(direction);
            }
            Message::RegisterArrowKey(direction) => {
                return self.navigate_inline_register_target(direction);
            }
            Message::MemoryScrolled(offset, viewport_height) => {
                self.memory_viewport_height = viewport_height;
                self.scroll_memory(offset);
                self.memory_scroll_visible_ticks = MEMORY_SCROLL_VISIBLE_TICKS;
            }
            Message::JumpMemoryAddress if !self.running => {
                if self.keyboard_modifiers.command() {
                    return self
                        .find_next_memory_address_in_direction(self.keyboard_modifiers.shift());
                }
                if self.keyboard_modifiers.alt() {
                    return self.jump_memory_address();
                }
                return self.advance_memory_address(self.keyboard_modifiers.shift());
            }
            Message::JumpMemoryTo(address) if !self.running => {
                if self.stack_view && address < STACK_VIEW_START {
                    self.disable_stack_view();
                }
                return self.jump_memory_to(address);
            }
            Message::MemoryAddressChanged(value) if !self.running => {
                self.change_memory_address(value);
                self.focused_input = Some(MEMORY_ADDRESS_INPUT_ID);
            }
            Message::MemoryValueChanged(value) if !self.running => {
                self.change_memory_value(value);
                self.focused_input = Some(MEMORY_VALUE_INPUT_ID);
            }
            Message::InlineMemoryValueChanged(address, value) if !self.running => {
                self.change_inline_memory_value(address, value);
                self.focused_input = Some(MEMORY_INLINE_INPUT_ID);
            }
            Message::ApplyInlineMemoryValue(address) if !self.running => {
                let replacing = self.replacement_input == Some(MEMORY_INLINE_INPUT_ID);
                let backward = self.keyboard_modifiers.shift();
                self.apply_inline_memory_value(address);
                let step = self.step_memory_address(if backward { -1 } else { 1 });
                if replacing {
                    self.begin_replacement(MEMORY_INLINE_INPUT_ID);
                }
                self.focused_input = Some(MEMORY_INLINE_INPUT_ID);
                return step.chain(iced::widget::operation::focus(MEMORY_INLINE_INPUT_ID));
            }
            Message::PasteMemoryBytesRequested if !self.running => {
                if self.selected_memory_paste_address().is_none() {
                    return Task::none();
                }
                return iced::clipboard::read().map(Message::MemoryBytesPasted);
            }
            Message::MemoryBytesPasted(Some(value)) if !self.running => {
                if let Some(address) = self.selected_memory_paste_address() {
                    self.paste_memory_bytes(address, value);
                }
            }
            Message::MemoryBytesPasted(None) => {}
            Message::OpcodeDropdownToggled(address) if !self.running => {
                self.toggle_opcode_dropdown(address)
            }
            Message::OpcodeSearchChanged(value) if !self.running => {
                self.change_opcode_search(value)
            }
            Message::OpcodeSelected(address, value) if !self.running => {
                self.select_opcode(address, value)
            }
            Message::OpcodeScrolled => {
                self.opcode_scroll_visible_ticks = MEMORY_SCROLL_VISIBLE_TICKS;
            }
            Message::HideOpcodeDropdown => self.hide_opcode_dropdown(),
            Message::DismissErrorNotice => self.clear_error_notice(),
            Message::DismissHaltNotice => self.clear_halt_notice(),
            Message::ToggleStackView => return self.toggle_stack_view(),
            Message::EscPressed => return self.handle_esc(),
            Message::EnterPressed => {
                if self.running {
                    return Task::none();
                }
                if self.opcode_dropdown_address.is_some() {
                    self.apply_highlighted_opcode();
                    return Task::none();
                }
                if let Some(target) = self.active_register_target {
                    return Task::done(Message::RegisterEnter(target));
                }
                let Some(address) = self.selected_memory_address() else {
                    return Task::none();
                };
                if self.keyboard_modifiers.alt() {
                    if self.keyboard_modifiers.shift() {
                        return self.return_to_memory_operand();
                    }
                    let memory = &self.snapshot.cpu.memory;
                    if let Some(port) = crate::view::operand_port_number(address, memory)
                        && let Some(open) = open_device_message(port)
                    {
                        let _ = self.update(open);
                        return Task::none();
                    }
                    if let Some(target) = crate::view::operand_jump_target(address, memory) {
                        return self.jump_from_memory_operand(address, target);
                    }
                }
                return Task::done(Message::MemoryEnter(address));
            }
            Message::OpenOpcodePicker if !self.running => {
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
            Message::ApplyMemory if !self.running => {
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
                if self.opcode_dropdown_address.is_some() {
                    self.step_opcode_highlight(if backward { -1 } else { 1 });
                    self.focused_input = Some(OPCODE_SEARCH_INPUT_ID);
                    return iced::widget::operation::focus(OPCODE_SEARCH_INPUT_ID);
                }
                use iced::advanced::widget::operation::focusable::find_focused;
                return iced::advanced::widget::operate(find_focused())
                    .map(move |focused| Message::FocusResolved { focused, backward });
            }
            Message::FocusResolved { focused, backward } => {
                return self.cycle_focus(focused, backward);
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
            Message::SpeedTierChanged(tier) => {
                self.apply_speed_tier(tier);
            }
            Message::Undo => return self.apply_undo(),
            Message::Redo => return self.apply_redo(),
            Message::ConfirmDiscard => return self.confirm_discard(),
            Message::CancelDiscard => self.cancel_discard(),
            _ => {}
        }
        Task::none()
    }
}

fn open_device_message(port: u8) -> Option<Message> {
    use crate::backend::IoBus;
    match port {
        IoBus::MONITOR_PORT => Some(Message::OpenMonitor),
        IoBus::FLOPPY_PORT => Some(Message::OpenFloppy),
        IoBus::HDD_PORT => Some(Message::OpenHdd),
        IoBus::NETWORK_PORT => Some(Message::OpenNetwork),
        IoBus::PRINTER_PORT => Some(Message::OpenPrinter),
        _ => None,
    }
}
