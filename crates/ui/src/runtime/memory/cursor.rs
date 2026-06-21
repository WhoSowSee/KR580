use crate::app::{
    DesktopApp, MEMORY_ROW_HEIGHT, MEMORY_SCROLL_VISIBLE_TICKS, Message, STACK_VIEW_SIZE,
    STACK_VIEW_START,
};
use iced::Task;
use k580_app::AppCommand;

use crate::runtime::parse::{parse_hex_u16, scroll_memory_to};

impl DesktopApp {
    pub(crate) fn select_memory(&mut self, address: u16) {
        self.finish_replacement();
        self.active_register_target = None;
        self.inline_register_target = None;
        self.opcode_dropdown_address = None;
        self.opcode_search_input.clear();
        self.set_memory_address(self.clamp_memory_address_to_view(address));
    }

    pub(crate) fn enter_inline_memory_replacing(&mut self, address: u16) {
        self.select_memory(address);
        self.begin_replacement(crate::app::MEMORY_INLINE_INPUT_ID);
    }

    pub(crate) fn selected_memory_address(&self) -> Option<u16> {
        parse_hex_u16(&self.memory_address_input).ok()
    }

    pub(crate) fn memory_view(&self) -> (u16, usize) {
        if self.stack_view {
            (STACK_VIEW_START, STACK_VIEW_SIZE)
        } else {
            (0, crate::app::MEMORY_ADDRESS_COUNT)
        }
    }

    pub(crate) fn clamp_memory_address_to_view(&self, address: u16) -> u16 {
        let (start, count) = self.memory_view();
        let end = start.saturating_add((count.saturating_sub(1)) as u16);
        address.clamp(start, end)
    }

    pub(crate) fn toggle_stack_view(&mut self) -> Task<Message> {
        if self.stack_view {
            self.disable_stack_view();
        } else {
            self.enable_stack_view();
        }
        Task::none()
    }

    pub(crate) fn enable_stack_view(&mut self) {
        self.stack_view_saved_address = self.selected_memory_address();
        self.stack_view_saved_scroll_offset = self.memory_scroll_offset;
        self.stack_view = true;
        self.memory_address_input = format!("{STACK_VIEW_START:04X}");
        self.memory_value_input =
            format!("{:02X}", self.snapshot.cpu.memory.read(STACK_VIEW_START));
        self.memory_inline_value_input = self.memory_value_input.clone();
        self.memory_scroll_offset = 0.0;
        self.memory_scroll_first_row = 0;
        self.memory_search_pattern = None;
        self.opcode_dropdown_address = None;
        self.opcode_search_input.clear();
    }

    pub(crate) fn disable_stack_view(&mut self) {
        self.stack_view = false;
        if let Some(address) = self.stack_view_saved_address {
            self.memory_address_input = format!("{address:04X}");
            self.memory_value_input = format!("{:02X}", self.snapshot.cpu.memory.read(address));
            self.memory_inline_value_input = self.memory_value_input.clone();
        } else {
            self.memory_address_input.clear();
            self.memory_value_input.clear();
            self.memory_inline_value_input.clear();
        }
        self.memory_scroll_offset = self.stack_view_saved_scroll_offset;
        self.memory_scroll_first_row = (self.memory_scroll_offset / MEMORY_ROW_HEIGHT)
            .floor()
            .clamp(0.0, u16::MAX as f32) as u16;
        self.memory_search_pattern = None;
        self.opcode_dropdown_address = None;
        self.opcode_search_input.clear();
    }

    pub(crate) fn step_memory_address(&mut self, delta: i32) -> Task<Message> {
        let (view_start, _) = self.memory_view();
        if self.memory_address_input.is_empty() {
            self.select_memory(view_start);
            return Task::none();
        }
        let address = parse_hex_u16(&self.memory_address_input).unwrap_or(0);
        let next = self.clamp_memory_address_to_view(step_address(address, delta));
        self.select_memory(next);

        if self.memory_viewport_height <= 0.0 {
            return Task::none();
        }
        let Some(target_offset) = self.scroll_offset_to_reveal(next) else {
            return Task::none();
        };
        self.scroll_memory(target_offset);
        self.memory_scroll_visible_ticks = MEMORY_SCROLL_VISIBLE_TICKS;
        scroll_memory_to(target_offset)
    }

