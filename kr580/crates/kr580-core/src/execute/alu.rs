//! Arithmetic family: ADD/ADC/SUB/SBB/CMP, INR/DCR, immediate variants.
//!
//! Auxiliary-carry semantics follow `prompt/02_cpu_core.md` and
//! `prompt/07_cpu_opcode_semantics.md` exactly:
//!
//! * ADD/ADC: `AC = (a & 0xF) + (b & 0xF) + cy_in > 0xF`
//! * SUB/SBB/CMP: AC from the *internal 8080 subtractor*, i.e.
//!   `AC = (a & 0xF) >= ((b & 0xF) + cy_in)`. This is *not* a Z80-style
//!   borrow: `1 - 0` yields `AC=1`, `0 - 1` yields `AC=0` under plain `SUB`.
//! * INR: AC set on low-nibble overflow `0xF -> 0x0`.
//! * DCR: AC = `(result & 0xF) != 0xF`.

use crate::decode::decode_r;
use crate::flags::Flags;
use crate::state::Cpu8080State;

#[inline]
fn read_src_reg_or_m(cpu: &Cpu8080State, op: u8) -> u8 {
    match decode_r(op & 0b111) {
        Some(r) => cpu.get_reg8(r),
        None => cpu.read_m(),
    }
}

#[inline]
fn read_dst_reg_or_m(cpu: &Cpu8080State, op: u8) -> u8 {
    match decode_r((op >> 3) & 0b111) {
        Some(r) => cpu.get_reg8(r),
        None => cpu.read_m(),
    }
}

#[inline]
fn write_dst_reg_or_m(cpu: &mut Cpu8080State, op: u8, v: u8) {
    match decode_r((op >> 3) & 0b111) {
        Some(r) => cpu.set_reg8(r, v),
        None => cpu.write_m(v),
    }
}

/// Core add operation — returns result and updates all flags accordingly.
fn add_inner(a: u8, b: u8, cy_in: bool, f: &mut Flags) -> u8 {
    let cy = cy_in as u16;
    let raw = a as u16 + b as u16 + cy;
    let r = raw as u8;
    f.cy = raw > 0xFF;
    f.ac = ((a & 0x0F) as u16 + (b & 0x0F) as u16 + cy) > 0x0F;
    f.set_szp(r);
    r
}

/// Core sub operation. AC follows the 8080 subtractor model (NOT a borrow).
fn sub_inner(a: u8, b: u8, cy_in: bool, f: &mut Flags) -> u8 {
    let cy = cy_in as i16;
    let raw = a as i16 - b as i16 - cy;
    let r = raw as u8;
    f.cy = raw < 0;
    // 8080 subtractor AC: AC=1 iff low nibble of A >= low nibble of operand + cy_in
    let lhs = (a & 0x0F) as i16;
    let rhs = (b & 0x0F) as i16 + cy;
    f.ac = lhs >= rhs;
    f.set_szp(r);
    r
}

/// `ADD r/M`
pub fn add(cpu: &mut Cpu8080State, op: u8) {
    let v = read_src_reg_or_m(cpu, op);
    cpu.a = add_inner(cpu.a, v, false, &mut cpu.flags);
}

/// `ADC r/M`
pub fn adc(cpu: &mut Cpu8080State, op: u8) {
    let v = read_src_reg_or_m(cpu, op);
    cpu.a = add_inner(cpu.a, v, cpu.flags.cy, &mut cpu.flags);
}

/// `SUB r/M`
pub fn sub(cpu: &mut Cpu8080State, op: u8) {
    let v = read_src_reg_or_m(cpu, op);
    cpu.a = sub_inner(cpu.a, v, false, &mut cpu.flags);
}

/// `SBB r/M`
pub fn sbb(cpu: &mut Cpu8080State, op: u8) {
    let v = read_src_reg_or_m(cpu, op);
    cpu.a = sub_inner(cpu.a, v, cpu.flags.cy, &mut cpu.flags);
}

/// `CMP r/M` — same as SUB but discards the result.
pub fn cmp(cpu: &mut Cpu8080State, op: u8) {
    let v = read_src_reg_or_m(cpu, op);
    let _ = sub_inner(cpu.a, v, false, &mut cpu.flags);
}

/// `ADI d8`
pub fn adi(cpu: &mut Cpu8080State) {
    let v = cpu.fetch_imm8();
    cpu.a = add_inner(cpu.a, v, false, &mut cpu.flags);
}

/// `ACI d8`
pub fn aci(cpu: &mut Cpu8080State) {
    let v = cpu.fetch_imm8();
    cpu.a = add_inner(cpu.a, v, cpu.flags.cy, &mut cpu.flags);
}

