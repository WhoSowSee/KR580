use super::{ExportFlag, ExportMemoryColumn, ExportRegister, ExportTab};
use crate::persistence::{ExportFlagKind, ExportRegisterKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExportModalFocus {
    None,
    TabXlsx,
    TabText,
    Page,
    TargetDropdown,
    TargetAdd,
    TargetDelete,
    MemoryStart,
    MemoryEnd,
    ColumnAddress,
    ColumnValue,
    ColumnCommand,
    ColumnComment,
    RegisterAccumulator,
    RegisterW,
    RegisterZ,
    RegisterB,
    RegisterC,
    RegisterD,
    RegisterE,
    RegisterH,
    RegisterL,
    RegisterStackPointer,
    RegisterProgramCounter,
    RegisterCycles,
    FlagSign,
    FlagZero,
    FlagAuxiliaryCarry,
    FlagParity,
    FlagCarry,
    Cancel,
    Confirm,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ExportMemoryColumns {
    pub(crate) address: bool,
    pub(crate) value: bool,
    pub(crate) command: bool,
    pub(crate) comment: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ExportRegisterSelection {
    pub(crate) accumulator: bool,
    pub(crate) w: bool,
    pub(crate) z: bool,
    pub(crate) b: bool,
    pub(crate) c: bool,
    pub(crate) d: bool,
    pub(crate) e: bool,
    pub(crate) h: bool,
    pub(crate) l: bool,
    pub(crate) stack_pointer: bool,
    pub(crate) program_counter: bool,
    pub(crate) cycles: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ExportFlagSelection {
    pub(crate) sign: bool,
    pub(crate) zero: bool,
    pub(crate) auxiliary_carry: bool,
    pub(crate) parity: bool,
    pub(crate) carry: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ExportTargetSettings {
    pub(crate) memory_start_input: String,
    pub(crate) memory_end_input: String,
    pub(crate) columns: ExportMemoryColumns,
    pub(crate) registers: ExportRegisterSelection,
    pub(crate) flags: ExportFlagSelection,
}

impl Default for ExportMemoryColumns {
    fn default() -> Self {
        Self {
            address: true,
            value: true,
            command: true,
            comment: false,
        }
    }
}

impl Default for ExportTargetSettings {
    fn default() -> Self {
        Self {
            memory_start_input: "0000".to_owned(),
            memory_end_input: "FFFF".to_owned(),
            columns: ExportMemoryColumns::default(),
            registers: ExportRegisterSelection::default(),
            flags: ExportFlagSelection::default(),
        }
    }
}

impl ExportMemoryColumns {
    pub(crate) fn toggle(&mut self, column: ExportMemoryColumn) {
        match column {
            ExportMemoryColumn::Address => self.address = !self.address,
            ExportMemoryColumn::Value => self.value = !self.value,
            ExportMemoryColumn::Command => self.command = !self.command,
            ExportMemoryColumn::Comment => self.comment = !self.comment,
        }
    }
}

impl ExportRegisterSelection {
    pub(crate) fn toggle(&mut self, register: ExportRegister) {
        match register {
            ExportRegister::Accumulator => self.accumulator = !self.accumulator,
            ExportRegister::W => self.w = !self.w,
            ExportRegister::Z => self.z = !self.z,
            ExportRegister::B => self.b = !self.b,
            ExportRegister::C => self.c = !self.c,
            ExportRegister::D => self.d = !self.d,
            ExportRegister::E => self.e = !self.e,
            ExportRegister::H => self.h = !self.h,
            ExportRegister::L => self.l = !self.l,
            ExportRegister::StackPointer => self.stack_pointer = !self.stack_pointer,
            ExportRegister::ProgramCounter => self.program_counter = !self.program_counter,
            ExportRegister::Cycles => self.cycles = !self.cycles,
        }
    }

    pub(crate) fn selected(self) -> Vec<ExportRegisterKind> {
        let mut out = Vec::new();
        push_if(&mut out, self.accumulator, ExportRegisterKind::Accumulator);
        push_if(&mut out, self.w, ExportRegisterKind::W);
        push_if(&mut out, self.z, ExportRegisterKind::Z);
        push_if(&mut out, self.b, ExportRegisterKind::B);
        push_if(&mut out, self.c, ExportRegisterKind::C);
        push_if(&mut out, self.d, ExportRegisterKind::D);
        push_if(&mut out, self.e, ExportRegisterKind::E);
        push_if(&mut out, self.h, ExportRegisterKind::H);
        push_if(&mut out, self.l, ExportRegisterKind::L);
        push_if(
            &mut out,
            self.stack_pointer,
            ExportRegisterKind::StackPointer,
        );
        push_if(
            &mut out,
            self.program_counter,
            ExportRegisterKind::ProgramCounter,
        );
        push_if(&mut out, self.cycles, ExportRegisterKind::Cycles);
        out
    }
}

impl ExportFlagSelection {
    pub(crate) fn toggle(&mut self, flag: ExportFlag) {
        match flag {
            ExportFlag::Sign => self.sign = !self.sign,
            ExportFlag::Zero => self.zero = !self.zero,
            ExportFlag::AuxiliaryCarry => self.auxiliary_carry = !self.auxiliary_carry,
            ExportFlag::Parity => self.parity = !self.parity,
            ExportFlag::Carry => self.carry = !self.carry,
        }
    }

    pub(crate) fn selected(self) -> Vec<ExportFlagKind> {
        let mut out = Vec::new();
        push_flag_if(&mut out, self.zero, ExportFlagKind::Zero);
        push_flag_if(&mut out, self.sign, ExportFlagKind::Sign);
        push_flag_if(&mut out, self.parity, ExportFlagKind::Parity);
        push_flag_if(&mut out, self.carry, ExportFlagKind::Carry);
        push_flag_if(
            &mut out,
            self.auxiliary_carry,
            ExportFlagKind::AuxiliaryCarry,
        );
        out
    }
}

impl ExportModalFocus {
    const ORDER: [Self; 32] = [
        Self::None,
        Self::TabXlsx,
        Self::TabText,
        Self::Page,
        Self::TargetDropdown,
        Self::TargetAdd,
        Self::TargetDelete,
        Self::MemoryStart,
        Self::MemoryEnd,
        Self::ColumnAddress,
        Self::ColumnValue,
        Self::ColumnCommand,
        Self::ColumnComment,
        Self::RegisterAccumulator,
        Self::RegisterW,
        Self::RegisterZ,
        Self::RegisterB,
        Self::RegisterC,
        Self::RegisterD,
        Self::RegisterE,
        Self::RegisterH,
        Self::RegisterL,
        Self::RegisterStackPointer,
        Self::RegisterProgramCounter,
        Self::RegisterCycles,
        Self::FlagZero,
        Self::FlagSign,
        Self::FlagParity,
        Self::FlagCarry,
        Self::FlagAuxiliaryCarry,
        Self::Cancel,
        Self::Confirm,
    ];

    pub(crate) fn next(self) -> Self {
        let index = Self::ORDER
            .iter()
            .position(|focus| *focus == self)
            .unwrap_or(0);
        Self::ORDER[(index + 1) % Self::ORDER.len()]
    }

    pub(crate) fn previous(self) -> Self {
        let index = Self::ORDER
            .iter()
            .position(|focus| *focus == self)
            .unwrap_or(0);
        Self::ORDER[(index + Self::ORDER.len() - 1) % Self::ORDER.len()]
    }

    pub(crate) fn next_for_tab(self, tab: ExportTab) -> Self {
        let mut focus = self.next();
        while !focus.is_visible_for_tab(tab) {
            focus = focus.next();
        }
        focus
    }

    pub(crate) fn previous_for_tab(self, tab: ExportTab) -> Self {
        let mut focus = self.previous();
        while !focus.is_visible_for_tab(tab) {
            focus = focus.previous();
        }
        focus
    }

    pub(crate) fn is_visible_for_tab(self, tab: ExportTab) -> bool {
        !matches!(
            (tab, self),
            (_, Self::None) | (ExportTab::Text, Self::ColumnComment)
        )
    }

    pub(crate) fn tab(self) -> Option<ExportTab> {
        match self {
            Self::TabXlsx => Some(ExportTab::Xlsx),
            Self::TabText => Some(ExportTab::Text),
            _ => None,
        }
    }

    pub(crate) fn memory_column(self) -> Option<ExportMemoryColumn> {
        match self {
            Self::ColumnAddress => Some(ExportMemoryColumn::Address),
            Self::ColumnValue => Some(ExportMemoryColumn::Value),
            Self::ColumnCommand => Some(ExportMemoryColumn::Command),
            Self::ColumnComment => Some(ExportMemoryColumn::Comment),
            _ => None,
        }
    }

    pub(crate) fn register(self) -> Option<ExportRegister> {
        match self {
            Self::RegisterAccumulator => Some(ExportRegister::Accumulator),
            Self::RegisterW => Some(ExportRegister::W),
            Self::RegisterZ => Some(ExportRegister::Z),
            Self::RegisterB => Some(ExportRegister::B),
            Self::RegisterC => Some(ExportRegister::C),
            Self::RegisterD => Some(ExportRegister::D),
            Self::RegisterE => Some(ExportRegister::E),
            Self::RegisterH => Some(ExportRegister::H),
            Self::RegisterL => Some(ExportRegister::L),
            Self::RegisterStackPointer => Some(ExportRegister::StackPointer),
            Self::RegisterProgramCounter => Some(ExportRegister::ProgramCounter),
            Self::RegisterCycles => Some(ExportRegister::Cycles),
            _ => None,
        }
    }

    pub(crate) fn flag(self) -> Option<ExportFlag> {
        match self {
            Self::FlagSign => Some(ExportFlag::Sign),
            Self::FlagZero => Some(ExportFlag::Zero),
            Self::FlagAuxiliaryCarry => Some(ExportFlag::AuxiliaryCarry),
            Self::FlagParity => Some(ExportFlag::Parity),
            Self::FlagCarry => Some(ExportFlag::Carry),
            _ => None,
        }
    }

    pub(crate) fn for_column(column: ExportMemoryColumn) -> Self {
        match column {
            ExportMemoryColumn::Address => Self::ColumnAddress,
            ExportMemoryColumn::Value => Self::ColumnValue,
            ExportMemoryColumn::Command => Self::ColumnCommand,
            ExportMemoryColumn::Comment => Self::ColumnComment,
        }
    }

    pub(crate) fn for_register(register: ExportRegister) -> Self {
        match register {
            ExportRegister::Accumulator => Self::RegisterAccumulator,
            ExportRegister::W => Self::RegisterW,
            ExportRegister::Z => Self::RegisterZ,
            ExportRegister::B => Self::RegisterB,
            ExportRegister::C => Self::RegisterC,
            ExportRegister::D => Self::RegisterD,
            ExportRegister::E => Self::RegisterE,
            ExportRegister::H => Self::RegisterH,
            ExportRegister::L => Self::RegisterL,
            ExportRegister::StackPointer => Self::RegisterStackPointer,
            ExportRegister::ProgramCounter => Self::RegisterProgramCounter,
            ExportRegister::Cycles => Self::RegisterCycles,
        }
    }

    pub(crate) fn for_flag(flag: ExportFlag) -> Self {
        match flag {
            ExportFlag::Sign => Self::FlagSign,
            ExportFlag::Zero => Self::FlagZero,
            ExportFlag::AuxiliaryCarry => Self::FlagAuxiliaryCarry,
            ExportFlag::Parity => Self::FlagParity,
            ExportFlag::Carry => Self::FlagCarry,
        }
    }

    pub(crate) fn after_target_action(dropdown_open: bool) -> Self {
        if dropdown_open {
            Self::TargetDropdown
        } else {
            Self::Page
        }
    }

    pub(crate) fn clears_on_escape(self) -> bool {
        matches!(
            self,
            Self::Page | Self::TargetDropdown | Self::MemoryStart | Self::MemoryEnd
        ) || self.memory_column().is_some()
            || self.register().is_some()
            || self.flag().is_some()
    }
}

fn push_if(out: &mut Vec<ExportRegisterKind>, enabled: bool, register: ExportRegisterKind) {
    if enabled {
        out.push(register);
    }
}

fn push_flag_if(out: &mut Vec<ExportFlagKind>, enabled: bool, flag: ExportFlagKind) {
    if enabled {
        out.push(flag);
    }
}

pub(super) fn hex4_input(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_hexdigit())
        .take(4)
        .map(|ch| ch.to_ascii_uppercase())
        .collect()
}

pub(super) fn parse_hex_u16_or(value: &str, fallback: u16) -> u16 {
    u16::from_str_radix(value.trim(), 16).unwrap_or(fallback)
}
