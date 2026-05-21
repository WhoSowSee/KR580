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

use parse::parse_hex_u16;

impl DesktopApp {
    pub(crate) fn dispatch(&mut self, command: AppCommand) {
        if let Err(error) = self.handle.send(command) {
            self.status = error.to_string();
        }
        self.pull_events();
    }

    pub(crate) fn pull_events(&mut self) {
        for event in self.handle.drain_events() {
            match event {
                AppEvent::StateChanged(snapshot) => self.apply_snapshot(*snapshot),
                AppEvent::InstructionBoundaryReached(outcome) => {
                    self.status = format!("{} at {:04X}", outcome.mnemonic, outcome.pc_before)
                }
                AppEvent::TactAdvanced(outcome) => {
                    self.status =
                        format!("Tact {} cycle {}", outcome.tact_phase, outcome.cycle_count)
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
