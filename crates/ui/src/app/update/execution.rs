use super::super::{DesktopApp, Message};
use iced::Task;

impl DesktopApp {
    pub(super) fn dispatch_execution_message(
        &mut self,
        message: &Message,
    ) -> Option<Task<Message>> {
        match message {
            Message::StepInstruction => return Some(self.step_instruction_and_advance()),
            Message::RestartProgram => self.restart_program(),
            Message::StepTact => return Some(self.step_tact_and_maybe_advance()),
            Message::ToggleRun => self.toggle_run(),
            Message::ResetCpu => {
                self.execution.run_blocked_after_halt = false;
                self.dispatch_edit(
                    crate::backend::AppCommand::ResetCpu,
                    super::super::UndoPolicy::Record,
                    None,
                    super::super::BackendAction::Instruction,
                );
            }
            Message::ResetRam => {
                self.execution.run_blocked_after_halt = false;
                self.dispatch_with_undo(crate::backend::AppCommand::ResetRam);
            }
            Message::ClearHalt => {
                if !self.snapshot.cpu.halted {
                    return Some(Task::none());
                }
                self.execution.run_blocked_after_halt = false;
                self.dispatch_edit(
                    crate::backend::AppCommand::ClearHalt,
                    super::super::UndoPolicy::Record,
                    None,
                    super::super::BackendAction::KeepCursor,
                );
                self.execution.pending_follow_pc = false;
            }
            Message::ToggleHalt => {
                self.execution.run_blocked_after_halt = false;
                self.dispatch_edit(
                    crate::backend::AppCommand::ToggleHalt,
                    super::super::UndoPolicy::Record,
                    None,
                    super::super::BackendAction::KeepCursor,
                );
            }
            _ => return None,
        }
        Some(Task::none())
    }
}
