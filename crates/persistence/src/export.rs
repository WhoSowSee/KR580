use crate::ExportError;
use k580_core::{Cpu8080State, decode_opcode};
use std::path::Path;

mod xlsx;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportRegisterKind {
    Accumulator,
    W,
    Z,
    B,
    C,
    D,
    E,
    H,
    L,
    StackPointer,
    ProgramCounter,
    Cycles,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportFlagKind {
    Sign,
    Zero,
    AuxiliaryCarry,
    Parity,
    Carry,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportOptions {
    pub page_name: String,
    pub memory_start: u16,
    pub memory_end: u16,
    pub include_memory_address: bool,
    pub include_memory_value: bool,
    pub include_memory_command: bool,
    pub include_comment_column: bool,
    pub registers: Vec<ExportRegisterKind>,
    pub flags: Vec<ExportFlagKind>,
    pub xlsx_pages: Vec<ExportXlsxPage>,
    pub text_sections: Vec<ExportTextSection>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportXlsxPage {
    pub name: String,
    pub memory_start: u16,
    pub memory_end: u16,
    pub include_memory_address: bool,
    pub include_memory_value: bool,
    pub include_memory_command: bool,
    pub include_comment_column: bool,
    pub registers: Vec<ExportRegisterKind>,
    pub flags: Vec<ExportFlagKind>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportTextSection {
    pub name: String,
    pub memory_start: u16,
    pub memory_end: u16,
    pub include_memory_address: bool,
    pub include_memory_value: bool,
    pub include_memory_command: bool,
    pub registers: Vec<ExportRegisterKind>,
    pub flags: Vec<ExportFlagKind>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportModel {
    pub registers: Vec<(String, String)>,
    pub flags: Vec<(String, bool)>,
    pub memory: Vec<(u16, u8)>,
}

pub struct Exporters;

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            page_name: "State".to_owned(),
            memory_start: 0,
            memory_end: u16::MAX,
            include_memory_address: true,
            include_memory_value: true,
            include_memory_command: false,
            include_comment_column: false,
            registers: vec![
                ExportRegisterKind::Accumulator,
                ExportRegisterKind::B,
                ExportRegisterKind::C,
                ExportRegisterKind::D,
                ExportRegisterKind::E,
                ExportRegisterKind::H,
                ExportRegisterKind::L,
                ExportRegisterKind::StackPointer,
                ExportRegisterKind::ProgramCounter,
                ExportRegisterKind::Cycles,
            ],
            flags: vec![
                ExportFlagKind::Zero,
                ExportFlagKind::Sign,
                ExportFlagKind::Parity,
                ExportFlagKind::Carry,
                ExportFlagKind::AuxiliaryCarry,
            ],
            xlsx_pages: Vec::new(),
            text_sections: Vec::new(),
        }
    }
}

impl ExportXlsxPage {
    pub fn to_options(&self) -> ExportOptions {
        ExportOptions {
            page_name: self.name.clone(),
            memory_start: self.memory_start,
            memory_end: self.memory_end,
            include_memory_address: self.include_memory_address,
            include_memory_value: self.include_memory_value,
            include_memory_command: self.include_memory_command,
            include_comment_column: self.include_comment_column,
            registers: self.registers.clone(),
            flags: self.flags.clone(),
            xlsx_pages: Vec::new(),
            text_sections: Vec::new(),
        }
    }
}

impl ExportTextSection {
    pub fn to_options(&self) -> ExportOptions {
        ExportOptions {
            page_name: self.name.clone(),
            memory_start: self.memory_start,
            memory_end: self.memory_end,
            include_memory_address: self.include_memory_address,
            include_memory_value: self.include_memory_value,
            include_memory_command: self.include_memory_command,
            include_comment_column: false,
            registers: self.registers.clone(),
            flags: self.flags.clone(),
            xlsx_pages: Vec::new(),
            text_sections: Vec::new(),
        }
    }
}

impl ExportModel {
    /// Memory is emitted sparsely – only non-zero cells. Importing
    /// zero-fills any address not in the export, so the round-trip
    /// stays exact.
    pub fn from_cpu(state: &Cpu8080State) -> Self {
        Self::from_cpu_with_options(state, &ExportOptions::default())
    }

    pub fn from_cpu_with_options(state: &Cpu8080State, options: &ExportOptions) -> Self {
        let registers = options
            .registers
            .iter()
            .map(|register| register_pair(state, *register))
            .collect();
        let flags = options
            .flags
            .iter()
            .map(|flag| flag_pair(state, *flag))
            .collect();
        let (start, end) = ordered_range(options.memory_start, options.memory_end);
        let memory = (start..=end)
            .filter_map(|address| {
                let value = state.memory.read(address);
                (value != 0).then_some((address, value))
            })
            .collect();
        Self {
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

    pub fn write_txt_sections(
        path: impl AsRef<Path>,
        sections: &[(String, ExportModel)],
    ) -> Result<(), ExportError> {
        std::fs::write(path, Self::to_text_sections(sections))?;
        Ok(())
    }

    /// Three sections (`[Registers]`, `[Flags]`, `[Memory]`) separated
    /// by blank lines.
    pub fn to_text(model: &ExportModel) -> String {
        let mut out = String::new();
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

    pub fn to_text_sections(sections: &[(String, ExportModel)]) -> String {
        let mut out = String::new();
        for (index, (name, model)) in sections.iter().enumerate() {
            if index > 0 {
                out.push('\n');
            }
            let name = text_section_name(name, index);
            out.push_str(&format!("[{name}]\n"));
            out.push_str(&Self::to_text(model));
        }
        out
    }
}

fn register_pair(state: &Cpu8080State, register: ExportRegisterKind) -> (String, String) {
    match register {
        ExportRegisterKind::Accumulator => ("A".to_owned(), hex8(state.registers.a)),
        ExportRegisterKind::W => ("W".to_owned(), hex8(state.registers.w)),
        ExportRegisterKind::Z => ("Z".to_owned(), hex8(state.registers.z)),
        ExportRegisterKind::B => ("B".to_owned(), hex8(state.registers.b)),
        ExportRegisterKind::C => ("C".to_owned(), hex8(state.registers.c)),
        ExportRegisterKind::D => ("D".to_owned(), hex8(state.registers.d)),
        ExportRegisterKind::E => ("E".to_owned(), hex8(state.registers.e)),
        ExportRegisterKind::H => ("H".to_owned(), hex8(state.registers.h)),
        ExportRegisterKind::L => ("L".to_owned(), hex8(state.registers.l)),
        ExportRegisterKind::StackPointer => ("SP".to_owned(), hex16(state.sp)),
        ExportRegisterKind::ProgramCounter => ("PC".to_owned(), hex16(state.pc)),
        ExportRegisterKind::Cycles => ("cycles".to_owned(), state.cycle_count.to_string()),
    }
}

fn flag_pair(state: &Cpu8080State, flag: ExportFlagKind) -> (String, bool) {
    match flag {
        ExportFlagKind::Sign => ("S".to_owned(), state.flags.sign),
        ExportFlagKind::Zero => ("Z".to_owned(), state.flags.zero),
        ExportFlagKind::AuxiliaryCarry => ("AC".to_owned(), state.flags.auxiliary_carry),
        ExportFlagKind::Parity => ("P".to_owned(), state.flags.parity),
        ExportFlagKind::Carry => ("C".to_owned(), state.flags.carry),
    }
}

fn ordered_range(a: u16, b: u16) -> (u16, u16) {
    if a <= b { (a, b) } else { (b, a) }
}

fn command_for(value: u8) -> String {
    decode_opcode(value)
        .map(|info| info.mnemonic)
        .unwrap_or_else(|_| "-".to_owned())
}

fn text_section_name(name: &str, index: usize) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        format!("Section {}", index + 1)
    } else {
        trimmed.to_owned()
    }
}

pub(crate) fn hex8(value: u8) -> String {
    format!("{value:02X}")
}

pub(crate) fn hex16(value: u16) -> String {
    format!("{value:04X}")
}
