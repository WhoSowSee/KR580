use crate::app::{
    DesktopApp, MEMORY_ADDRESS_INPUT_ID, MEMORY_INLINE_INPUT_ID, MEMORY_SCROLL_VISIBLE_TICKS,
    MEMORY_VALUE_INPUT_ID, Message, OPCODE_LIST_HEIGHT, OPCODE_OPTION_HEIGHT, OPCODE_SCROLL_ID,
    StatusKind,
};
use crate::backend::AppCommand;
use iced::Task;
use iced::widget::{operation, scrollable::AbsoluteOffset};

use crate::app::filtered_opcode_choices;
use crate::runtime::parse::{
    bounded_hex_input, parse_hex_byte_sequence, parse_hex_byte_sequence_edit, parse_hex_u8,
    parse_hex_u16, saturating_step_u8,
};

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
        if let Some(address) = parse_hex_u16(&self.memory_address_input) {
            self.refresh_memory_value(address);
            self.sync_pc_to_cursor(address);
        } else {
            self.memory_value_input.clear();
        }
    }

    pub(crate) fn change_memory_value(&mut self, value: String) {
        match parse_hex_byte_sequence_edit(&value, &self.memory_value_input) {
            Ok(Some(values)) => {
                self.materialize_input_fallback(MEMORY_ADDRESS_INPUT_ID);
                if let Some(address) = parse_hex_u16(&self.memory_address_input) {
                    self.write_memory_block(address, values);
                }
                return;
            }
            Err(()) if is_single_byte_edit_overflow(&value, &self.memory_value_input) => return,
            Err(()) => {
                self.set_status(StatusKind::InvalidMemoryBytes);
                return;
            }
            Ok(None) => {}
        }
        if let Some(value) = bounded_hex_input(&value, 2) {
            if !value.is_empty() {
                self.materialize_input_fallback(MEMORY_ADDRESS_INPUT_ID);
            }
            let before = self.memory_value_input.clone();
            self.memory_value_input = value;
            self.memory_inline_value_input = self.memory_value_input.clone();
            self.undo_stack.push_text(
                MEMORY_VALUE_INPUT_ID,
                before,
                self.memory_value_input.clone(),
            );
        }
    }

    pub(crate) fn step_memory_value_input(&mut self, delta: i32) {
        self.commit_replacement(MEMORY_VALUE_INPUT_ID);
        let current = parse_hex_u8(&self.memory_value_input).unwrap_or(0);
        let next = saturating_step_u8(current, delta);
        self.memory_value_input = format!("{next:02X}");
    }

    pub(crate) fn change_inline_memory_value(&mut self, address: u16, value: String) {
        match parse_hex_byte_sequence_edit(&value, &self.memory_inline_value_input) {
            Ok(Some(values)) => {
                self.write_memory_block(address, values);
                return;
            }
            Err(()) if is_single_byte_edit_overflow(&value, &self.memory_inline_value_input) => {
                return;
            }
            Err(()) => {
                self.set_status(StatusKind::InvalidMemoryBytes);
                return;
            }
            Ok(None) => {}
        }
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

    pub(crate) fn paste_memory_bytes(&mut self, address: u16, value: String) {
        match parse_hex_byte_sequence(&value) {
            Ok(Some(values)) => self.write_memory_block(address, values),
            Ok(None) => {}
            Err(()) => self.set_status(StatusKind::InvalidMemoryBytes),
        }
    }

    pub(crate) fn apply_inline_memory_value(&mut self, address: u16) {
        self.commit_replacement(MEMORY_INLINE_INPUT_ID);
        match parse_hex_u8(&self.memory_inline_value_input) {
            Some(value) => {
                self.memory_address_input = format!("{address:04X}");
                self.memory_value_input = format!("{value:02X}");
                self.memory_inline_value_input = self.memory_value_input.clone();
                self.undo_stack.break_coalescing();
                self.dispatch_with_undo(AppCommand::SetMemory(address, value));
            }
            None => self.set_status(StatusKind::InvalidByteHex),
        }
    }

    pub(crate) fn enter_memory_cell_replacement(&mut self, address: u16) -> Task<Message> {
        self.enter_inline_memory_replacing(address);
        self.focused_input = Some(MEMORY_INLINE_INPUT_ID);
        Task::done(Message::RefocusInline)
    }

    pub(crate) fn enter_selected_memory_replacement(&mut self) -> Task<Message> {
        if self.focused_input == Some(MEMORY_INLINE_INPUT_ID) {
            if self.replacement_input != Some(MEMORY_INLINE_INPUT_ID) {
                self.begin_replacement(MEMORY_INLINE_INPUT_ID);
            }
            return Task::none();
        }
        let Some(address) = self.selected_memory_action_address() else {
            return Task::none();
        };
        self.enter_memory_cell_replacement(address)
    }

    pub(crate) fn handle_inline_memory_submit(&mut self, address: u16) -> Task<Message> {
        let replacing = self.replacement_input == Some(MEMORY_INLINE_INPUT_ID);
        let backward = self.keyboard_modifiers.shift();
        self.apply_inline_memory_value(address);
        let step = self.step_memory_address(if backward { -1 } else { 1 });
        if replacing {
            self.begin_replacement(MEMORY_INLINE_INPUT_ID);
        }
        self.focused_input = Some(MEMORY_INLINE_INPUT_ID);
        step.chain(operation::focus(MEMORY_INLINE_INPUT_ID))
    }

    pub(crate) fn cancel_inline_memory_edit(&mut self) -> Task<Message> {
        if let Some(address) = parse_hex_u16(&self.memory_address_input) {
            let stored = format!("{:02X}", self.snapshot.cpu.memory.read(address));
            self.memory_inline_value_input = stored.clone();
            self.memory_value_input = stored;
        }
        self.focused_input = None;
        iced::advanced::widget::operate(crate::runtime::unfocus_except(
            iced::advanced::widget::Id::new("__nothing__"),
        ))
        .discard()
    }

    pub(crate) fn toggle_opcode_dropdown(&mut self, address: u16) -> Task<Message> {
        if self.opcode_dropdown_address == Some(address) {
            self.opcode_dropdown_address = None;
            self.opcode_search_input.clear();
            return Task::none();
        }
        self.set_memory_address(address);
        self.opcode_dropdown_address = Some(address);
        self.opcode_highlight_index = 0;
        self.scroll_opcode_to(0.0)
    }

    pub(crate) fn change_opcode_search(&mut self, value: String) -> Task<Message> {
        self.opcode_search_input = value;
        self.opcode_highlight_index = 0;
        self.scroll_opcode_to(0.0)
    }

    pub(crate) fn handle_opcode_scrolled(&mut self, offset: f32) {
        self.opcode_scroll_offset = offset;
        self.opcode_scroll_visible_ticks = MEMORY_SCROLL_VISIBLE_TICKS;
    }

    pub(crate) fn scroll_opcode_to(&mut self, offset: f32) -> Task<Message> {
        self.handle_opcode_scrolled(offset);
        operation::scroll_to(OPCODE_SCROLL_ID, AbsoluteOffset { x: 0.0, y: offset })
    }

    pub(crate) fn step_opcode_highlight(&mut self, delta: i32) -> Task<Message> {
        let len = filtered_opcode_choices(&self.opcode_search_input).len();
        if len == 0 {
            self.opcode_highlight_index = 0;
            return Task::none();
        }

        let current = self.opcode_highlight_index.min(len - 1) as i32;
        self.opcode_highlight_index = (current + delta).rem_euclid(len as i32) as usize;

        let row_top = self.opcode_highlight_index as f32 * OPCODE_OPTION_HEIGHT;
        let min_offset = (row_top + OPCODE_OPTION_HEIGHT - OPCODE_LIST_HEIGHT).max(0.0);
        let max_offset =
            row_top.min((len as f32 * OPCODE_OPTION_HEIGHT - OPCODE_LIST_HEIGHT).max(0.0));
        let offset = self.opcode_scroll_offset.clamp(min_offset, max_offset);
        if offset == self.opcode_scroll_offset {
            Task::none()
        } else {
            self.scroll_opcode_to(offset)
        }
    }

    pub(crate) fn highlighted_opcode_value(&self) -> Option<u8> {
        filtered_opcode_choices(&self.opcode_search_input)
            .get(self.opcode_highlight_index)
            .map(|choice| choice.value)
    }

    pub(crate) fn apply_highlighted_opcode(&mut self) {
        let Some(address) = self.opcode_dropdown_address else {
            return;
        };
        if let Some(value) = self.highlighted_opcode_value() {
            self.select_opcode(address, value);
        }
    }

    pub(crate) fn select_opcode(&mut self, address: u16, value: u8) {
        self.memory_address_input = format!("{address:04X}");
        self.memory_value_input = format!("{value:02X}");
        self.memory_inline_value_input = self.memory_value_input.clone();
        self.opcode_dropdown_address = None;
        self.opcode_search_input.clear();
        self.opcode_highlight_index = 0;
        self.undo_stack.break_coalescing();
        self.dispatch_with_undo(AppCommand::SetMemory(address, value));
    }

    pub(crate) fn hide_opcode_dropdown(&mut self) {
        self.opcode_dropdown_address = None;
        self.opcode_search_input.clear();
        self.opcode_highlight_index = 0;
    }

    pub(crate) fn apply_memory(&mut self) -> Task<Message> {
        self.commit_replacement(MEMORY_ADDRESS_INPUT_ID);
        self.commit_replacement(MEMORY_VALUE_INPUT_ID);
        match (
            parse_hex_u16(&self.memory_address_input),
            parse_hex_u8(&self.memory_value_input),
        ) {
            (Some(address), Some(value)) => {
                self.memory_inline_value_input = format!("{value:02X}");
                self.undo_stack.break_coalescing();
                self.dispatch_with_undo(AppCommand::SetMemory(address, value));
                Task::none()
            }
            (None, _) => {
                self.set_status(StatusKind::InvalidAddressHex);
                Task::none()
            }
            (_, None) => {
                self.set_status(StatusKind::InvalidByteHex);
                Task::none()
            }
        }
    }

    pub(crate) fn apply_memory_and_step(&mut self, backward: bool) -> Task<Message> {
        let write = self.apply_memory();
        self.step_address_in_input(backward);
        self.continue_replacement(MEMORY_VALUE_INPUT_ID);
        self.focused_input = Some(MEMORY_VALUE_INPUT_ID);
        write.chain(operation::focus(MEMORY_VALUE_INPUT_ID))
    }

    pub(crate) fn apply_memory_and_jump(&mut self) -> Task<Message> {
        let write = self.apply_memory();
        let jump = self.jump_memory_address();
        write.chain(jump)
    }

    fn write_memory_block(&mut self, start: u16, values: Vec<u8>) {
        let end = start as usize + values.len();
        if end > k580_core::Memory64K::SIZE {
            self.set_status(StatusKind::MemoryBytesOutOfRange);
            return;
        }
        let first = values[0];
        self.finish_replacement();
        self.undo_stack.break_coalescing();
        self.dispatch_with_undo(AppCommand::SetMemoryBlock { start, values });
        self.memory_address_input = format!("{start:04X}");
        self.memory_value_input = format!("{first:02X}");
        self.memory_inline_value_input = self.memory_value_input.clone();
    }
}

fn is_single_byte_edit_overflow(input: &str, existing: &str) -> bool {
    if existing.len() != 2 || input.len() <= 2 || input.chars().any(char::is_whitespace) {
        return false;
    }
    (0..=existing.len()).any(|split| {
        let (prefix, suffix) = existing.split_at(split);
        input
            .strip_prefix(prefix)
            .and_then(|value| value.strip_suffix(suffix))
            .is_some_and(|inserted| !inserted.is_empty())
    })
}

#[cfg(test)]
#[path = "editor/tests.rs"]
mod tests;
