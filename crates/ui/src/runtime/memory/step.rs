use crate::app::{DesktopApp, MEMORY_SCROLL_VISIBLE_TICKS, Message};
use crate::backend::AppCommand;
use iced::Task;

use crate::runtime::parse::{parse_hex_u16, scroll_memory_to};

impl DesktopApp {
    pub(crate) fn step_instruction_and_advance(&mut self) -> Task<Message> {
        if self.execution.run_blocked_after_halt {
            self.raise_halt_notice();
            return Task::none();
        }
        if self.snapshot.cpu.halted {
            self.raise_halt_notice();
            return Task::none();
        }
        self.dispatch_edit(
            AppCommand::StepInstruction,
            crate::app::UndoPolicy::Skip,
            None,
            crate::app::BackendAction::Instruction,
        );
        Task::none()
    }

    /// PC advances before instruction completion; follow it only at a tact boundary.
    pub(crate) fn step_tact_and_maybe_advance(&mut self) -> Task<Message> {
        if self.execution.run_blocked_after_halt {
            self.raise_halt_notice();
            return Task::none();
        }
        if self.snapshot.cpu.halted {
            self.raise_halt_notice();
            return Task::none();
        }
        self.dispatch_edit(
            AppCommand::StepTact,
            crate::app::UndoPolicy::Skip,
            None,
            crate::app::BackendAction::Tact,
        );
        Task::none()
    }

    pub(crate) fn follow_pc_after_execution_boundary(&mut self) -> Task<Message> {
        if self.snapshot.cpu.halted {
            self.execution.pending_follow_pc = false;
            self.follow_pc_during_run()
        } else {
            self.follow_pc_into_memory_list()
        }
    }

    pub(crate) fn follow_pc_into_memory_list(&mut self) -> Task<Message> {
        let pc = self.snapshot.cpu.pc;
        self.select_memory(pc);

        if self.memory.memory_viewport_height <= 0.0 {
            return Task::none();
        }
        let Some(target_offset) = self.scroll_offset_to_reveal(pc) else {
            return Task::none();
        };
        self.scroll_memory(target_offset);
        self.memory.memory_scroll_visible_ticks = MEMORY_SCROLL_VISIBLE_TICKS;
        scroll_memory_to(target_offset)
    }

    /// During execution PC owns the cursor; preserve unrelated inline drafts.
    pub(crate) fn follow_pc_during_run(&mut self) -> Task<Message> {
        // HLT advances PC past its opcode; the memory highlight must retain the halt address.
        let target = if self.snapshot.cpu.halted && self.snapshot.cpu.pc > 0 {
            self.snapshot.cpu.pc.wrapping_sub(1)
        } else {
            self.snapshot.cpu.pc
        };
        let current_address = parse_hex_u16(&self.memory.memory_address_input);
        if current_address == Some(target) {
            return Task::none();
        }

        let inline_was_clean = match current_address {
            Some(addr) => {
                let stored = format!("{:02X}", self.snapshot.cpu.memory.read(addr));
                self.memory
                    .memory_inline_value_input
                    .eq_ignore_ascii_case(&stored)
            }
            None => true,
        };

        self.memory.opcode_dropdown_address = None;
        self.memory.opcode_search_input.clear();
        self.memory.memory_address_input = format!("{target:04X}");
        self.memory.memory_value_input = format!("{:02X}", self.snapshot.cpu.memory.read(target));

        if inline_was_clean {
            self.memory.memory_inline_value_input = self.memory.memory_value_input.clone();
        }

        if self.memory.memory_viewport_height <= 0.0 {
            return Task::none();
        }
        let Some(target_offset) = self.scroll_offset_to_reveal(target) else {
            return Task::none();
        };
        self.scroll_memory(target_offset);
        self.memory.memory_scroll_visible_ticks = MEMORY_SCROLL_VISIBLE_TICKS;
        scroll_memory_to(target_offset)
    }

    pub(crate) fn track_pc_in_place(&mut self) {
        let target = if self.snapshot.cpu.halted && self.snapshot.cpu.pc > 0 {
            self.snapshot.cpu.pc.wrapping_sub(1)
        } else {
            self.snapshot.cpu.pc
        };
        self.memory.memory_address_input = format!("{target:04X}");
        self.memory.memory_value_input = format!("{:02X}", self.snapshot.cpu.memory.read(target));
        self.memory.memory_inline_value_input = self.memory.memory_value_input.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::DesktopApp;
    use crate::app::Message;
    use crate::app::test_support::settle_backend;

    fn app_with_clean_startup() -> DesktopApp {
        let (mut app, _) = DesktopApp::with_initial_path(None);
        settle_backend(&mut app);
        app
    }

    #[test]
    fn manual_halt_toggle_does_not_move_memory_selection() {
        let mut app = app_with_clean_startup();
        app.select_memory(0x0010);

        let _ = app.update(Message::ToggleHalt);
        settle_backend(&mut app);
        assert!(app.snapshot.cpu.halted);
        assert_eq!(app.snapshot.cpu.pc, 0x0010);
        assert_eq!(app.memory.memory_address_input, "0010");

        let _ = app.update(Message::Tick);

        assert_eq!(app.memory.memory_address_input, "0010");

        let _ = app.update(Message::ToggleHalt);
        settle_backend(&mut app);
        let _ = app.update(Message::Tick);

        assert!(!app.snapshot.cpu.halted);
        assert_eq!(app.snapshot.cpu.pc, 0x0010);
        assert_eq!(app.memory.memory_address_input, "0010");
        let _ = app.update(Message::ToggleHalt);
        let _ = app.update(Message::ToggleHalt);
        settle_backend(&mut app);
        assert!(!app.snapshot.cpu.halted);
        assert_eq!(app.memory.memory_address_input, "0010");
    }

    #[test]
    fn reset_cpu_follows_pc_after_halt_was_cleared_manually() {
        let mut app = app_with_clean_startup();
        app.select_memory(0x0010);

        let _ = app.update(Message::ToggleHalt);
        settle_backend(&mut app);
        let _ = app.update(Message::ToggleHalt);
        settle_backend(&mut app);
        let _ = app.update(Message::ResetCpu);
        settle_backend(&mut app);
        let _ = app.update(Message::Tick);

        assert!(!app.snapshot.cpu.halted);
        assert_eq!(app.snapshot.cpu.pc, 0x0000);
        assert_eq!(app.memory.memory_address_input, "0000");
    }

    #[test]
    fn step_instruction_keeps_halt_opcode_selected() {
        let mut app = app_with_clean_startup();
        app.select_memory(0x0010);
        app.select_opcode(0x0010, 0x76);

        let _ = app.update(Message::StepInstruction);
        settle_backend(&mut app);
        assert!(app.snapshot.cpu.halted);
        assert_eq!(app.snapshot.cpu.pc, 0x0011);
        assert_eq!(app.memory.memory_address_input, "0010");

        let _ = app.update(Message::Tick);

        assert_eq!(app.memory.memory_address_input, "0010");
    }

    #[test]
    fn step_tact_keeps_halt_opcode_selected() {
        let mut app = app_with_clean_startup();
        app.select_memory(0x0010);
        app.select_opcode(0x0010, 0x76);

        for _ in 0..7 {
            let _ = app.update(Message::StepTact);
        }

        settle_backend(&mut app);
        assert!(app.snapshot.cpu.halted);
        assert_eq!(app.snapshot.cpu.pc, 0x0011);
        assert_eq!(app.memory.memory_address_input, "0010");

        let _ = app.update(Message::Tick);

        assert_eq!(app.memory.memory_address_input, "0010");
    }
}
