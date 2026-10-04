use crate::app::DesktopApp;
use crate::app::PendingRequest;
use crate::backend::AppCommand;
use crate::backend::RequestId;

impl DesktopApp {
    pub(crate) fn dispatch(&mut self, command: AppCommand) {
        if let Err(error) = self.handle.send(command) {
            self.set_status_custom(error.to_string());
        }
        self.pull_events();
    }

    pub(crate) fn dispatch_request(&mut self, command: AppCommand) {
        self.dispatch_pending_request(
            command,
            PendingRequest::Command {
                action: crate::app::BackendAction::None,
            },
        );
    }

    pub(crate) fn dispatch_action(
        &mut self,
        command: AppCommand,
        action: crate::app::BackendAction,
    ) -> bool {
        self.enqueue_pending_request(command, PendingRequest::Command { action })
    }

    pub(crate) fn dispatch_edit(
        &mut self,
        command: AppCommand,
        undo: crate::app::UndoPolicy,
        register_selection: Option<(k580_core::RegisterName, k580_core::RegisterName)>,
        action: crate::app::BackendAction,
    ) -> bool {
        self.enqueue_pending_request(
            AppCommand::Edit(Box::new(command)),
            PendingRequest::CpuEdit {
                undo,
                register_selection,
                action,
            },
        )
    }

    pub(crate) fn dispatch_async_request(&mut self, command: AppCommand) -> Option<RequestId> {
        match self.handle.send_request(command) {
            Ok(id) => Some(id),
            Err(error) => {
                self.set_status_custom(error.to_string());
                None
            }
        }
    }

    pub(crate) fn dispatch_pending_request(
        &mut self,
        command: AppCommand,
        pending: PendingRequest,
    ) {
        self.enqueue_pending_request(command, pending);
    }

    fn enqueue_pending_request(&mut self, command: AppCommand, pending: PendingRequest) -> bool {
        if matches!(pending, PendingRequest::CpuEdit { .. }) && self.cpu_document_pending() {
            self.show_error_notice(self.lang.t(crate::i18n::Key::ErrDeviceBusy));
            return false;
        }
        if self.pending_requests.len() >= 128 {
            self.show_error_notice(self.lang.t(crate::i18n::Key::ErrDeviceBusy));
            return false;
        }
        let Some(id) = self.dispatch_async_request(command) else {
            return false;
        };
        if matches!(pending, PendingRequest::CpuEdit { .. }) {
            self.edit_epoch = self.edit_epoch.wrapping_add(1);
        }
        if matches!(
            pending,
            PendingRequest::CpuEdit {
                undo: crate::app::UndoPolicy::Record,
                ..
            }
        ) {
            self.dirty = true;
            self.undo_stack.reserve_cpu(id);
        }
        self.pending_requests.insert(id, pending);
        true
    }

    pub(crate) fn toggle_run(&mut self) {
        if self.cpu_document_pending() {
            self.show_error_notice(self.lang.t(crate::i18n::Key::ErrDeviceBusy));
            return;
        }
        let restarting = self.pending_requests.values().any(|request| {
            matches!(
                request,
                PendingRequest::CpuEdit {
                    action: crate::app::BackendAction::Restart,
                    ..
                }
            )
        });
        if self.running || restarting {
            for request in self.pending_requests.values_mut() {
                if let PendingRequest::CpuEdit { action, .. } = request
                    && matches!(action, crate::app::BackendAction::Restart)
                {
                    *action = crate::app::BackendAction::None;
                }
            }
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
        self.dispatch_edit(
            AppCommand::ResetCpu,
            crate::app::UndoPolicy::Record,
            None,
            crate::app::BackendAction::Restart,
        );
    }

    pub(crate) fn dispatch_with_undo(&mut self, command: AppCommand) {
        self.dispatch_edit(
            command,
            crate::app::UndoPolicy::Record,
            None,
            crate::app::BackendAction::None,
        );
    }

    pub(crate) fn cpu_document_pending(&self) -> bool {
        self.pending_requests.values().any(|request| {
            matches!(
                request,
                PendingRequest::LoadProgram { .. }
                    | PendingRequest::Command {
                        action: crate::app::BackendAction::NewFile { .. }
                    }
            )
        })
    }
}
