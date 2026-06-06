use crate::app::{DesktopApp, MEMORY_ROW_HEIGHT, MEMORY_SCROLL_VISIBLE_TICKS, Message};
use iced::Task;
use k580_app::AppCommand;

use crate::runtime::parse::{parse_hex_u16, scroll_memory_to};

impl DesktopApp {
    pub(crate) fn select_memory(&mut self, address: u16) {
        self.active_register_target = None;
        self.inline_register_target = None;
        self.opcode_dropdown_address = None;
        self.opcode_search_input.clear();
        self.set_memory_address(address);
    }

    pub(crate) fn selected_memory_address(&self) -> Option<u16> {
        parse_hex_u16(&self.memory_address_input).ok()
    }

    pub(crate) fn step_memory_address(&mut self, delta: i32) -> Task<Message> {
        if self.memory_address_input.is_empty() {
            self.select_memory(0x0000);
            return Task::none();
        }
        let address = parse_hex_u16(&self.memory_address_input).unwrap_or(0);
        let next = step_address(address, delta);
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
        if self.memory_address_input.is_empty() {
            self.memory_address_input = "0000".to_owned();
            self.refresh_memory_value(0x0000);
            self.memory_search_pattern = None;
            return Task::none();
        }
        let address = parse_hex_u16(&self.memory_address_input).unwrap_or(0);
        let next = step_address(address, delta);

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
        self.memory_scroll_offset = offset.max(0.0);
        self.memory_scroll_first_row = (self.memory_scroll_offset / MEMORY_ROW_HEIGHT)
            .floor()
            .clamp(0.0, u16::MAX as f32) as u16;
    }

    /// `None` if the row is already on screen.
    pub(super) fn scroll_offset_to_reveal(&self, address: u16) -> Option<f32> {
        let viewport = self.memory_viewport_height;
        if viewport <= 0.0 {
            return Some(address as f32 * MEMORY_ROW_HEIGHT);
        }

        let row_top = address as f32 * MEMORY_ROW_HEIGHT;
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

    pub(super) fn refresh_memory_value(&mut self, address: u16) {
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
