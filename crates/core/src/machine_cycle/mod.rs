//! Machine-cycle / T-phase layout per opcode for the schematic readout.
//! Numbers come from the Intel 8080A Datasheet ("STATES" column and the
//! "Machine Cycle" section). Conditional opcodes carry both taken and
//! not-taken sequences; undocumented opcodes return an empty layout.

mod tables;
#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) use tables::kinds_for;
pub use tables::{kind_at, layout_for};

pub type MachineCycleLengths = &'static [u8];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MachineCycleLayout {
    pub taken: MachineCycleLengths,
    pub not_taken: Option<MachineCycleLengths>,
}

impl MachineCycleLayout {
    pub const fn fixed(cycles: MachineCycleLengths) -> Self {
        Self {
            taken: cycles,
            not_taken: None,
        }
    }

    pub(crate) const fn branch(taken: MachineCycleLengths, not_taken: MachineCycleLengths) -> Self {
        Self {
            taken,
            not_taken: Some(not_taken),
        }
    }

    pub fn total_t_states(self, branch_taken: bool) -> u8 {
        let cycles = if branch_taken {
            self.taken
        } else {
            self.not_taken.unwrap_or(self.taken)
        };
        let mut sum = 0u8;
        let mut i = 0;
        while i < cycles.len() {
            sum += cycles[i];
            i += 1;
        }
        sum
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MachineCyclePosition {
    pub m_cycle: u8,
    pub t_in_cycle: u8,
    pub m_cycle_length: u8,
}

pub fn position_for(
    layout: MachineCycleLayout,
    branch_taken: bool,
    linear_phase: u8,
) -> Option<MachineCyclePosition> {
    let cycles = if branch_taken {
        layout.taken
    } else {
        layout.not_taken.unwrap_or(layout.taken)
    };
    if cycles.is_empty() {
        return None;
    }
    let mut consumed = 0u8;
    for (idx, &length) in cycles.iter().enumerate() {
        if linear_phase < consumed + length {
            return Some(MachineCyclePosition {
                m_cycle: (idx as u8) + 1,
                t_in_cycle: linear_phase - consumed + 1,
                m_cycle_length: length,
            });
        }
        consumed += length;
    }
    None
}

/// 8080 M-cycle kind. `status_byte()` returns the byte the chip latches on
/// T1 of each M-cycle (Intel 8080A datasheet, "Status Information").
/// Bits: D7 MEMR, D6 INP, D5 M1, D4 OUT, D3 HLTA, D2 STACK, D1 WO, D0 INTA.
/// `WO` is inverted relative to read/write (1 = read/input).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MachineCycleKind {
    M1Fetch,
    MemoryRead,
    MemoryWrite,
    StackRead,
    StackWrite,
    IoRead,
    IoWrite,
    InterruptAck,
    HaltAck,
    /// Internal idle cycle (DAD, INX/DCX): bus is not driven.
    BusIdle,
}

impl MachineCycleKind {
    pub fn status_byte(self) -> u8 {
        match self {
            Self::M1Fetch => 0b1010_0010,
            Self::MemoryRead => 0b1000_0010,
            Self::MemoryWrite => 0b0000_0000,
            Self::StackRead => 0b1000_0110,
            Self::StackWrite => 0b0000_0100,
            Self::IoRead => 0b0100_0010,
            Self::IoWrite => 0b0001_0000,
            Self::InterruptAck => 0b0010_0011,
            Self::HaltAck => 0b1000_1010,
            Self::BusIdle => 0,
        }
    }

    pub fn label_ru(self) -> &'static str {
        match self {
            Self::M1Fetch => "Загрузка опкода",
            Self::MemoryRead => "Чтение памяти",
            Self::MemoryWrite => "Запись в память",
            Self::StackRead => "Чтение из стека",
            Self::StackWrite => "Запись в стек",
            Self::IoRead => "Чтение из порта",
            Self::IoWrite => "Запись в порт",
            Self::InterruptAck => "Подтв. прерывания",
            Self::HaltAck => "Подтв. останова",
            Self::BusIdle => "Внутренний цикл",
        }
    }

    pub fn label_en(self) -> &'static str {
        match self {
            Self::M1Fetch => "Opcode fetch",
            Self::MemoryRead => "Memory read",
            Self::MemoryWrite => "Memory write",
            Self::StackRead => "Stack read",
            Self::StackWrite => "Stack write",
            Self::IoRead => "Port read",
            Self::IoWrite => "Port write",
            Self::InterruptAck => "Interrupt ack",
            Self::HaltAck => "Halt ack",
            Self::BusIdle => "Internal cycle",
        }
    }
}

pub type MachineCycleKinds = &'static [MachineCycleKind];
