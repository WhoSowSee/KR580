use crate::app::{DesktopApp, MEMORY_ADDRESS_INPUT_ID, MEMORY_ROW_HEIGHT, Message, StatusKind};
use iced::Task;
use iced::widget::operation;

use crate::runtime::parse::{parse_hex_u16, scroll_memory_to};

impl DesktopApp {
    pub(crate) fn jump_memory_address(&mut self) -> Task<Message> {
        self.commit_replacement(MEMORY_ADDRESS_INPUT_ID);
        match parse_hex_u16(&self.memory.memory_address_input) {
            Some(address) => {
                self.refresh_memory_value(address);
                if let Some(target_offset) = self.scroll_offset_to_reveal(address) {
                    self.scroll_memory(target_offset);
                    return scroll_memory_to(target_offset);
                }
                Task::none()
            }
            None => {
                self.set_status(StatusKind::InvalidAddressHex);
                Task::none()
            }
        }
    }

    /// Relocates the view to an address without changing PC.
    pub(crate) fn jump_memory_to(&mut self, address: u16) -> Task<Message> {
        self.memory.memory_address_input = format!("{address:04X}");
        self.refresh_memory_value(address);
        if let Some(target_offset) = self.scroll_offset_to_reveal(address) {
            self.scroll_memory(target_offset);
            return scroll_memory_to(target_offset);
        }
        Task::none()
    }

    pub(crate) fn jump_from_memory_operand(&mut self, origin: u16, target: u16) -> Task<Message> {
        self.memory.operand_return = Some(crate::app::OperandReturn {
            address: origin,
            scroll_offset: self.memory.memory_scroll_offset,
        });
        self.jump_memory_to(target)
    }

    pub(crate) fn return_to_memory_operand(&mut self) -> Task<Message> {
        let Some(restore) = self.memory.operand_return.take() else {
            return Task::none();
        };
        self.memory.memory_address_input = format!("{:04X}", restore.address);
        self.refresh_memory_value(restore.address);
        self.scroll_memory(restore.scroll_offset);
        scroll_memory_to(self.memory.memory_scroll_offset)
    }

    pub(crate) fn advance_memory_address(&mut self, backward: bool) -> Task<Message> {
        self.commit_replacement(MEMORY_ADDRESS_INPUT_ID);
        self.step_address_in_input(backward);
        self.continue_replacement(MEMORY_ADDRESS_INPUT_ID);
        self.interaction.focused_input = Some(MEMORY_ADDRESS_INPUT_ID);
        operation::focus(MEMORY_ADDRESS_INPUT_ID)
    }

    pub(super) fn step_address_in_input(&mut self, backward: bool) {
        let (view_start, view_count) = self.memory_view();
        let current = (parse_hex_u16(&self.memory.memory_address_input)
            .unwrap_or(view_start)
            .saturating_sub(view_start)) as i32;
        let total = view_count as i32;
        let delta = if backward { -1 } else { 1 };
        let next = view_start + ((current + delta).rem_euclid(total)) as u16;

        self.memory.memory_address_input = format!("{next:04X}");
        self.refresh_memory_value(next);
        self.memory.memory_search_pattern = None;
    }

    /// Each match overwrites the visible address; retain the original query fragment.
    pub(crate) fn find_next_memory_address_in_direction(
        &mut self,
        backward: bool,
    ) -> Task<Message> {
        if self.memory.memory_search_pattern.is_none() {
            let pattern = self.memory.memory_address_input.trim().to_ascii_uppercase();
            if pattern.is_empty() {
                self.set_status(crate::app::StatusKind::EnterHexPattern);
                return Task::none();
            }
            self.memory.memory_search_pattern = Some(pattern);
        }

        let pattern = match self.memory.memory_search_pattern.as_deref() {
            Some(pattern) if !pattern.is_empty() => pattern.to_owned(),
            _ => {
                self.set_status(crate::app::StatusKind::EnterHexPattern);
                return Task::none();
            }
        };

        let (view_start, view_count) = self.memory_view();
        let start = parse_hex_u16(&self.memory.memory_address_input).unwrap_or(view_start) as i32;
        let start_idx = (start - view_start as i32).rem_euclid(view_count as i32);
        let total = view_count as i32;
        let direction = if backward { -1 } else { 1 };

        let mut next_match = None;
        for step in 1..=total {
            let candidate_idx = (start_idx + direction * step).rem_euclid(total);
            let candidate = view_start + candidate_idx as u16;
            if format!("{candidate:04X}").contains(&pattern) {
                next_match = Some(candidate);
                break;
            }
        }

        match next_match {
            Some(address) => {
                self.memory.memory_address_input = format!("{address:04X}");
                self.refresh_memory_value(address);
                self.set_status(crate::app::StatusKind::PatternFound { pattern, address });
                let target_offset = (address.saturating_sub(view_start) as f32) * MEMORY_ROW_HEIGHT;
                self.scroll_memory(target_offset);
                scroll_memory_to(target_offset)
            }
            None => {
                self.set_status(crate::app::StatusKind::NoMatchesFor { pattern });
                Task::none()
            }
        }
    }
}

#[cfg(test)]
mod tests;
