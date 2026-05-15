//! Opcode decoding tables.
//!
//! The 8080 opcodes are mostly regular: the bit patterns themselves encode
//! register / pair / condition operands. We expose small helper accessors used
//! by `execute.rs`.
//!
//! The list of *undocumented* 8080 opcode slots that must raise `DecodeError`
//! per `prompt/opcode_dispatch.md`:
//!
//! ```text
//! 08 10 18 20 28 30 38 CB D9 DD ED FD
//! ```

use crate::error::DecodeError;
use crate::state::{Reg8, RegPair};
use crate::timing::InstructionTiming;

/// 8080 condition codes selected by bits 5-3 in conditional opcodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cond {
    /// NZ — branch when zero flag clear.
    NZ,
    /// Z — branch when zero flag set.
    Z,
    /// NC — branch when carry flag clear.
    NC,
    /// C — branch when carry flag set.
    C,
    /// PO — parity odd (P flag clear).
    PO,
    /// PE — parity even (P flag set).
    PE,
    /// P — sign positive (S flag clear).
    P,
    /// M — minus (S flag set).
    M,
}

impl Cond {
    /// Decode a 3-bit condition code (bits 5-3 of the opcode).
    #[must_use]
    pub const fn from_ccc(ccc: u8) -> Self {
        match ccc & 0b111 {
            0b000 => Cond::NZ,
            0b001 => Cond::Z,
            0b010 => Cond::NC,
            0b011 => Cond::C,
            0b100 => Cond::PO,
            0b101 => Cond::PE,
            0b110 => Cond::P,
            _ => Cond::M, // 0b111
        }
    }
}

/// Decode the 3-bit register code into either a 8-bit register or memory `M`.
///
/// Returns `None` for `M` (encoding `110`).
#[inline]
#[must_use]
pub const fn decode_r(rrr: u8) -> Option<Reg8> {
    match rrr & 0b111 {
        0b000 => Some(Reg8::B),
        0b001 => Some(Reg8::C),
        0b010 => Some(Reg8::D),
        0b011 => Some(Reg8::E),
        0b100 => Some(Reg8::H),
        0b101 => Some(Reg8::L),
        0b110 => None, // M
        _ => Some(Reg8::A),
    }
}

/// Decode the 2-bit register-pair code in pp form (`BC`, `DE`, `HL`, `SP`).
#[inline]
#[must_use]
pub const fn decode_rp_sp(pp: u8) -> RegPair {
    match pp & 0b11 {
        0b00 => RegPair::Bc,
        0b01 => RegPair::De,
        0b10 => RegPair::Hl,
        _ => RegPair::Sp, // 0b11
    }
}

/// Decode the 2-bit register-pair code in `PUSH` / `POP` form
/// (`BC`, `DE`, `HL`, `PSW`). Returns `None` for the `PSW` slot.
#[inline]
#[must_use]
pub const fn decode_rp_psw(pp: u8) -> Option<RegPair> {
    match pp & 0b11 {
        0b00 => Some(RegPair::Bc),
        0b01 => Some(RegPair::De),
        0b10 => Some(RegPair::Hl),
        _ => None, // PSW
    }
}

/// Returns true if this opcode value is one of the undocumented 8080 slots.
#[inline]
#[must_use]
pub const fn is_undocumented(op: u8) -> bool {
    matches!(
        op,
        0x08 | 0x10 | 0x18 | 0x20 | 0x28 | 0x30 | 0x38 | 0xCB | 0xD9 | 0xDD | 0xED | 0xFD
    )
}

/// Pre-decode pass: validate the byte at the program counter.
///
/// Used by debug tooling and by the executor's pre-fetch step.
pub fn validate_opcode(op: u8) -> Result<(), DecodeError> {
    if is_undocumented(op) {
        Err(DecodeError::UndocumentedOpcode(op))
    } else {
        Ok(())
    }
}

