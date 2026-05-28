use crate::app::{DesktopApp, MEMORY_SCROLL_VISIBLE_TICKS, Message};
use iced::Task;
use k580_app::AppCommand;

use crate::runtime::parse::{parse_hex_u16, scroll_memory_to};

impl DesktopApp {
    pub(crate) fn step_instruction_and_advance(&mut self) -> Task<Message> {
        if self.run_blocked_after_halt {
            self.raise_halt_notice();
            return Task::none();
        }
        if self.snapshot.cpu.halted {
            self.raise_halt_notice();
            return Task::none();
        }
        self.dispatch_sync(AppCommand::StepInstruction);
        self.follow_pc_into_memory_list()
    }

    /// PC mutates on the first tact, so before/after PC comparison
    /// would teleport the cursor every press. Watch
    /// `last_tact_was_boundary` instead.
    pub(crate) fn step_tact_and_maybe_advance(&mut self) -> Task<Message> {
        if self.run_blocked_after_halt {
            self.raise_halt_notice();
            return Task::none();
        }
        if self.snapshot.cpu.halted {
            self.raise_halt_notice();
            return Task::none();
        }
        self.last_tact_was_boundary = false;
        self.dispatch_sync(AppCommand::StepTact);
        if !self.last_tact_was_boundary {
            return Task::none();
        }
        self.last_tact_was_boundary = false;
        self.follow_pc_into_memory_list()
    }

    pub(crate) fn follow_pc_into_memory_list(&mut self) -> Task<Message> {
        let pc = self.snapshot.cpu.pc;
        self.select_memory(pc);

        if self.memory_viewport_height <= 0.0 {
            return Task::none();
        }
        let Some(target_offset) = self.scroll_offset_to_reveal(pc) else {
            return Task::none();
        };
        self.scroll_memory(target_offset);
        self.memory_scroll_visible_ticks = MEMORY_SCROLL_VISIBLE_TICKS;
        scroll_memory_to(target_offset)
    }

    /// Differs from `follow_pc_into_memory_list`: skips
    /// `sync_pc_to_cursor` (PC is authoritative during a run), and
    /// preserves an in-progress inline edit on a faraway cell.
    pub(crate) fn follow_pc_during_run(&mut self) -> Task<Message> {
        // After HLT, PC sits one past the opcode but the highlight
        // should land on the HLT row itself.
        let target = if self.snapshot.cpu.halted && self.snapshot.cpu.pc > 0 {
            self.snapshot.cpu.pc.wrapping_sub(1)
        } else {
            self.snapshot.cpu.pc
        };
        let current_address = parse_hex_u16(&self.memory_address_input).ok();
        if current_address == Some(target) {
            return Task::none();
        }

        let inline_was_clean = match current_address {
            Some(addr) => {
                let stored = format!("{:02X}", self.snapshot.cpu.memory.read(addr));
                self.memory_inline_value_input.eq_ignore_ascii_case(&stored)
            }
            None => true,
        };

        self.opcode_dropdown_address = None;
        self.opcode_search_input.clear();
        self.memory_address_input = format!("{target:04X}");
        self.memory_value_input = format!("{:02X}", self.snapshot.cpu.memory.read(target));

        if inline_was_clean {
            self.memory_inline_value_input = self.memory_value_input.clone();
        }

        if self.memory_viewport_height <= 0.0 {
            return Task::none();
        }
        let Some(target_offset) = self.scroll_offset_to_reveal(target) else {
            return Task::none();
        };
        self.scroll_memory(target_offset);
        self.memory_scroll_visible_ticks = MEMORY_SCROLL_VISIBLE_TICKS;
        scroll_memory_to(target_offset)
    }
}
