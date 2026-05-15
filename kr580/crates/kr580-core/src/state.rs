//! `Cpu8080State` plus register / register-pair enumerations.

use crate::flags::Flags;
use crate::memory::Memory64K;
use serde::{Deserialize, Serialize};

/// 8-bit registers addressable by the 3-bit `r` encoding.
///
/// The encoding `0b110` (`M`) addresses memory `[HL]`, but is handled at the
/// decoder level rather than as a `Reg8` variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Reg8 {
    /// Register B (encoding 000).
    B,
    /// Register C (encoding 001).
    C,
    /// Register D (encoding 010).
    D,
    /// Register E (encoding 011).
    E,
    /// Register H (encoding 100).
    H,
    /// Register L (encoding 101).
    L,
    /// Register A (encoding 111).
    A,
}

/// 16-bit register pairs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegPair {
    /// `BC`.
    Bc,
    /// `DE`.
    De,
    /// `HL`.
    Hl,
    /// `SP`.
    Sp,
}

/// All architectural state owned by the core. Per `prompt/01_architecture.md`,
/// no UI widgets ever own this data.
#[derive(Clone, Serialize, Deserialize)]
pub struct Cpu8080State {
    /// Accumulator.
    pub a: u8,
    /// Register B.
    pub b: u8,
    /// Register C.
    pub c: u8,
    /// Register D.
    pub d: u8,
    /// Register E.
    pub e: u8,
    /// Register H.
    pub h: u8,
    /// Register L.
    pub l: u8,
    /// Program counter.
    pub pc: u16,
    /// Stack pointer.
    pub sp: u16,
    /// PSW flags.
    pub flags: Flags,
    /// 64 KiB RAM.
    pub ram: Memory64K,

    /// Live interrupt enable.
    pub interrupt_enable: bool,
    /// Latch armed by `EI`. Becomes the live `interrupt_enable` after the
    /// next instruction boundary. Per `prompt/02_cpu_core.md`.
    pub interrupt_enable_pending: bool,
    /// External interrupt pending.
    pub interrupt_request_pending: bool,
    /// Single-byte interrupt vector supplied during interrupt acknowledge,
    /// typically a `RST n` opcode.
    pub interrupt_vector_byte: Option<u8>,

    /// `HLT` halted state.
    pub halted: bool,

    /// Total T-states executed since reset.
    pub cycle_count: u64,
    /// T-state index inside the active machine cycle. `None` between
    /// instructions.
    pub tact_phase: Option<u8>,
}

impl Default for Cpu8080State {
    fn default() -> Self {
        Self::new()
    }
}

