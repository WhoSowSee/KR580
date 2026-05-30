use crate::app::{
    DesktopApp, MEMORY_ADDRESS_INPUT_ID, MEMORY_INLINE_INPUT_ID, MEMORY_VALUE_INPUT_ID, Message,
};
use iced::Task;
use iced::widget::operation;
use k580_app::AppCommand;

use crate::runtime::parse::{bounded_hex_input, parse_hex_u8, parse_hex_u16, saturating_step_u8};

impl DesktopApp {
    pub(crate) fn change_memory_address(&mut self, value: String) {
        let Some(value) = bounded_hex_input(&value, 4) else {
            return;
        };

        self.memory_search_pattern = None;
        let before = self.memory_address_input.clone();
        self.memory_address_input = value;
        self.undo_stack.push_text(
            MEMORY_ADDRESS_INPUT_ID,
            before,
            self.memory_address_input.clone(),
        );
        if let Ok(address) = parse_hex_u16(&self.memory_address_input) {
            self.refresh_memory_value(address);
            self.sync_pc_to_cursor(address);
        }
    }

    pub(crate) fn change_memory_value(&mut self, value: String) {
        if let Some(value) = bounded_hex_input(&value, 2) {
            let before = self.memory_value_input.clone();
            self.memory_value_input = value;
            self.undo_stack.push_text(
                MEMORY_VALUE_INPUT_ID,
                before,
                self.memory_value_input.clone(),
            );
        }
    }

    pub(crate) fn step_memory_value_input(&mut self, delta: i32) {
        let current = parse_hex_u8(&self.memory_value_input).unwrap_or(0);
        let next = saturating_step_u8(current, delta);
        self.memory_value_input = format!("{next:02X}");
    }

    pub(crate) fn change_inline_memory_value(&mut self, address: u16, value: String) {
        let Some(value) = bounded_hex_input(&value, 2) else {
            return;
        };

        self.memory_address_input = format!("{address:04X}");
        self.memory_inline_value_input = value;
        // No text-undo entry: the inline buffer follows whichever
        // address is highlighted, so a text entry tied to this id
        // would be interpreted against a different address on
        // Ctrl+Z. The byte mutation lands as a `Cpu` undo pair on
        // Enter.
    }

    pub(crate) fn apply_inline_memory_value(&mut self, address: u16) {
        match parse_hex_u8(&self.memory_inline_value_input) {
            Ok(value) => {
                self.memory_address_input = format!("{address:04X}");
                self.memory_value_input = format!("{value:02X}");
                self.memory_inline_value_input = self.memory_value_input.clone();
                self.undo_stack.break_coalescing();
                self.dispatch_with_undo(AppCommand::SetMemory(address, value));
            }
            Err(error) => self.set_status_custom(error),
        }
    }

    pub(crate) fn cancel_inline_memory_edit(&mut self) -> Task<Message> {
        let Ok(address) = parse_hex_u16(&self.memory_address_input) else {
            return operation::focus(MEMORY_INLINE_INPUT_ID);
        };
        let stored = format!("{:02X}", self.snapshot.cpu.memory.read(address));
        if self.memory_inline_value_input.eq_ignore_ascii_case(&stored) {
            return Task::none();
        }
        self.memory_inline_value_input = stored.clone();
        self.memory_value_input = stored;
        operation::focus(MEMORY_INLINE_INPUT_ID)
    }

    pub(crate) fn toggle_opcode_dropdown(&mut self, address: u16) {
        if self.opcode_dropdown_address == Some(address) {
            self.opcode_dropdown_address = None;
            self.opcode_search_input.clear();
            return;
        }
        self.set_memory_address(address);
        self.opcode_dropdown_address = Some(address);
    }

    pub(crate) fn select_opcode(&mut self, address: u16, value: u8) {
        self.memory_address_input = format!("{address:04X}");
        self.memory_value_input = format!("{value:02X}");
        self.memory_inline_value_input = self.memory_value_input.clone();
        self.opcode_dropdown_address = None;
        self.opcode_search_input.clear();
        self.undo_stack.break_coalescing();
        self.dispatch_with_undo(AppCommand::SetMemory(address, value));
    }

    pub(crate) fn hide_opcode_dropdown(&mut self) {
        self.opcode_dropdown_address = None;
        self.opcode_search_input.clear();
    }

    pub(crate) fn apply_memory(&mut self) -> Task<Message> {
        match (
            parse_hex_u16(&self.memory_address_input),
            parse_hex_u8(&self.memory_value_input),
        ) {
            (Ok(address), Ok(value)) => {
                self.memory_inline_value_input = format!("{value:02X}");
                self.undo_stack.break_coalescing();
                self.dispatch_with_undo(AppCommand::SetMemory(address, value));
                Task::none()
            }
            (Err(error), _) | (_, Err(error)) => {
                self.set_status_custom(error);
                Task::none()
            }
        }
    }

    pub(crate) fn apply_memory_and_step(&mut self, backward: bool) -> Task<Message> {
        let write = self.apply_memory();
        self.step_address_in_input(backward);
        self.focused_input = Some(MEMORY_VALUE_INPUT_ID);
        write.chain(operation::focus(MEMORY_VALUE_INPUT_ID))
    }

    pub(crate) fn apply_memory_and_jump(&mut self) -> Task<Message> {
        let write = self.apply_memory();
        let jump = self.jump_memory_address();
        write.chain(jump)
    }
}