/// `SUI d8`
pub fn sui(cpu: &mut Cpu8080State) {
    let v = cpu.fetch_imm8();
    cpu.a = sub_inner(cpu.a, v, false, &mut cpu.flags);
}

/// `SBI d8`
pub fn sbi(cpu: &mut Cpu8080State) {
    let v = cpu.fetch_imm8();
    cpu.a = sub_inner(cpu.a, v, cpu.flags.cy, &mut cpu.flags);
}

/// `CPI d8`
pub fn cpi(cpu: &mut Cpu8080State) {
    let v = cpu.fetch_imm8();
    let _ = sub_inner(cpu.a, v, false, &mut cpu.flags);
}

/// `INR r/M` — does not change CY. AC set on `0xF -> 0x0` low-nibble overflow.
pub fn inr(cpu: &mut Cpu8080State, op: u8) {
    let v = read_dst_reg_or_m(cpu, op);
    let r = v.wrapping_add(1);
    cpu.flags.ac = (v & 0x0F) == 0x0F;
    cpu.flags.set_szp(r);
    write_dst_reg_or_m(cpu, op, r);
}

/// `DCR r/M` — does not change CY.
/// AC = `((result & 0x0F) != 0x0F)` per `prompt/07_cpu_opcode_semantics.md`.
pub fn dcr(cpu: &mut Cpu8080State, op: u8) {
    let v = read_dst_reg_or_m(cpu, op);
    let r = v.wrapping_sub(1);
    cpu.flags.ac = (r & 0x0F) != 0x0F;
    cpu.flags.set_szp(r);
    write_dst_reg_or_m(cpu, op, r);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flags::Flags;

    fn flagged(cpu: &Cpu8080State) -> Flags {
        cpu.flags
    }

    #[test]
    fn sub_one_minus_zero_sets_ac() {
        // Per prompt: 1 - 0 must yield AC=1 under plain SUB.
        let mut c = Cpu8080State::new();
        c.a = 1;
        c.b = 0;
        c.ram.write(0, 0x90); // SUB B
        let mut bus = crate::io::NullIoBus;
        c.step_instruction(&mut bus).unwrap();
        assert_eq!(c.a, 1);
        assert!(flagged(&c).ac, "1-0 must set AC under 8080 SUB");
        assert!(!flagged(&c).cy);
        assert!(!flagged(&c).z);
    }

    #[test]
    fn sub_zero_minus_one_clears_ac() {
        // Per prompt: 0 - 1 must yield AC=0 under plain SUB.
        let mut c = Cpu8080State::new();
        c.a = 0;
        c.b = 1;
        c.ram.write(0, 0x90);
        let mut bus = crate::io::NullIoBus;
        c.step_instruction(&mut bus).unwrap();
        assert_eq!(c.a, 0xFF);
        assert!(!flagged(&c).ac, "0-1 must clear AC under 8080 SUB");
        assert!(flagged(&c).cy, "0-1 produces borrow");
        assert!(flagged(&c).s);
    }

    #[test]
    fn add_low_nibble_overflow_sets_ac() {
        let mut c = Cpu8080State::new();
        c.a = 0x0F;
        c.b = 0x01;
        c.ram.write(0, 0x80); // ADD B
        let mut bus = crate::io::NullIoBus;
        c.step_instruction(&mut bus).unwrap();
        assert_eq!(c.a, 0x10);
        assert!(c.flags.ac);
        assert!(!c.flags.cy);
    }

    #[test]
    fn inr_sets_ac_on_nibble_rollover() {
        let mut c = Cpu8080State::new();
        c.b = 0x0F;
        c.ram.write(0, 0x04); // INR B
        let mut bus = crate::io::NullIoBus;
        c.step_instruction(&mut bus).unwrap();
        assert_eq!(c.b, 0x10);
        assert!(c.flags.ac);
        assert!(!c.flags.cy, "INR must not touch CY");
    }

    #[test]
    fn dcr_ac_rule() {
        let mut c = Cpu8080State::new();
        c.b = 0x10;
        c.ram.write(0, 0x05); // DCR B → 0x0F → AC = false
        let mut bus = crate::io::NullIoBus;
        c.step_instruction(&mut bus).unwrap();
        assert_eq!(c.b, 0x0F);
        assert!(!c.flags.ac, "DCR 0x10 -> 0x0F must clear AC");

        let mut c = Cpu8080State::new();
        c.b = 0x01;
        c.ram.write(0, 0x05); // DCR B → 0x00 → AC = true
        c.step_instruction(&mut bus).unwrap();
        assert_eq!(c.b, 0x00);
        assert!(c.flags.ac);
        assert!(c.flags.z);
    }
}
