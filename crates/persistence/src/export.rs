use crate::ExportError;
use docx_rs::{Docx, Paragraph, Run};
use k580_core::Cpu8080State;
use rust_xlsxwriter::Workbook;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportModel {
    pub format_version: u32,
    pub registers: Vec<(String, String)>,
    pub flags: Vec<(String, bool)>,
    pub memory: Vec<(u16, u8)>,
}

pub struct Exporters;

impl ExportModel {
    pub fn from_cpu(state: &Cpu8080State, memory_start: u16, memory_len: usize) -> Self {
        let registers = vec![
            ("A".to_owned(), hex8(state.registers.a)),
            ("B".to_owned(), hex8(state.registers.b)),
            ("C".to_owned(), hex8(state.registers.c)),
            ("D".to_owned(), hex8(state.registers.d)),
            ("E".to_owned(), hex8(state.registers.e)),
            ("H".to_owned(), hex8(state.registers.h)),
            ("L".to_owned(), hex8(state.registers.l)),
            ("PC".to_owned(), hex16(state.pc)),
            ("SP".to_owned(), hex16(state.sp)),
            ("cycles".to_owned(), state.cycle_count.to_string()),
        ];
        let flags = vec![
            ("S".to_owned(), state.flags.sign),
            ("Z".to_owned(), state.flags.zero),
            ("AC".to_owned(), state.flags.auxiliary_carry),
            ("P".to_owned(), state.flags.parity),
            ("CY".to_owned(), state.flags.carry),
        ];
        let memory = (0..memory_len)
            .map(|offset| memory_start.wrapping_add(offset as u16))
            .map(|address| (address, state.memory.read(address)))
            .collect();
        Self {
            format_version: 1,
            registers,
            flags,
            memory,
        }
    }
}

impl Exporters {
    pub fn write_txt(path: impl AsRef<Path>, model: &ExportModel) -> Result<(), ExportError> {
        std::fs::write(path, Self::to_text(model))?;
        Ok(())
    }

    pub fn write_xlsx(path: impl AsRef<Path>, model: &ExportModel) -> Result<(), ExportError> {
        let mut workbook = Workbook::new();
        {
            let sheet = workbook.add_worksheet();
            sheet
                .set_name("CPU")
                .map_err(|e| ExportError::Spreadsheet(e.to_string()))?;
            sheet
                .write_string(0, 0, "Field")
                .map_err(|e| ExportError::Spreadsheet(e.to_string()))?;
            sheet
                .write_string(0, 1, "Value")
                .map_err(|e| ExportError::Spreadsheet(e.to_string()))?;
            for (row, (name, value)) in model.registers.iter().enumerate() {
                let row = (row + 1) as u32;
                sheet
                    .write_string(row, 0, name)
                    .map_err(|e| ExportError::Spreadsheet(e.to_string()))?;
                sheet
                    .write_string(row, 1, value)
                    .map_err(|e| ExportError::Spreadsheet(e.to_string()))?;
            }
            let flag_start = model.registers.len() as u32 + 3;
            sheet
                .write_string(flag_start, 0, "Flag")
                .map_err(|e| ExportError::Spreadsheet(e.to_string()))?;
            sheet
                .write_string(flag_start, 1, "Set")
                .map_err(|e| ExportError::Spreadsheet(e.to_string()))?;
            for (idx, (name, set)) in model.flags.iter().enumerate() {
                let row = flag_start + idx as u32 + 1;
                sheet
                    .write_string(row, 0, name)
                    .map_err(|e| ExportError::Spreadsheet(e.to_string()))?;
                sheet
                    .write_string(row, 1, set.to_string())
                    .map_err(|e| ExportError::Spreadsheet(e.to_string()))?;
            }
        }
        {
            let sheet = workbook.add_worksheet();
            sheet
                .set_name("Memory")
                .map_err(|e| ExportError::Spreadsheet(e.to_string()))?;
            sheet
                .write_string(0, 0, "Address")
                .map_err(|e| ExportError::Spreadsheet(e.to_string()))?;
            sheet
                .write_string(0, 1, "Value")
                .map_err(|e| ExportError::Spreadsheet(e.to_string()))?;
            for (idx, (address, value)) in model.memory.iter().enumerate() {
                let row = (idx + 1) as u32;
                sheet
                    .write_string(row, 0, hex16(*address))
                    .map_err(|e| ExportError::Spreadsheet(e.to_string()))?;
                sheet
                    .write_string(row, 1, hex8(*value))
                    .map_err(|e| ExportError::Spreadsheet(e.to_string()))?;
            }
        }
        workbook
            .save(path)
            .map_err(|e| ExportError::Spreadsheet(e.to_string()))
    }

    pub fn write_docx(path: impl AsRef<Path>, model: &ExportModel) -> Result<(), ExportError> {
        let mut doc = Docx::new().add_paragraph(heading("KR580 CPU State"));
        doc = doc.add_paragraph(heading("Registers"));
        for (name, value) in &model.registers {
            doc = doc.add_paragraph(line(format!("{name}: {value}")));
        }
        doc = doc.add_paragraph(heading("Flags"));
        for (name, set) in &model.flags {
            doc = doc.add_paragraph(line(format!("{name}: {set}")));
        }
        doc = doc.add_paragraph(heading("Memory"));
        for (address, value) in &model.memory {
            doc = doc.add_paragraph(line(format!("{}: {}", hex16(*address), hex8(*value))));
        }
        let file = std::fs::File::create(path)?;
        doc.build()
            .pack(file)
            .map_err(|e| ExportError::Document(e.to_string()))
    }

    pub fn to_text(model: &ExportModel) -> String {
        let mut out = String::new();
        out.push_str("KR580 CPU State\n");
        out.push_str(&format!("formatVersion: {}\n\n", model.format_version));
        out.push_str("[Registers]\n");
        for (name, value) in &model.registers {
            out.push_str(&format!("{name}={value}\n"));
        }
        out.push_str("\n[Flags]\n");
        for (name, set) in &model.flags {
            out.push_str(&format!("{name}={set}\n"));
        }
        out.push_str("\n[Memory]\n");
        for (address, value) in &model.memory {
            out.push_str(&format!("{}={}\n", hex16(*address), hex8(*value)));
        }
        out
    }
}

fn heading(text: impl Into<String>) -> Paragraph {
    Paragraph::new().add_run(Run::new().bold().add_text(text.into()))
}

fn line(text: impl Into<String>) -> Paragraph {
    Paragraph::new().add_run(Run::new().add_text(text.into()))
}

fn hex8(value: u8) -> String {
    format!("{value:02X}")
}

fn hex16(value: u16) -> String {
    format!("{value:04X}")
}