impl Cpu8080State {
    /// Power-on reset state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            pc: 0,
            sp: 0,
            flags: Flags::default(),
            ram: Memory64K::new(),
            interrupt_enable: false,
            interrupt_enable_pending: false,
            interrupt_request_pending: false,
            interrupt_vector_byte: None,
            halted: false,
            cycle_count: 0,
            tact_phase: None,
        }
    }

    /// Reset CPU state per `prompt/09_quality_gates.md`. RAM is *not* touched.
    pub fn reset_cpu(&mut self) {
        let ram = std::mem::take(&mut self.ram);
        *self = Self::new();
        self.ram = ram;
    }

    /// Reset registers (A B C D E H L), flags, PC, SP, halt and tact phase.
    /// RAM and interrupt state are untouched (separate command).
    pub fn reset_registers(&mut self) {
        self.a = 0;
        self.b = 0;
        self.c = 0;
        self.d = 0;
        self.e = 0;
        self.h = 0;
        self.l = 0;
        self.pc = 0;
        self.sp = 0;
        self.flags = Flags::default();
        self.halted = false;
        self.tact_phase = None;
    }

    /// Read 8-bit register.
    #[inline]
    #[must_use]
    pub fn get_reg8(&self, r: Reg8) -> u8 {
        match r {
            Reg8::B => self.b,
            Reg8::C => self.c,
            Reg8::D => self.d,
            Reg8::E => self.e,
            Reg8::H => self.h,
            Reg8::L => self.l,
            Reg8::A => self.a,
        }
    }

    /// Write 8-bit register.
    #[inline]
    pub fn set_reg8(&mut self, r: Reg8, v: u8) {
        match r {
            Reg8::B => self.b = v,
            Reg8::C => self.c = v,
            Reg8::D => self.d = v,
            Reg8::E => self.e = v,
            Reg8::H => self.h = v,
            Reg8::L => self.l = v,
            Reg8::A => self.a = v,
        }
    }

    /// Read 16-bit register pair value.
    #[inline]
    #[must_use]
    pub fn get_pair(&self, rp: RegPair) -> u16 {
        match rp {
            RegPair::Bc => u16::from_be_bytes([self.b, self.c]),
            RegPair::De => u16::from_be_bytes([self.d, self.e]),
            RegPair::Hl => u16::from_be_bytes([self.h, self.l]),
            RegPair::Sp => self.sp,
        }
    }

    /// Write 16-bit register pair.
    #[inline]
    pub fn set_pair(&mut self, rp: RegPair, v: u16) {
        let [hi, lo] = v.to_be_bytes();
        match rp {
            RegPair::Bc => {
                self.b = hi;
                self.c = lo;
            }
            RegPair::De => {
                self.d = hi;
                self.e = lo;
            }
            RegPair::Hl => {
                self.h = hi;
                self.l = lo;
            }
            RegPair::Sp => self.sp = v,
        }
    }

    /// HL convenience accessor.
    #[inline]
    #[must_use]
    pub fn hl(&self) -> u16 {
        self.get_pair(RegPair::Hl)
    }

    /// Read memory at `[HL]`.
    #[inline]
    #[must_use]
    pub fn read_m(&self) -> u8 {
        self.ram.read(self.hl())
    }

    /// Write memory at `[HL]`.
    #[inline]
    pub fn write_m(&mut self, v: u8) {
        let addr = self.hl();
        self.ram.write(addr, v);
    }

    /// Push a 16-bit word on the stack (high byte at SP-1, low at SP-2).
    #[inline]
    pub fn push_word(&mut self, value: u16) {
        let [hi, lo] = value.to_be_bytes();
        self.sp = self.sp.wrapping_sub(1);
        self.ram.write(self.sp, hi);
        self.sp = self.sp.wrapping_sub(1);
        self.ram.write(self.sp, lo);
    }

    /// Pop a 16-bit word from the stack.
    #[inline]
    pub fn pop_word(&mut self) -> u16 {
        let lo = self.ram.read(self.sp);
        self.sp = self.sp.wrapping_add(1);
        let hi = self.ram.read(self.sp);
        self.sp = self.sp.wrapping_add(1);
        u16::from_be_bytes([hi, lo])
    }

    /// Fetch the next byte from `[PC]` and advance PC.
    #[inline]
    pub fn fetch_imm8(&mut self) -> u8 {
        let v = self.ram.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        v
    }

    /// Fetch a little-endian word from `[PC]` and advance PC by 2.
    #[inline]
    pub fn fetch_imm16(&mut self) -> u16 {
        let lo = self.fetch_imm8() as u16;
        let hi = self.fetch_imm8() as u16;
        (hi << 8) | lo
    }
}

impl std::fmt::Debug for Cpu8080State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cpu8080State")
            .field("A", &format_args!("{:02X}", self.a))
            .field("B", &format_args!("{:02X}", self.b))
            .field("C", &format_args!("{:02X}", self.c))
            .field("D", &format_args!("{:02X}", self.d))
            .field("E", &format_args!("{:02X}", self.e))
            .field("H", &format_args!("{:02X}", self.h))
            .field("L", &format_args!("{:02X}", self.l))
            .field("PC", &format_args!("{:04X}", self.pc))
            .field("SP", &format_args!("{:04X}", self.sp))
            .field("flags", &self.flags)
            .field("halted", &self.halted)
            .field("ie", &self.interrupt_enable)
            .field("ie_pending", &self.interrupt_enable_pending)
            .field("irq_pending", &self.interrupt_request_pending)
            .field("cycle_count", &self.cycle_count)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pair_accessors_are_big_endian() {
        let mut s = Cpu8080State::new();
        s.set_pair(RegPair::Bc, 0xBEEF);
        assert_eq!(s.b, 0xBE);
        assert_eq!(s.c, 0xEF);
        assert_eq!(s.get_pair(RegPair::Bc), 0xBEEF);
    }

    #[test]
    fn push_pop_word_roundtrip() {
        let mut s = Cpu8080State::new();
        s.sp = 0x1000;
        s.push_word(0x1234);
        assert_eq!(s.sp, 0x0FFE);
        assert_eq!(s.ram.read(0x0FFF), 0x12);
        assert_eq!(s.ram.read(0x0FFE), 0x34);
        assert_eq!(s.pop_word(), 0x1234);
        assert_eq!(s.sp, 0x1000);
    }

    #[test]
    fn reset_cpu_preserves_ram() {
        let mut s = Cpu8080State::new();
        s.ram.write(0x1234, 0xAB);
        s.a = 0xFF;
        s.pc = 0x4000;
        s.cycle_count = 999;
        s.reset_cpu();
        assert_eq!(s.a, 0);
        assert_eq!(s.pc, 0);
        assert_eq!(s.cycle_count, 0);
        assert_eq!(s.ram.read(0x1234), 0xAB, "RAM must be preserved");
    }
}
