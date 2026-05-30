use std::time::{Duration, Instant};

use crate::app::DesktopApp;
use crate::app::StatusKind;
use crate::i18n::Key;
use k580_app::{AppEvent, AppSnapshot};

use super::humanize_error;
use super::parse::parse_hex_u16;

impl DesktopApp {
    pub(crate) fn pull_events(&mut self) {
        for event in self.handle.drain_events() {
            self.consume_event(event);
        }
    }

    pub(super) fn consume_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::StateChanged(snapshot) => self.apply_snapshot(*snapshot),
            AppEvent::InstructionBoundaryReached(outcome) => {
                self.set_status(StatusKind::InstructionAt {
                    mnemonic: outcome.mnemonic,
                    pc_before: outcome.pc_before,
                });
            }
            AppEvent::TactAdvanced(outcome) => {
                if outcome.instruction_boundary {
                    self.last_tact_was_boundary = true;
                }
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
            AppEvent::SnapshotFlavourLoaded(flavour) => {
                self.pending_snapshot_flavour = Some(flavour);
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
                let raw = error.to_string();
                self.set_status_custom(raw.clone());
                self.error_notice = Some(format!(
                    "{}: {}",
                    self.lang.t(Key::ErrorPrefix),
                    humanize_error::humanize(&raw, self.lang)
                ));
                self.error_notice_dismiss_at = Some(Instant::now() + Duration::from_secs(8));
            }
            AppEvent::Stopped => {
                self.running = false;
                self.pending_follow_pc = true;
                self.set_status(StatusKind::Stopped);
            }
        }
    }

    fn apply_snapshot(&mut self, snapshot: AppSnapshot) {
        let register_value_follows_snapshot =
            crate::app::parse_register_name(&self.register_name_input)
                == Some(self.selected_register)
                && self.register_value_input
                    == format!(
                        "{:02X}",
                        self.snapshot.cpu.registers.get(self.selected_register)
                    );
        let memory_address = parse_hex_u16(&self.memory_address_input).ok();
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
