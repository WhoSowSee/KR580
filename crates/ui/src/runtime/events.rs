use std::time::{Duration, Instant};

use crate::app::DesktopApp;
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
                self.status = format!("{} at {:04X}", outcome.mnemonic, outcome.pc_before)
            }
            AppEvent::TactAdvanced(outcome) => {
                if outcome.instruction_boundary {
                    self.last_tact_was_boundary = true;
                }
                self.status = format!("Такт {} цикл {}", outcome.tact_phase, outcome.cycle_count)
            }
            AppEvent::PortRead { port, value } => {
                self.status = format!("IN {port:02X} -> {value:02X}")
            }
            AppEvent::PortWritten { port, value } => {
                self.status = format!("OUT {port:02X} <- {value:02X}")
            }
            AppEvent::SnapshotFlavourLoaded(flavour) => {
                self.pending_snapshot_flavour = Some(flavour);
            }
            AppEvent::HaltStateChanged(_) => {
                self.running = false;
                // High-Hz: `running` is already false by the time Tick
                // reads it, so the closing follow-pc runs via the pending flag.
                self.pending_follow_pc = true;
                self.status = "ЦП остановлен".to_owned();
            }
            AppEvent::ErrorRaised(error) => {
                self.running = false;
                self.pending_follow_pc = true;
                let raw = error.to_string();
                // Status bar shows raw English (dev-visible); overlay gets
                // a Russian translation via `humanize_error`.
                self.status = raw.clone();
                self.error_notice = Some(format!("Ошибка: {}", humanize_error::humanize(&raw)));
                self.error_notice_dismiss_at = Some(Instant::now() + Duration::from_secs(8));
            }
            AppEvent::Stopped => {
                self.running = false;
                self.pending_follow_pc = true;
                self.status = "Остановлен".to_owned();
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
