use iced::Task;

use super::super::constants::{
    MEMORY_ADDRESS_INPUT_ID, MEMORY_INLINE_INPUT_ID, MEMORY_SCROLL_VISIBLE_TICKS,
    MEMORY_VALUE_INPUT_ID, OPCODE_SEARCH_INPUT_ID, REGISTER_INLINE_INPUT_ID,
    REGISTER_NAME_INPUT_ID, REGISTER_VALUE_INPUT_ID, STACK_VIEW_START,
};
use super::super::messages::{Message, RegisterInlineTarget};
use super::super::update_routes::open_device_message;
use super::super::{DesktopApp, PendingAction};
use crate::runtime::parse::scroll_memory_to;

impl DesktopApp {
    pub(super) fn update_inner(&mut self, message: Message) -> Task<Message> {
        let message = match message {
            Message::DeviceButtonPressed(kind, action) => {
                return self.press_device_button(kind, *action);
            }
            Message::RuntimeEvent {
                event,
                status,
                window,
            } => return self.handle_runtime_event(event, status, window),
            message => message,
        };
        if let Some(task) = self.route_blocking_ui_message(&message) {
            return task;
        }
        if let Some(task) = self.dispatch_settings_message(message.clone()) {
            return task;
        }
        if let Some(task) = self.dispatch_overlay_message(&message) {
            return task;
        }
        if let Some(task) = self.route_file_dialog_message(&message) {
            return task;
        }

        if let Some(task) = self.dispatch_execution_message(&message) {
            return task;
        }
        match message {
            Message::Tick => return self.handle_tick(),
            Message::FloppyImageContentsLoaded(path, result) => {
                self.apply_floppy_image_contents(path, result);
            }
            Message::HddImageContentsLoaded(path, result) => {
                self.apply_hdd_image_contents(path, result);
            }
            Message::CursorMoved(point) => self.interaction.latest_cursor_position = point,
            Message::FileDragCursorPosition(position) => self.update_file_drag_cursor(position),
            Message::MousePressed | Message::MousePressedIgnored => {
                self.interaction.mouse_press_generation =
                    self.interaction.mouse_press_generation.wrapping_add(1);
                let generation = self.interaction.mouse_press_generation;
                self.handle_replacement_double_click(generation);
                return iced::advanced::widget::operate(crate::runtime::find_focusable_at(
                    self.interaction.latest_cursor_position,
                ))
                .map(move |hit| Message::FocusReconciled { generation, hit });
            }
            Message::FocusReconciled { generation, hit } => {
                return self.handle_focus_reconciled(generation, hit);
            }
            Message::ResolveFocusedTracker(None) => {
                self.interaction.focused_input = None;
            }
            Message::ResolveFocusedTracker(Some(_)) => {}
            Message::OpenSnapshot => {
                if self.document.dirty {
                    self.open_discard_modal(PendingAction::OpenSnapshot);
                } else {
                    return self.open_program();
                }
            }
            Message::NewFile => {
                if self.document.dirty {
                    self.open_discard_modal(PendingAction::NewFile);
                } else {
                    self.run_new_file();
                }
            }
            Message::Export => self.open_export_modal(),
            Message::Import => {
                if self.document.dirty {
                    self.open_discard_modal(PendingAction::Import);
                } else {
                    self.open_import_modal();
                }
            }
            Message::RegisterNameChanged(value) if !self.execution.running => {
                self.register.active_register_target = None;
                self.register.inline_register_target = None;
                self.change_register_name(value);
                self.interaction.focused_input = Some(REGISTER_NAME_INPUT_ID);
            }
            Message::RegisterPrevious if !self.execution.running => {
                if self.register.register_name_input.is_empty() {
                    self.select_register_target(RegisterInlineTarget::for_register(
                        k580_core::RegisterName::A,
                    ));
                } else {
                    self.step_register(-1);
                }
            }
            Message::RegisterNext if !self.execution.running => {
                if self.register.register_name_input.is_empty() {
                    self.select_register_target(RegisterInlineTarget::for_register(
                        k580_core::RegisterName::A,
                    ));
                } else {
                    self.step_register(1);
                }
            }
            Message::RegisterValueChanged(value) if !self.execution.running => {
                self.change_register_value(value);
                self.register.active_register_target = None;
                self.register.inline_register_target = None;
                self.interaction.focused_input = Some(REGISTER_VALUE_INPUT_ID);
            }
            Message::ApplyRegister if !self.execution.running => {
                return self.apply_register_and_step(self.interaction.keyboard_modifiers.shift());
            }
            Message::RegisterSelected(target) if !self.execution.running => {
                self.select_register_target(target)
            }
            Message::RegisterEnter(target) if !self.execution.running => {
                self.enter_inline_register(target);
                self.interaction.focused_input = Some(REGISTER_INLINE_INPUT_ID);
                return iced::widget::operation::focus(REGISTER_INLINE_INPUT_ID);
            }
            Message::RegisterReplace(target) if !self.execution.running => {
                self.enter_inline_register_replacing(target);
                self.interaction.focused_input = Some(REGISTER_INLINE_INPUT_ID);
                return iced::widget::operation::focus(REGISTER_INLINE_INPUT_ID);
            }
            Message::InlineRegisterValueChanged(target, value) if !self.execution.running => {
                self.change_inline_register_value(target, value);
                self.interaction.focused_input = Some(REGISTER_INLINE_INPUT_ID);
            }
            Message::ApplyInlineRegisterValue(target) if !self.execution.running => {
                return self.apply_inline_register_value(
                    target,
                    self.interaction.keyboard_modifiers.shift(),
                );
            }
            Message::RegisterHoverStarted(target) => {
                self.register.hovered_register_target = Some(target);
            }
            Message::RegisterHoverEnded(target)
                if self.register.hovered_register_target == Some(target) =>
            {
                self.register.hovered_register_target = None;
            }
            Message::RegisterHoverEnded(_) => {}
            Message::MemorySelected(address) if !self.execution.running => {
                self.select_memory(address)
            }
            Message::MemoryEnter(address) if !self.execution.running => {
                self.select_memory(address);
                self.interaction.focused_input = Some(MEMORY_INLINE_INPUT_ID);
                return Task::done(Message::RefocusInline);
            }
            Message::MemoryReplace(address) if !self.execution.running => {
                return self.enter_memory_cell_replacement(address);
            }
            Message::RefocusInline => {
                return iced::widget::operation::focus(MEMORY_INLINE_INPUT_ID);
            }
            Message::MemoryAddressPrevious if !self.execution.running => {
                return self.step_memory_address(-1);
            }
            Message::MemoryAddressNext if !self.execution.running => {
                return self.step_memory_address(1);
            }
            Message::MemoryAddressPageUp if !self.execution.running => {
                return self.step_memory_address(-16);
            }
            Message::MemoryAddressPageDown if !self.execution.running => {
                return self.step_memory_address(16);
            }
            Message::ArrowKey(direction) => return self.handle_arrow_key(direction),
            Message::HorizontalArrowKey(direction) => {
                return self.handle_horizontal_arrow_key(direction);
            }
            Message::RegisterArrowKey(direction) => {
                return self.navigate_inline_register_target(direction);
            }
            Message::MemoryScrolled(offset, viewport_height) => {
                self.memory.memory_viewport_height = viewport_height;
                self.scroll_memory(offset);
                self.memory.memory_scroll_visible_ticks = MEMORY_SCROLL_VISIBLE_TICKS;
            }
            Message::MemoryScrollbarDragged(offset, viewport_height) => {
                self.memory.memory_viewport_height = viewport_height;
                self.scroll_memory(offset);
                self.memory.memory_scroll_visible_ticks = MEMORY_SCROLL_VISIBLE_TICKS;
                return scroll_memory_to(self.memory.memory_scroll_offset);
            }
            Message::JumpMemoryAddress if !self.execution.running => {
                if self.interaction.keyboard_modifiers.alt() {
                    return self.jump_memory_address();
                }
                return self.advance_memory_address(self.interaction.keyboard_modifiers.shift());
            }
            Message::JumpMemoryTo(address) if !self.execution.running => {
                if self.memory.view.is_stack() && address < STACK_VIEW_START {
                    self.disable_stack_view();
                }
                return self.jump_memory_to(address);
            }
            Message::MemoryAddressChanged(value) if !self.execution.running => {
                self.change_memory_address(value);
                self.interaction.focused_input = Some(MEMORY_ADDRESS_INPUT_ID);
            }
            Message::MemoryValueChanged(value) if !self.execution.running => {
                self.change_memory_value(value);
                self.interaction.focused_input = Some(MEMORY_VALUE_INPUT_ID);
            }
            Message::InlineMemoryValueChanged(address, value) if !self.execution.running => {
                self.change_inline_memory_value(address, value);
                self.interaction.focused_input = Some(MEMORY_INLINE_INPUT_ID);
            }
            Message::ApplyInlineMemoryValue(address) if !self.execution.running => {
                return self.handle_inline_memory_submit(address);
            }
            Message::PasteMemoryBytesRequested if !self.execution.running => {
                if self.selected_memory_action_address().is_none() {
                    return Task::none();
                }
                return iced::clipboard::read().map(Message::MemoryBytesPasted);
            }
            Message::MemoryBytesPasted(Some(value)) if !self.execution.running => {
                if let Some(address) = self.selected_memory_action_address() {
                    self.paste_memory_bytes(address, value);
                }
            }
            Message::MemoryBytesPasted(None) => {}
            Message::OpcodeDropdownToggled(address) if !self.execution.running => {
                return self.toggle_opcode_dropdown(address);
            }
            Message::OpcodeSearchChanged(value) if !self.execution.running => {
                return self.change_opcode_search(value);
            }
            Message::OpcodeSelected(address, value) if !self.execution.running => {
                self.select_opcode(address, value)
            }
            Message::OpcodeScrolled(offset) => self.handle_opcode_scrolled(offset),
            Message::OpcodeScrollbarDragged(offset) => {
                return self.scroll_opcode_to(offset);
            }
            Message::HideOpcodeDropdown => self.hide_opcode_dropdown(),
            Message::DismissErrorNotice => self.clear_error_notice(),
            Message::DismissHaltNotice => self.clear_halt_notice(),
            Message::DismissSettingsNotice => self.preferences.settings_notice = None,
            Message::ToggleStackView => return self.toggle_stack_view(),
            Message::EscPressed => return self.handle_esc(),
            Message::EnterPressed => {
                if self.execution.running {
                    return Task::none();
                }
                if self.memory.opcode_dropdown_address.is_some() {
                    self.apply_highlighted_opcode();
                    return Task::none();
                }
                if let Some(target) = self.register.active_register_target {
                    return Task::done(Message::RegisterEnter(target));
                }
                let Some(address) = self.selected_memory_address() else {
                    return Task::none();
                };
                return Task::done(Message::MemoryEnter(address));
            }
            Message::MemoryCellReplace if !self.execution.running => {
                return self.enter_selected_memory_replacement();
            }
            Message::MemoryCellAction => {
                if self.execution.running {
                    return Task::none();
                }
                if self.interaction.focused_input == Some(MEMORY_ADDRESS_INPUT_ID) {
                    return self.jump_memory_address();
                }
                if self.interaction.focused_input == Some(MEMORY_VALUE_INPUT_ID) {
                    return self.apply_memory_and_jump();
                }
                let Some(address) = self.selected_memory_address() else {
                    return Task::none();
                };
                let memory = &self.snapshot.cpu.memory;
                if let Some(port) = crate::view::operand_port_number(address, memory)
                    && let Some(open) = open_device_message(port)
                {
                    return self.update(open);
                }
                if let Some(target) = crate::view::operand_jump_target(address, memory) {
                    return self.jump_from_memory_operand(address, target);
                }
                return Task::done(Message::MemoryEnter(address));
            }
            Message::MemoryCellReturn => {
                if self.execution.running {
                    return Task::none();
                }
                return self.return_to_memory_operand();
            }
            Message::MemoryPatternSearch if !self.execution.running => {
                if matches!(
                    self.interaction.focused_input,
                    Some(MEMORY_ADDRESS_INPUT_ID | MEMORY_VALUE_INPUT_ID)
                ) {
                    return self.find_next_memory_address_in_direction(
                        self.interaction.keyboard_modifiers.shift(),
                    );
                }
            }
            Message::OpenOpcodePicker if !self.execution.running => {
                let Some(address) = self.selected_memory_address() else {
                    return Task::none();
                };
                let scroll = self.toggle_opcode_dropdown(address);
                if self.memory.opcode_dropdown_address.is_none() {
                    return Task::none();
                }
                self.interaction.focused_input = Some(OPCODE_SEARCH_INPUT_ID);
                return scroll.chain(iced::widget::operation::focus(OPCODE_SEARCH_INPUT_ID));
            }
            Message::ApplyMemory if !self.execution.running => {
                if self.interaction.keyboard_modifiers.alt() {
                    return self.apply_memory_and_jump();
                }
                let from_address = self.interaction.focused_input == Some(MEMORY_ADDRESS_INPUT_ID);
                let backward = self.interaction.keyboard_modifiers.shift();
                if from_address {
                    return self.advance_memory_address(backward);
                }
                return self.apply_memory_and_step(backward);
            }
            Message::ModifiersChanged(modifiers) => {
                self.interaction.keyboard_modifiers = modifiers;
            }
            Message::FocusCycle { backward } => {
                if self.memory.opcode_dropdown_address.is_some() {
                    let scroll = self.step_opcode_highlight(if backward { -1 } else { 1 });
                    self.interaction.focused_input = Some(OPCODE_SEARCH_INPUT_ID);
                    return scroll.chain(iced::widget::operation::focus(OPCODE_SEARCH_INPUT_ID));
                }
                if let Some(task) = self.cycle_selected_focus(backward) {
                    return task;
                }
                use iced::advanced::widget::operation::focusable::find_focused;
                return iced::advanced::widget::operate(find_focused())
                    .map(move |focused| Message::FocusResolved { focused, backward });
            }
            Message::FocusResolved { focused, backward } => {
                return self.cycle_focus(focused, backward);
            }
            Message::MenuToggled(menu) => {
                self.toggle_top_menu(menu);
            }
            Message::MenuHovered(menu) => self.hover_top_menu(menu),
            Message::MenuClosed => {
                self.close_top_menu();
            }
            Message::MenuCategoriesToggled => {
                self.preferences.menu_categories_visible =
                    !self.preferences.menu_categories_visible;
                if !self.preferences.menu_categories_visible {
                    self.close_top_menu();
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
