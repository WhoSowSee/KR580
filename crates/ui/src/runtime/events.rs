use std::time::{Duration, Instant};

use crate::app::DesktopApp;
use crate::app::PendingRequest;
use crate::app::StatusKind;
use crate::backend::{AppEvent, AppSnapshot, CommandResult};
use crate::i18n::Key;

use super::humanize_error;
use super::parse::parse_hex_u16;

mod actions;

impl DesktopApp {
    pub(crate) fn pull_events(&mut self) {
        for event in self.handle.drain_events() {
            self.consume_event(event);
        }
    }

    pub(crate) fn consume_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::SubprogramLoaded { path, start, end } => {
                self.finish_subprogram_load(path, start, end, None);
            }
            AppEvent::StateChanged(snapshot) => self.apply_snapshot(*snapshot),
            AppEvent::InstructionBoundaryReached(outcome) => {
                self.set_status(StatusKind::InstructionAt {
                    mnemonic: outcome.mnemonic,
                    pc_before: outcome.pc_before,
                });
            }
            AppEvent::TactAdvanced(outcome) => {
                self.set_status(StatusKind::TactProgress {
                    tact_phase: outcome.tact_phase,
                    cycle_count: outcome.cycle_count,
                });
            }
            AppEvent::PortRead { port, value } => {
                self.set_status(StatusKind::PortRead { port, value });
            }
            AppEvent::PortWritten { port, value } => {
                self.set_status(StatusKind::PortWrite { port, value });
            }
            AppEvent::HaltStateChanged(halted) => {
                self.running = false;
                self.pending_follow_pc = true;
                if halted {
                    self.set_status(StatusKind::CpuHalted);
                } else {
                    self.set_status(StatusKind::Ready);
                }
            }
            AppEvent::ErrorRaised(error) => {
                self.running = false;
                self.pending_follow_pc = true;
                tracing::error!(%error, "backend command failed");
                let humanized = humanize_error::humanize(&error, self.lang);
                self.set_status_custom(humanized.clone());
                self.error_notice =
                    Some(format!("{}: {}", self.lang.t(Key::ErrorPrefix), humanized));
                self.error_notice_dismiss_at = Some(Instant::now() + Duration::from_secs(8));
            }
            AppEvent::Stopped => {
                self.running = false;
                self.pending_follow_pc = true;
                self.set_status(StatusKind::Stopped);
            }
            AppEvent::WorkerStopped => {
                for id in self.pending_requests.keys().copied().collect::<Vec<_>>() {
                    self.undo_stack.complete_cpu(id, None, None);
                }
                self.pending_requests.clear();
                self.consume_event(AppEvent::ErrorRaised(
                    crate::backend::AppError::WorkerStopped,
                ));
                self.recompute_dirty();
            }
            AppEvent::CommandFinished { id, result } => self.finish_request(id, result),
        }
    }

    fn finish_request(
        &mut self,
        id: crate::backend::RequestId,
        result: Result<CommandResult, crate::backend::AppError>,
    ) {
        let Some(pending) = self.pending_requests.remove(&id) else {
            return;
        };
        let Err(error) = result else {
            let result = result.unwrap();
            match (pending, result) {
                (
                    PendingRequest::CpuEdit {
                        undo,
                        register_selection,
                        action,
                    },
                    CommandResult::CpuChanged { revision, change },
                ) => {
                    let register_selection = match (&action, register_selection) {
                        (
                            crate::app::BackendAction::Register {
                                source,
                                value,
                                target,
                                ..
                            },
                            Some((before, after)),
                        ) => Some((
                            before,
                            if self.register_completion_current(*source, *value, target) {
                                after
                            } else {
                                before
                            },
                        )),
                        (_, selection) => selection,
                    };
                    debug_assert!(revision <= self.snapshot.revision);
                    let action = if matches!(action, crate::app::BackendAction::Tact)
                        && change.after().tact_phase.is_some()
                    {
                        crate::app::BackendAction::None
                    } else {
                        action
                    };
                    if matches!(undo, crate::app::UndoPolicy::Record) {
                        self.undo_stack
                            .complete_cpu(id, Some(*change), register_selection);
                        self.recompute_dirty();
                    }
                    self.finish_backend_action(action);
                }
                (PendingRequest::Command { action }, CommandResult::Completed) => {
                    self.finish_backend_action(action);
                }
                (PendingRequest::LoadProgram { path, display }, CommandResult::LoadedProgram) => {
                    self.current_snapshot_path = Some(path);
                    self.current_subprogram_range = None;
                    self.undo_stack.clear();
                    self.mark_saved();
                    self.speed_tier = self.default_speed;
                    self.set_memory_address(self.snapshot.cpu.pc);
                    self.set_status(StatusKind::Opened { display });
                }
                (
                    PendingRequest::SaveProgram { path, display },
                    CommandResult::SavedProgram { state },
                ) => {
                    self.current_snapshot_path = Some(path);
                    self.current_subprogram_range = None;
                    self.saved_cpu = *state;
                    self.recompute_dirty();
                    self.set_status(StatusKind::SavedTo { display });
                }
                (
                    PendingRequest::SaveSubprogram {
                        path,
                        display,
                        start,
                        end,
                    },
                    CommandResult::SavedSubprogram { state },
                ) => {
                    self.current_snapshot_path = Some(path);
                    self.current_subprogram_range = Some((start, end));
                    let range = usize::from(start)..=usize::from(end);
                    self.saved_cpu.memory.as_mut_slice()[range.clone()]
                        .copy_from_slice(&state.memory.as_slice()[range]);
                    self.recompute_dirty();
                    self.set_status(StatusKind::SavedTo { display });
                }
                (
                    PendingRequest::LoadSubprogram {
                        dialog,
                        start,
                        edit_epoch,
                    },
                    CommandResult::LoadedSubprogram { end },
                ) => self.finish_subprogram_load(dialog.path, start, end, Some(edit_epoch)),
                (PendingRequest::Export { display }, CommandResult::Exported) => {
                    self.set_status(StatusKind::ExportTo { display });
                }
                (
                    PendingRequest::Import {
                        display,
                        edit_epoch,
                    },
                    CommandResult::Imported,
                ) => {
                    if edit_epoch == self.edit_epoch {
                        self.undo_stack.clear();
                        self.mark_saved();
                    } else {
                        self.recompute_dirty();
                    }
                    self.set_status(StatusKind::ImportFrom { display });
                }
                _ => {}
            }
            return;
        };
        match pending {
            PendingRequest::LoadSubprogram { dialog, .. } => {
                self.error_notice_dismiss_at = None;
                self.restore_subprogram_error_text(dialog, error.to_string());
            }
            PendingRequest::Command {
                action: crate::app::BackendAction::HddAttached(path),
            } if self.snapshot.devices.hdd.path.as_ref() == Some(&path) => {
                self.hdd_file_exists = false
            }
            _ => {}
        }
        self.undo_stack.complete_cpu(id, None, None);
        self.recompute_dirty();
    }

    fn apply_snapshot(&mut self, snapshot: AppSnapshot) {
        if snapshot.revision < self.snapshot.revision {
            return;
        }
        let register_value_follows_snapshot =
            crate::app::parse_register_name(&self.register_name_input)
                == Some(self.selected_register)
                && self.register_value_input
                    == format!(
                        "{:02X}",
                        self.snapshot.cpu.registers.get(self.selected_register)
                    );
        let memory_address = parse_hex_u16(&self.memory_address_input);
        let old_memory_value =
            memory_address.map(|address| format!("{:02X}", self.snapshot.cpu.memory.read(address)));
        let memory_value_follows_snapshot = old_memory_value
            .as_ref()
            .is_some_and(|value| self.memory_value_input == *value);
        let inline_value_follows_snapshot = old_memory_value
            .as_ref()
            .is_some_and(|value| self.memory_inline_value_input == *value);

        self.snapshot = snapshot;

        if !self.snapshot.cpu.halted {
            self.clear_halt_notice();
            self.run_blocked_after_halt = false;
        }

        if register_value_follows_snapshot {
            self.register_value_input = format!(
                "{:02X}",
                self.snapshot.cpu.registers.get(self.selected_register)
            );
        }

        if let Some(address) = memory_address {
            let value = format!("{:02X}", self.snapshot.cpu.memory.read(address));
            if memory_value_follows_snapshot {
                self.memory_value_input = value.clone();
            }
            if inline_value_follows_snapshot {
                self.memory_inline_value_input = value;
            }
        }
    }
}
