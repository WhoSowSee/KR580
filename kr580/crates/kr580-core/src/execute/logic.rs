//! Logical family: ANA, XRA, ORA, ANI, XRI, ORI.
//!
//! Per `prompt/07_cpu_opcode_semantics.md`:
//!
//! * `ANA/ANI` clear CY and set AC to `(bit3(A) | bit3(operand))` *before* the
//!   AND result is written.
//! * `ORA/XRA/ORI/XRI` clear both CY and AC.

use crate::decode::decode_r;
use crate::state::Cpu8080State;

#[inline]
fn read_src_reg_or_m(cpu: &Cpu8080State, op: u8) -> u8 {
    match decode_r(op & 0b111) {
        Some(r) => cpu.get_reg8(r),
        None => cpu.read_m(),
    }
}

/// `ANA r/M`
pub fn ana(cpu: &mut Cpu8080State, op: u8) {
    let v = read_src_reg_or_m(cpu, op);
    ana_inner(cpu, v);
}

/// `ANI d8`
pub fn ani(cpu: &mut Cpu8080State) {
    let v = cpu.fetch_imm8();
    ana_inner(cpu, v);
}

fn ana_inner(cpu: &mut Cpu8080State, v: u8) {
    let ac = ((cpu.a | v) & 0x08) != 0;
    let r = cpu.a & v;
    cpu.a = r;
    cpu.flags.cy = false;
    cpu.flags.ac = ac;
    cpu.flags.set_szp(r);
}

/// `XRA r/M`
pub fn xra(cpu: &mut Cpu8080State, op: u8) {
    let v = read_src_reg_or_m(cpu, op);
    xra_inner(cpu, v);
}

/// `XRI d8`
pub fn xri(cpu: &mut Cpu8080State) {
    let v = cpu.fetch_imm8();
    xra_inner(cpu, v);
}

fn xra_inner(cpu: &mut Cpu8080State, v: u8) {
    let r = cpu.a ^ v;
    cpu.a = r;
    cpu.flags.cy = false;
    cpu.flags.ac = false;
    cpu.flags.set_szp(r);
}

/// `ORA r/M`
pub fn ora(cpu: &mut Cpu8080State, op: u8) {
    let v = read_src_reg_or_m(cpu, op);
    ora_inner(cpu, v);
}

/// `ORI d8`
pub fn ori(cpu: &mut Cpu8080State) {
    let v = cpu.fetch_imm8();
    ora_inner(cpu, v);
}

fn ora_inner(cpu: &mut Cpu8080State, v: u8) {
    let r = cpu.a | v;
    cpu.a = r;
    cpu.flags.cy = false;
    cpu.flags.ac = false;
    cpu.flags.set_szp(r);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::NullIoBus;

    #[test]
    fn ana_clears_cy_and_sets_ac_from_bit3_or() {
        let mut c = Cpu8080State::new();
        c.a = 0b0000_1000;
        c.b = 0b0000_0000;
        c.flags.cy = true;
        c.ram.write(0, 0xA0); // ANA B
        let mut bus = NullIoBus;
        c.step_instruction(&mut bus).unwrap();
        assert_eq!(c.a, 0); // bit-3 only on accumulator → AND = 0
        assert!(!c.flags.cy, "ANA must clear CY");
        assert!(c.flags.ac, "ANA AC = bit3(A)|bit3(operand) before write");
        assert!(c.flags.z);
    }

    #[test]
    fn ana_ac_is_zero_when_neither_bit3_set() {
        let mut c = Cpu8080State::new();
        c.a = 0b1111_0000;
        c.b = 0b1111_0000;
        c.flags.cy = true;
        c.ram.write(0, 0xA0);
        let mut bus = NullIoBus;
        c.step_instruction(&mut bus).unwrap();
        assert_eq!(c.a, 0xF0);
        assert!(!c.flags.ac);
        assert!(!c.flags.cy);
    }

    #[test]
    fn ora_clears_cy_and_ac() {
        let mut c = Cpu8080State::new();
        c.a = 0xFF;
        c.b = 0xFF;
        c.flags.cy = true;
        c.flags.ac = true;
        c.ram.write(0, 0xB0); // ORA B
        let mut bus = NullIoBus;
        c.step_instruction(&mut bus).unwrap();
        assert!(!c.flags.cy);
        assert!(!c.flags.ac);
        assert!(c.flags.s);
    }

    #[test]
    fn xra_clears_cy_and_ac() {
        let mut c = Cpu8080State::new();
        c.a = 0xFF;
        c.flags.cy = true;
        c.flags.ac = true;
        c.ram.write(0, 0xAF); // XRA A → 0
        let mut bus = NullIoBus;
        c.step_instruction(&mut bus).unwrap();
        assert_eq!(c.a, 0);
        assert!(!c.flags.cy);
        assert!(!c.flags.ac);
        assert!(c.flags.z);
        assert!(c.flags.p);
    }
}
