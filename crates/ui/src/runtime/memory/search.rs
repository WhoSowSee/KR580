use crate::app::{
    DesktopApp, MEMORY_ADDRESS_COUNT, MEMORY_ADDRESS_INPUT_ID, MEMORY_ROW_HEIGHT, Message,
};
use iced::Task;
use iced::widget::operation;

use crate::runtime::parse::{parse_hex_u16, scroll_memory_to};

impl DesktopApp {
    pub(crate) fn jump_memory_address(&mut self) -> Task<Message> {
        self.commit_replacement(MEMORY_ADDRESS_INPUT_ID);
        match parse_hex_u16(&self.memory_address_input) {
            Ok(address) => {
                self.refresh_memory_value(address);
                if let Some(target_offset) = self.scroll_offset_to_reveal(address) {
                    self.scroll_memory(target_offset);
                    return scroll_memory_to(target_offset);
                }
                Task::none()
            }
            Err(error) => {
                self.set_status_custom(error);
                Task::none()
            }
        }
    }

    pub(crate) fn advance_memory_address(&mut self, backward: bool) -> Task<Message> {
        self.commit_replacement(MEMORY_ADDRESS_INPUT_ID);
        self.step_address_in_input(backward);
        self.continue_replacement(MEMORY_ADDRESS_INPUT_ID);
        self.focused_input = Some(MEMORY_ADDRESS_INPUT_ID);
        operation::focus(MEMORY_ADDRESS_INPUT_ID)
    }

    pub(super) fn step_address_in_input(&mut self, backward: bool) {
        let current = parse_hex_u16(&self.memory_address_input).unwrap_or(0) as i32;
        let total = MEMORY_ADDRESS_COUNT as i32;
        let delta = if backward { -1 } else { 1 };
        let next = ((current + delta).rem_euclid(total)) as u16;

        self.memory_address_input = format!("{next:04X}");
        self.refresh_memory_value(next);
        self.memory_search_pattern = None;
    }

    /// The pattern is cached on the first call so subsequent presses
    /// iterate matches – every match overwrites the address input
    /// with a full hex code that would otherwise become the next
    /// pattern.
    pub(crate) fn find_next_memory_address_in_direction(
        &mut self,
        backward: bool,
    ) -> Task<Message> {
        if self.memory_search_pattern.is_none() {
            let pattern = self.memory_address_input.trim().to_ascii_uppercase();
            if pattern.is_empty() {
                self.set_status(crate::app::StatusKind::EnterHexPattern);
                return Task::none();
            }
            self.memory_search_pattern = Some(pattern);
        }

        let pattern = match self.memory_search_pattern.as_deref() {
            Some(pattern) if !pattern.is_empty() => pattern.to_owned(),
            _ => {
                self.set_status(crate::app::StatusKind::EnterHexPattern);
                return Task::none();
            }
        };

        let start = parse_hex_u16(&self.memory_address_input).unwrap_or(0) as i32;
        let total = MEMORY_ADDRESS_COUNT as i32;
        let direction = if backward { -1 } else { 1 };

        let mut next_match = None;
        for step in 1..=total {
            let candidate = ((start + direction * step).rem_euclid(total)) as u16;
            if format!("{candidate:04X}").contains(&pattern) {
                next_match = Some(candidate);
                break;
            }
        }

        match next_match {
            Some(address) => {
                self.memory_address_input = format!("{address:04X}");
                self.refresh_memory_value(address);
                self.set_status(crate::app::StatusKind::PatternFound { pattern, address });
                let target_offset = address as f32 * MEMORY_ROW_HEIGHT;
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
