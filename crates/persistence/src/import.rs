use crate::ImportError;
use crate::export::ExportModel;
use calamine::{Data, Reader, open_workbook_auto};
use k580_core::Cpu8080State;
use std::fs;
use std::path::Path;

pub struct Importers;

impl Importers {
    pub fn read_txt(path: impl AsRef<Path>) -> Result<ExportModel, ImportError> {
        let raw = fs::read_to_string(path)?;
        Self::parse_txt(&raw)
    }

    pub fn read_xlsx(path: impl AsRef<Path>) -> Result<ExportModel, ImportError> {
        let mut workbook = open_workbook_auto(path.as_ref())
            .map_err(|e| ImportError::Spreadsheet(e.to_string()))?;

        // Header rows (`Field/Value`, `Flag/Set`, `Address/Value`)
        // mirror the TXT `[Section]` markers.
        let sheet_name = workbook
            .sheet_names()
            .first()
            .cloned()
            .ok_or_else(|| ImportError::Spreadsheet("workbook has no sheets".to_owned()))?;
        let sheet = workbook
            .worksheet_range(&sheet_name)
            .map_err(|e| ImportError::Spreadsheet(e.to_string()))?;

        let mut registers: Vec<(String, String)> = Vec::new();
        let mut flags: Vec<(String, bool)> = Vec::new();
        let mut memory: Vec<(u16, u8)> = Vec::new();
        let mut section: Option<Section> = None;

        for row in sheet.rows() {
            let key = cell_string(row.first());
            let value = cell_string(row.get(1));
            if key.is_empty() && value.is_empty() {
                continue;
            }
            if key.eq_ignore_ascii_case("Field") && value.eq_ignore_ascii_case("Value") {
                section = Some(Section::Registers);
                continue;
            }
            if key.eq_ignore_ascii_case("Flag") && value.eq_ignore_ascii_case("Set") {
                section = Some(Section::Flags);
                continue;
            }
            if key.eq_ignore_ascii_case("Address") && value.eq_ignore_ascii_case("Value") {
                section = Some(Section::Memory);
                continue;
            }
            match section {
                Some(Section::Registers) => registers.push((key, value)),
                Some(Section::Flags) => {
                    let set = parse_bool(&value).ok_or_else(|| {
                        ImportError::Malformed(format!("invalid flag value `{value}` for `{key}`"))
                    })?;
                    flags.push((key, set));
                }
                Some(Section::Memory) => {
                    let address = parse_u16_hex(&key).ok_or_else(|| {
                        ImportError::Malformed(format!("invalid memory address `{key}`"))
                    })?;
                    let cell = parse_u8_hex(&value).ok_or_else(|| {
                        ImportError::Malformed(format!("invalid memory value `{value}`"))
                    })?;
                    memory.push((address, cell));
                }
                None => {
                    return Err(ImportError::Malformed(format!(
                        "data row `{key}`/`{value}` outside of any section"
                    )));
                }
            }
        }

        Ok(ExportModel {
            registers,
            flags,
            memory,
        })
    }

    fn parse_txt(text: &str) -> Result<ExportModel, ImportError> {
        let mut registers: Vec<(String, String)> = Vec::new();
        let mut flags: Vec<(String, bool)> = Vec::new();
        let mut memory: Vec<(u16, u8)> = Vec::new();
        let mut section: Option<Section> = None;

        for raw_line in text.lines() {
            let line = raw_line.trim();
            if line.is_empty() {
                continue;
            }
            match line {
                "[Registers]" => {
                    section = Some(Section::Registers);
                    continue;
                }
                "[Flags]" => {
                    section = Some(Section::Flags);
                    continue;
                }
                "[Memory]" => {
                    section = Some(Section::Memory);
                    continue;
                }
                _ => {}
            }
            let Some((key, value)) = line.split_once('=') else {
                return Err(ImportError::Malformed(format!(
                    "expected `key=value` line, got `{line}`"
                )));
            };
            let key = key.trim().to_owned();
            let value = value.trim().to_owned();
            match section {
                Some(Section::Registers) => registers.push((key, value)),
                Some(Section::Flags) => {
                    let set = parse_bool(&value).ok_or_else(|| {
                        ImportError::Malformed(format!("invalid flag value `{value}` for `{key}`"))
                    })?;
                    flags.push((key, set));
                }
                Some(Section::Memory) => {
                    let address = parse_u16_hex(&key).ok_or_else(|| {
                        ImportError::Malformed(format!("invalid memory address `{key}`"))
                    })?;
                    let cell = parse_u8_hex(&value).ok_or_else(|| {
                        ImportError::Malformed(format!("invalid memory value `{value}`"))
                    })?;
                    memory.push((address, cell));
                }
                None => {
                    return Err(ImportError::Malformed(format!(
                        "data line `{line}` outside of any section"
                    )));
                }
            }
        }

        Ok(ExportModel {
            registers,
            flags,
            memory,
        })
    }
}

