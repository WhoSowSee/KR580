//! Misc opcodes: DAA, CMA, STC, CMC, HLT, EI, DI, IN, OUT.

use crate::flags::parity_even;
use crate::io::IoBus;
use crate::state::Cpu8080State;

/// `DAA` — standard 8080 BCD adjustment after addition.
///
/// Per `prompt/07_cpu_opcode_semantics.md`, the result of `DAA` after
/// subtraction or compare is undefined and is *not* a compatibility target.
/// We implement only the documented post-addition behavior.
pub fn daa(cpu: &mut Cpu8080State) {
    let mut adjust: u8 = 0;
    let mut new_cy = cpu.flags.cy;

    let low = cpu.a & 0x0F;
    if low > 9 || cpu.flags.ac {
        adjust |= 0x06;
    }
    let high = cpu.a >> 4;
    if cpu.flags.cy || high > 9 || (high >= 9 && low > 9) {
        adjust |= 0x60;
        new_cy = true;
    }

    let new_ac = (cpu.a & 0x0F) + (adjust & 0x0F) > 0x0F;
    let r = cpu.a.wrapping_add(adjust);
    cpu.a = r;
    cpu.flags.cy = new_cy;
    cpu.flags.ac = new_ac;
    cpu.flags.s = (r & 0x80) != 0;
    cpu.flags.z = r == 0;
    cpu.flags.p = parity_even(r);
}

/// `CMA` — complement A. Flags are *not* affected.
pub fn cma(cpu: &mut Cpu8080State) {
    cpu.a = !cpu.a;
}

/// `STC` — set CY = 1. No other flags change.
pub fn stc(cpu: &mut Cpu8080State) {
    cpu.flags.cy = true;
}

/// `CMC` — invert CY. No other flags change.
pub fn cmc(cpu: &mut Cpu8080State) {
    cpu.flags.cy = !cpu.flags.cy;
}

/// `HLT` — set halt state.
pub fn hlt(cpu: &mut Cpu8080State) {
    cpu.halted = true;
}

/// `EI` — arm pending enable. Becomes live after the next instruction
/// boundary. `prompt/02_cpu_core.md`.
pub fn ei(cpu: &mut Cpu8080State) {
    cpu.interrupt_enable_pending = true;
}

/// `DI` — clear both live and pending enable.
pub fn di(cpu: &mut Cpu8080State) {
    cpu.interrupt_enable = false;
    cpu.interrupt_enable_pending = false;
}

/// `IN d8` — read 8-bit port into A.
pub fn input<B: IoBus>(cpu: &mut Cpu8080State, bus: &mut B) {
    let port = cpu.fetch_imm8();
    cpu.a = bus.read(port);
}

/// `OUT d8` — write A to 8-bit port.
pub fn output<B: IoBus>(cpu: &mut Cpu8080State, bus: &mut B) {
    let port = cpu.fetch_imm8();
    bus.write(port, cpu.a);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::{NullIoBus, RecordingIoBus};

    #[test]
    fn daa_basic_addition_adjusts_low_nibble() {
        let mut c = Cpu8080State::new();
        c.a = 0x9B;
        c.flags.ac = false;
        c.flags.cy = false;
        c.ram.write(0, 0x27);
        let mut bus = NullIoBus;
        c.step_instruction(&mut bus).unwrap();
        // 9B → +60 (high nibble adj) → +06 (low nibble adj) = 0x101 → A=0x01, CY=1
        assert_eq!(c.a, 0x01);
        assert!(c.flags.cy);
    }

    #[test]
    fn cma_does_not_change_flags() {
        let mut c = Cpu8080State::new();
        c.a = 0x55;
        c.flags = crate::flags::Flags {
            s: true,
            z: true,
            ac: true,
            p: true,
            cy: true,
        };
        c.ram.write(0, 0x2F);
        let mut bus = NullIoBus;
        c.step_instruction(&mut bus).unwrap();
        assert_eq!(c.a, 0xAA);
        assert!(c.flags.s);
        assert!(c.flags.z);
        assert!(c.flags.ac);
        assert!(c.flags.p);
        assert!(c.flags.cy);
    }

    #[test]
    fn stc_only_touches_cy() {
        let mut c = Cpu8080State::new();
        c.flags = crate::flags::Flags {
            s: true,
            z: true,
            ac: false,
            p: true,
            cy: false,
        };
        c.ram.write(0, 0x37);
        let mut bus = NullIoBus;
        c.step_instruction(&mut bus).unwrap();
        assert!(c.flags.cy);
        assert!(c.flags.s);
        assert!(c.flags.z);
        assert!(!c.flags.ac);
        assert!(c.flags.p);
    }

    #[test]
    fn out_writes_through_bus() {
        let mut c = Cpu8080State::new();
        c.a = 0x42;
        c.ram.write(0, 0xD3); // OUT 04
        c.ram.write(1, 0x04);
        let mut bus = RecordingIoBus::default();
        c.step_instruction(&mut bus).unwrap();
        assert_eq!(bus.out_log, vec![(0x04u8, 0x42u8)]);
    }

    #[test]
    fn in_reads_from_bus() {
        let mut c = Cpu8080State::new();
        c.ram.write(0, 0xDB); // IN 03
        c.ram.write(1, 0x03);
        let mut bus = RecordingIoBus::default();
        bus.in_queue[0x03].push(0x99);
        c.step_instruction(&mut bus).unwrap();
        assert_eq!(c.a, 0x99);
    }
}