/// Static base timing table indexed by opcode (T-states, taken / not-taken).
///
/// The table reflects the standard 8080 timing reported in the Intel data
/// book and copied widely in 8080 references. Conditional branches store
/// distinct taken / not-taken values per `prompt/01_architecture.md`.
///
/// (See `docs/cpu/timing.md` for the full table reasoning.)
pub fn opcode_timing(op: u8) -> InstructionTiming {
    use InstructionTiming as T;
    match op {
        // NOP / undocumented slots also report 4; the executor enforces decode
        // errors before timing is consumed.
        0x00 => T::fixed(4),

        // LXI rp,d16 — 10
        0x01 | 0x11 | 0x21 | 0x31 => T::fixed(10),
        // STAX / LDAX — 7
        0x02 | 0x12 | 0x0A | 0x1A => T::fixed(7),
        // INX / DCX — 5
        0x03 | 0x13 | 0x23 | 0x33 | 0x0B | 0x1B | 0x2B | 0x3B => T::fixed(5),
        // INR/DCR r — 5; INR/DCR M — 10
        0x34 | 0x35 => T::fixed(10),
        0x04 | 0x0C | 0x14 | 0x1C | 0x24 | 0x2C | 0x3C | 0x05 | 0x0D | 0x15 | 0x1D | 0x25
        | 0x2D | 0x3D => T::fixed(5),
        // MVI r,d8 — 7; MVI M,d8 — 10
        0x36 => T::fixed(10),
        0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x3E => T::fixed(7),

        // RLC / RRC / RAL / RAR / DAA / CMA / STC / CMC — 4
        0x07 | 0x0F | 0x17 | 0x1F | 0x27 | 0x2F | 0x37 | 0x3F => T::fixed(4),

        // DAD rp — 10
        0x09 | 0x19 | 0x29 | 0x39 => T::fixed(10),

        // SHLD / LHLD — 16
        0x22 | 0x2A => T::fixed(16),
        // STA / LDA — 13
        0x32 | 0x3A => T::fixed(13),

        // MOV r,M / MOV M,r — 7;   MOV r,r' — 5
        0x76 => T::fixed(7), // HLT — 7
        0x40..=0x7F => {
            // dst rrr at bits 5-3, src rrr at bits 2-0
            let dst = (op >> 3) & 0b111;
            let src = op & 0b111;
            if dst == 0b110 || src == 0b110 {
                T::fixed(7)
            } else {
                T::fixed(5)
            }
        }

        // ADD/ADC/SUB/SBB/ANA/XRA/ORA/CMP r — 4; M variant — 7
        0x80..=0xBF => {
            let src = op & 0b111;
            if src == 0b110 {
                T::fixed(7)
            } else {
                T::fixed(4)
            }
        }

        // Conditional return: 11 taken / 5 not taken
        0xC0 | 0xC8 | 0xD0 | 0xD8 | 0xE0 | 0xE8 | 0xF0 | 0xF8 => T::cond(11, 5),
        // Unconditional RET — 10
        0xC9 => T::fixed(10),
        // Conditional jump — 10 either way (still use cond for explicitness)
        0xC2 | 0xCA | 0xD2 | 0xDA | 0xE2 | 0xEA | 0xF2 | 0xFA => T::cond(10, 10),
        // Unconditional jump — 10
        0xC3 => T::fixed(10),
        // Conditional call — 17 taken / 11 not taken
        0xC4 | 0xCC | 0xD4 | 0xDC | 0xE4 | 0xEC | 0xF4 | 0xFC => T::cond(17, 11),
        // Unconditional CALL — 17
        0xCD => T::fixed(17),
        // PUSH/POP — 11/10
        0xC5 | 0xD5 | 0xE5 | 0xF5 => T::fixed(11),
        0xC1 | 0xD1 | 0xE1 | 0xF1 => T::fixed(10),
        // RST — 11
        0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF => T::fixed(11),
        // Immediate ALU — 7
        0xC6 | 0xCE | 0xD6 | 0xDE | 0xE6 | 0xEE | 0xF6 | 0xFE => T::fixed(7),

        // IN / OUT — 10
        0xDB | 0xD3 => T::fixed(10),
        // XTHL — 18, XCHG — 5, SPHL — 5, PCHL — 5
        0xE3 => T::fixed(18),
        0xEB => T::fixed(5),
        0xE9 => T::fixed(5),
        0xF9 => T::fixed(5),
        // EI / DI — 4
        0xF3 | 0xFB => T::fixed(4),

        // Anything else (the undocumented slots) falls to a placeholder.
        // The executor will refuse to run them via `validate_opcode`.
        _ => T::fixed(4),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undocumented_slots_listed() {
        for op in [
            0x08u8, 0x10, 0x18, 0x20, 0x28, 0x30, 0x38, 0xCB, 0xD9, 0xDD, 0xED, 0xFD,
        ] {
            assert_eq!(
                validate_opcode(op),
                Err(DecodeError::UndocumentedOpcode(op))
            );
        }
    }

    #[test]
    fn nop_is_documented() {
        assert!(validate_opcode(0x00).is_ok());
    }

    #[test]
    fn cond_decoding() {
        assert_eq!(Cond::from_ccc(0), Cond::NZ);
        assert_eq!(Cond::from_ccc(1), Cond::Z);
        assert_eq!(Cond::from_ccc(2), Cond::NC);
        assert_eq!(Cond::from_ccc(3), Cond::C);
        assert_eq!(Cond::from_ccc(4), Cond::PO);
        assert_eq!(Cond::from_ccc(5), Cond::PE);
        assert_eq!(Cond::from_ccc(6), Cond::P);
        assert_eq!(Cond::from_ccc(7), Cond::M);
    }
}
