//! Runtime-side helpers for `DesktopApp`: command dispatch, snapshot
//! reconciliation, file dialogs, and follow-on submodules that group the
//! per-panel update logic.
//!
//! The actual editor methods are split by responsibility:
//!
//! - `register` — register name/value editing.
//! - `memory` — memory list, address spinner, inline editor, search.
//! - `focus` — Tab/Shift+Tab cycling.
//! - `parse` — small free helpers (hex parsing, saturating step).

mod focus;
mod memory;
mod parse;
mod register;

use crate::app::DesktopApp;
use k580_app::{AppCommand, AppEvent, AppSnapshot};
use std::time::Duration;

use parse::parse_hex_u16;

/// Upper bound on how long a UI handler will wait for the worker thread
/// to publish the post-command snapshot. The emulator is single-stepping
/// or running ≤100k T-states per command, all of which complete in
/// well under a millisecond on contemporary hardware; 50 ms is generous
/// enough to absorb scheduler hiccups without making the UI feel
/// sluggish if the worker has actually died.
const SYNC_DISPATCH_TIMEOUT: Duration = Duration::from_millis(50);

impl DesktopApp {
    pub(crate) fn dispatch(&mut self, command: AppCommand) {
        if let Err(error) = self.handle.send(command) {
            self.status = error.to_string();
        }
        self.pull_events();
    }

    /// Same as `dispatch`, but blocks until the worker thread has
    /// published a `StateChanged` for this command (or 50 ms have
    /// elapsed). Used by handlers that immediately read fields of
    /// `self.snapshot` to decide what to do next — e.g. the step button
    /// follows the new program counter in the memory list. Without this
    /// the snapshot read races the channel and the user has to click
    /// twice for the highlight to move.
    pub(crate) fn dispatch_sync(&mut self, command: AppCommand) {
        if let Err(error) = self.handle.send(command) {
            self.status = error.to_string();
            return;
        }
        for event in self.handle.drain_until_state_change(SYNC_DISPATCH_TIMEOUT) {
            self.consume_event(event);
        }
    }

    /// Flips the visual run/pause state of the action panel's leftmost
    /// button and conditionally dispatches the underlying CPU command.
    /// Mirrors the reference KR-580 emulator: clicking the play glyph on a
    /// page that has no program (the byte at `cpu.pc` is `0x00`) is purely
    /// cosmetic — the icon swaps to a red pause and back without burning
    /// any T-states. When a program *is* present the toggle still drives
    /// the real `Run` / `Stop` commands, so a user who has actually loaded
    /// code keeps the original behaviour.
    pub(crate) fn toggle_run(&mut self) {
        self.running = !self.running;
        let pc = self.snapshot.cpu.pc;
        let has_program = self.snapshot.cpu.memory.read(pc) != 0;
        if !has_program {
            // Empty memory at PC — only the icon flips. No CPU work, no
            // status churn beyond a hint so the user understands why
            // nothing is moving.
            self.status = if self.running {
                format!("No program at {pc:04X}")
            } else {
                "Stopped".to_owned()
            };
            return;
        }
        if self.running {
            self.dispatch(AppCommand::Run);
        } else {
            self.dispatch(AppCommand::Stop);
        }
    }

    /// Resets the CPU registers/flags and re-runs the program from
    /// `0x0000`. Bound to the action panel's second button while the run
    /// state is armed: the icon swaps from `step-forward` to
    /// `refresh-ccw`, and pressing it acts as a "restart from the
    /// beginning" gesture mirroring the reference KR-580 emulator.
    /// Memory is preserved (only `ResetCpu` is sent, not `ResetRam`), so
    /// the loaded program survives the restart and starts executing again
    /// at PC = 0.
    pub(crate) fn restart_program(&mut self) {
        self.dispatch(AppCommand::ResetCpu);
        // Keep the run state armed and dispatch the run command directly,
        // so the visible play/pause toggle stays consistent with what the
        // CPU is actually doing.
        self.running = true;
        self.dispatch(AppCommand::Run);
    }

    pub(crate) fn pull_events(&mut self) {
        for event in self.handle.drain_events() {
            self.consume_event(event);
        }
    }

    fn consume_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::StateChanged(snapshot) => self.apply_snapshot(*snapshot),
            AppEvent::InstructionBoundaryReached(outcome) => {
                self.status = format!("{} at {:04X}", outcome.mnemonic, outcome.pc_before)
            }
            AppEvent::TactAdvanced(outcome) => {
                if outcome.instruction_boundary {
                    self.last_tact_was_boundary = true;
                }
                self.status = format!("Tact {} cycle {}", outcome.tact_phase, outcome.cycle_count)
            }
            AppEvent::PortRead { port, value } => {
                self.status = format!("IN {port:02X} -> {value:02X}")
            }
            AppEvent::PortWritten { port, value } => {
                self.status = format!("OUT {port:02X} <- {value:02X}")
            }
            AppEvent::HaltStateChanged(_) => self.status = "CPU halted".to_owned(),
            AppEvent::ErrorRaised(error) => self.status = error.to_string(),
            AppEvent::Stopped => self.status = "Stopped".to_owned(),
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

    pub(crate) fn open_snapshot(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("KR580 file", &["580"])
            .pick_file()
        {
            self.dispatch(AppCommand::LoadSnapshot(path));
        }
    }

    pub(crate) fn save_snapshot(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("KR580 file", &["580"])
            .save_file()
        {
            self.dispatch(AppCommand::SaveSnapshot(path));
        }
    }

    pub(crate) fn export_txt(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Text", &["txt"])
            .save_file()
        {
            self.dispatch(AppCommand::ExportTxt(path));
        }
    }

    pub(crate) fn export_xlsx(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Spreadsheet", &["xlsx"])
            .save_file()
        {
            self.dispatch(AppCommand::ExportXlsx(path));
        }
    }

    pub(crate) fn export_docx(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Document", &["docx"])
            .save_file()
        {
            self.dispatch(AppCommand::ExportDocx(path));
        }
    }
}
