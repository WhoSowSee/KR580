use crate::app::DesktopApp;
use k580_app::{AppCommand, AppEvent};
use k580_core::RegisterName;

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
                AppEvent::StateChanged(snapshot) => self.snapshot = *snapshot,
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

    pub(crate) fn open_snapshot(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("KR580 snapshot", &["580"])
            .pick_file()
        {
            self.dispatch(AppCommand::LoadSnapshot(path));
        }
    }

    pub(crate) fn save_snapshot(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("KR580 snapshot", &["580"])
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

    pub(crate) fn apply_register_a(&mut self) {
        match parse_hex_u8(&self.register_a_input) {
            Ok(value) => self.dispatch(AppCommand::SetRegister(RegisterName::A, value)),
            Err(error) => self.status = error,
        }
    }

    pub(crate) fn apply_memory(&mut self) {
        match (
            parse_hex_u16(&self.memory_address_input),
            parse_hex_u8(&self.memory_value_input),
        ) {
            (Ok(address), Ok(value)) => self.dispatch(AppCommand::SetMemory(address, value)),
            (Err(error), _) | (_, Err(error)) => self.status = error,
        }
    }
}

fn parse_hex_u8(input: &str) -> Result<u8, String> {
    u8::from_str_radix(input.trim().trim_start_matches("0x"), 16)
        .map_err(|_| format!("Invalid byte hex: {input}"))
}

fn parse_hex_u16(input: &str) -> Result<u16, String> {
    u16::from_str_radix(input.trim().trim_start_matches("0x"), 16)
        .map_err(|_| format!("Invalid address hex: {input}"))
}
