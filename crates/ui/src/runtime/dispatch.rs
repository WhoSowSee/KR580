use std::time::Duration;

use crate::app::DesktopApp;
use k580_app::AppCommand;

pub(super) const SYNC_DISPATCH_TIMEOUT: Duration = Duration::from_millis(50);

impl DesktopApp {
    pub(crate) fn dispatch(&mut self, command: AppCommand) {
        if let Err(error) = self.handle.send(command) {
            self.set_status_custom(error.to_string());
        }
        self.pull_events();
    }

    /// Blocks until the worker publishes a `StateChanged` (or 50 ms).
    /// Without this, handlers that read `self.snapshot` right after
    /// dispatch race the channel and the user clicks twice.
    pub(crate) fn dispatch_sync(&mut self, command: AppCommand) {
        if let Err(error) = self.handle.send(command) {
            self.set_status_custom(error.to_string());
            return;
        }
        for event in self.handle.drain_until_state_change(SYNC_DISPATCH_TIMEOUT) {
            self.consume_event(event);
        }
    }

    pub(crate) fn toggle_run(&mut self) {
        // Pause wins first — otherwise once PC walks off the loaded
        // program into NOP territory the gates below refuse the press.
        if self.running {
            self.running = false;
            self.dispatch(AppCommand::Stop);
            return;
        }

        if self.run_blocked_after_halt {
            self.raise_halt_notice();
            return;
        }

        if self.snapshot.cpu.halted {
            self.raise_halt_notice();
            return;
        }
        let pc = self.snapshot.cpu.pc;
        let has_program = self.snapshot.cpu.memory.read(pc) != 0;
        if !has_program {
            self.set_status(crate::app::StatusKind::NoProgramAt { pc });
            return;
        }
        self.running = true;
        self.dispatch(AppCommand::Run);
    }

    pub(crate) fn restart_program(&mut self) {
        if self.run_blocked_after_halt {
            self.raise_halt_notice();
            return;
        }
        self.dispatch_with_undo(AppCommand::ResetCpu);
        self.running = true;
        self.dispatch(AppCommand::Run);
    }

    /// Sync dispatch is non-negotiable: an async version captured
    /// `after == before` and `push_cpu` silently dropped every entry.
    pub(crate) fn dispatch_with_undo(&mut self, command: AppCommand) {
        let before = self.snapshot.cpu.clone();
        self.dispatch_sync(command);
        let after = self.snapshot.cpu.clone();
        if before != after {
            self.dirty = true;
        }
        self.undo_stack.push_cpu(before, after);
    }
}