    /// Skips `SetPc` dispatch – sync round-trips were eating focus
    /// on the inline editor every ArrowUp/Down keystroke.
    pub(crate) fn step_memory_address_browse(&mut self, delta: i32) -> Task<Message> {
        let (view_start, _) = self.memory_view();
        if self.memory_address_input.is_empty() {
            self.memory_address_input = format!("{view_start:04X}");
            self.refresh_memory_value(view_start);
            self.memory_search_pattern = None;
            return Task::none();
        }
        let address = parse_hex_u16(&self.memory_address_input).unwrap_or(0);
        let next = self.clamp_memory_address_to_view(step_address(address, delta));

        self.opcode_dropdown_address = None;
        self.opcode_search_input.clear();
        self.memory_address_input = format!("{next:04X}");
        self.refresh_memory_value(next);
        self.memory_search_pattern = None;

        if self.memory_viewport_height <= 0.0 {
            return Task::none();
        }
        let Some(target_offset) = self.scroll_offset_to_reveal(next) else {
            return Task::none();
        };
        self.scroll_memory(target_offset);
        self.memory_scroll_visible_ticks = MEMORY_SCROLL_VISIBLE_TICKS;
        scroll_memory_to(target_offset)
    }

    pub(crate) fn scroll_memory(&mut self, offset: f32) {
        let (_, view_count) = self.memory_view();
        let max_row = view_count.saturating_sub(1) as f32;
        let max_offset = max_row * MEMORY_ROW_HEIGHT;
        self.memory_scroll_offset = offset.clamp(0.0, max_offset);
        self.memory_scroll_first_row = (self.memory_scroll_offset / MEMORY_ROW_HEIGHT)
            .floor()
            .clamp(0.0, max_row) as u16;
    }

    /// `None` if the row is already on screen.
    pub(super) fn scroll_offset_to_reveal(&self, address: u16) -> Option<f32> {
        let (view_start, _) = self.memory_view();
        let viewport = self.memory_viewport_height;
        let row_offset = (address.saturating_sub(view_start)) as f32 * MEMORY_ROW_HEIGHT;
        if viewport <= 0.0 {
            return Some(row_offset);
        }

        let row_top = row_offset;
        let row_bottom = row_top + MEMORY_ROW_HEIGHT;
        let view_top = self.memory_scroll_offset;
        let view_bottom = view_top + viewport;

        if row_top < view_top {
            Some(row_top)
        } else if row_bottom > view_bottom {
            Some((row_bottom - viewport).max(0.0))
        } else {
            None
        }
    }

    pub(crate) fn set_memory_address(&mut self, address: u16) {
        self.memory_address_input = format!("{address:04X}");
        self.refresh_memory_value(address);
        self.sync_pc_to_cursor(address);
    }

    /// Skipped when halted – PC sits past the halt opcode, and the
    /// `SetPc` round-trip would race with the halt snapshot and bump
    /// the visible address forward on every click.
    pub(super) fn sync_pc_to_cursor(&mut self, address: u16) {
        if self.snapshot.cpu.tact_phase.is_some()
            || self.snapshot.cpu.halted
            || self.snapshot.cpu.pc == address
        {
            return;
        }
        self.dispatch_sync(AppCommand::SetPc(address));
    }

    pub(crate) fn refresh_memory_value(&mut self, address: u16) {
        self.memory_value_input = format!("{:02X}", self.snapshot.cpu.memory.read(address));
        self.memory_inline_value_input = self.memory_value_input.clone();
    }
}

fn step_address(current: u16, delta: i32) -> u16 {
    if delta.is_negative() {
        current.saturating_sub((-delta) as u16)
    } else {
        current.saturating_add(delta as u16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stack_view_toggle_restores_previous_memory_view() {
        let (mut app, _) = DesktopApp::with_initial_path(None);
        app.select_memory(0x1234);
        app.memory_scroll_offset = 84.0;
        app.memory_scroll_first_row = 3;

        let _ = app.update(Message::ToggleStackView);

        assert!(app.stack_view);
        assert_eq!(app.memory_address_input, "FF00");
        assert_eq!(app.memory_scroll_offset, 0.0);
        assert_eq!(app.memory_scroll_first_row, 0);

        let _ = app.update(Message::ToggleStackView);

        assert!(!app.stack_view);
        assert_eq!(app.memory_address_input, "1234");
        assert_eq!(app.memory_scroll_offset, 84.0);
        assert_eq!(app.memory_scroll_first_row, 3);
    }
}
