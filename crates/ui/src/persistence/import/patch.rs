use super::{parse_u8_hex, parse_u16_hex, reg_err};
use crate::persistence::{ExportFlagKind, ExportModel, ImportError};
use k580_core::{Cpu8080State, RegisterName};
use std::borrow::Cow;

#[derive(Debug)]
pub(crate) struct CpuPatch<'a> {
    registers: Vec<RegisterChange>,
    flags: Vec<(ExportFlagKind, bool)>,
    memory: Cow<'a, [(u16, u8)]>,
}

#[derive(Debug)]
enum RegisterChange {
    Byte(RegisterName, u8),
    W(u8),
    Z(u8),
    Pc(u16),
    Sp(u16),
    Cycles(u64),
}

impl<'a> CpuPatch<'a> {
    pub(crate) fn parse(model: &'a ExportModel) -> Result<Self, ImportError> {
        let registers = model
            .registers
            .iter()
            .map(|(name, value)| RegisterChange::parse(name, value))
            .collect::<Result<_, _>>()?;
        let flags = model
            .flags
            .iter()
            .map(|(name, value)| {
                let kind = match name.as_str() {
                    "S" => ExportFlagKind::Sign,
                    "Z" => ExportFlagKind::Zero,
                    "AC" => ExportFlagKind::AuxiliaryCarry,
                    "P" => ExportFlagKind::Parity,
                    "C" | "CY" => ExportFlagKind::Carry,
                    _ => return Err(ImportError::Malformed(format!("unknown flag `{name}`"))),
                };
                Ok((kind, *value))
            })
            .collect::<Result<_, _>>()?;
        Ok(Self {
            registers,
            flags,
            memory: Cow::Borrowed(&model.memory),
        })
    }

    pub(crate) fn apply_to(self, state: &mut Cpu8080State) {
        for change in self.registers {
            match change {
                RegisterChange::Byte(name, value) => state.registers.set(name, value),
                RegisterChange::W(value) => state.registers.w = value,
                RegisterChange::Z(value) => state.registers.z = value,
                RegisterChange::Pc(value) => state.pc = value,
                RegisterChange::Sp(value) => state.sp = value,
                RegisterChange::Cycles(value) => state.cycle_count = value,
            }
        }
        for (kind, value) in self.flags {
            match kind {
                ExportFlagKind::Sign => state.flags.sign = value,
                ExportFlagKind::Zero => state.flags.zero = value,
                ExportFlagKind::AuxiliaryCarry => state.flags.auxiliary_carry = value,
                ExportFlagKind::Parity => state.flags.parity = value,
                ExportFlagKind::Carry => state.flags.carry = value,
            }
        }
        for &(address, value) in self.memory.iter() {
            state.memory.write(address, value);
        }
    }
}

impl CpuPatch<'static> {
    pub(crate) fn owned(model: ExportModel) -> Result<Self, ImportError> {
        let (registers, flags) = {
            let patch = CpuPatch::parse(&model)?;
            (patch.registers, patch.flags)
        };
        Ok(Self {
            registers,
            flags,
            memory: Cow::Owned(model.memory),
        })
    }
}

impl RegisterChange {
    fn parse(name: &str, value: &str) -> Result<Self, ImportError> {
        let byte = || parse_u8_hex(value).ok_or_else(|| reg_err(name, value));
        let word = || parse_u16_hex(value).ok_or_else(|| reg_err(name, value));
        let change = match name {
            "A" => Self::Byte(RegisterName::A, byte()?),
            "B" => Self::Byte(RegisterName::B, byte()?),
            "C" => Self::Byte(RegisterName::C, byte()?),
            "D" => Self::Byte(RegisterName::D, byte()?),
            "E" => Self::Byte(RegisterName::E, byte()?),
            "H" => Self::Byte(RegisterName::H, byte()?),
            "L" => Self::Byte(RegisterName::L, byte()?),
            "W" => Self::W(byte()?),
            "Z" => Self::Z(byte()?),
            "PC" => Self::Pc(word()?),
            "SP" => Self::Sp(word()?),
            "cycles" => Self::Cycles(value.parse().map_err(|_| reg_err(name, value))?),
            _ => return Err(ImportError::Malformed(format!("unknown register `{name}`"))),
        };
        Ok(change)
    }
}