impl ExportModel {
    pub fn apply_to(&self, state: &mut Cpu8080State) -> Result<(), ImportError> {
        for (name, value) in &self.registers {
            match name.as_str() {
                "A" => {
                    state.registers.a = parse_u8_hex(value).ok_or_else(|| reg_err(name, value))?
                }
                "B" => {
                    state.registers.b = parse_u8_hex(value).ok_or_else(|| reg_err(name, value))?
                }
                "C" => {
                    state.registers.c = parse_u8_hex(value).ok_or_else(|| reg_err(name, value))?
                }
                "D" => {
                    state.registers.d = parse_u8_hex(value).ok_or_else(|| reg_err(name, value))?
                }
                "E" => {
                    state.registers.e = parse_u8_hex(value).ok_or_else(|| reg_err(name, value))?
                }
                "H" => {
                    state.registers.h = parse_u8_hex(value).ok_or_else(|| reg_err(name, value))?
                }
                "L" => {
                    state.registers.l = parse_u8_hex(value).ok_or_else(|| reg_err(name, value))?
                }
                "PC" => state.pc = parse_u16_hex(value).ok_or_else(|| reg_err(name, value))?,
                "SP" => state.sp = parse_u16_hex(value).ok_or_else(|| reg_err(name, value))?,
                "cycles" => {
                    state.cycle_count = value.parse::<u64>().map_err(|_| reg_err(name, value))?;
                }
                _ => {
                    return Err(ImportError::Malformed(format!("unknown register `{name}`")));
                }
            }
        }

        for (name, set) in &self.flags {
            match name.as_str() {
                "S" => state.flags.sign = *set,
                "Z" => state.flags.zero = *set,
                "AC" => state.flags.auxiliary_carry = *set,
                "P" => state.flags.parity = *set,
                "CY" => state.flags.carry = *set,
                _ => {
                    return Err(ImportError::Malformed(format!("unknown flag `{name}`")));
                }
            }
        }

        for (address, value) in &self.memory {
            state.memory.write(*address, *value);
        }

        Ok(())
    }
}

#[derive(Clone, Copy)]
enum Section {
    Registers,
    Flags,
    Memory,
}

fn cell_string(cell: Option<&Data>) -> String {
    match cell {
        Some(Data::String(s)) => s.trim().to_owned(),
        Some(Data::Float(f)) => {
            if f.fract() == 0.0 {
                format!("{}", *f as i64)
            } else {
                f.to_string()
            }
        }
        Some(Data::Int(i)) => i.to_string(),
        Some(Data::Bool(b)) => b.to_string(),
        Some(Data::DateTime(d)) => d.to_string(),
        Some(Data::DateTimeIso(s)) => s.trim().to_owned(),
        Some(Data::DurationIso(s)) => s.trim().to_owned(),
        Some(Data::Error(e)) => format!("{e:?}"),
        Some(Data::Empty) | None => String::new(),
    }
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "set" => Some(true),
        "false" | "0" | "no" | "unset" => Some(false),
        _ => None,
    }
}

fn parse_u8_hex(value: &str) -> Option<u8> {
    let v = value
        .trim()
        .trim_start_matches("0x")
        .trim_start_matches("0X");
    u8::from_str_radix(v, 16).ok()
}

fn parse_u16_hex(value: &str) -> Option<u16> {
    let v = value
        .trim()
        .trim_start_matches("0x")
        .trim_start_matches("0X");
    u16::from_str_radix(v, 16).ok()
}

fn reg_err(name: &str, value: &str) -> ImportError {
    ImportError::Malformed(format!("invalid value `{value}` for register `{name}`"))
}
