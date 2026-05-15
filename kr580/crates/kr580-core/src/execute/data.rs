//! Data-transfer family: MOV, MVI, LXI, LDA/STA, LHLD/SHLD, LDAX/STAX,
//! XCHG, XTHL, SPHL, INX/DCX, DAD.

use crate::decode::{decode_r, decode_rp_sp};
use crate::state::{Cpu8080State, RegPair};

/// `MOV dst, src`. `M` (memory at HL) is permitted on either side, but the
/// `MOV M,M` slot is `HLT` and is dispatched separately.
pub fn mov(cpu: &mut Cpu8080State, op: u8) {
    let dst_code = (op >> 3) & 0b111;
    let src_code = op & 0b111;
    let v = match decode_r(src_code) {
        Some(r) => cpu.get_reg8(r),
        None => cpu.read_m(),
    };
    match decode_r(dst_code) {
        Some(r) => cpu.set_reg8(r, v),
        None => cpu.write_m(v),
    }
}

/// `MVI dst, d8` for `dst` in {B,C,D,E,H,L,M,A}, opcode bits 5-3.
pub fn mvi(cpu: &mut Cpu8080State, op: u8) {
    let v = cpu.fetch_imm8();
    let dst_code = (op >> 3) & 0b111;
    match decode_r(dst_code) {
        Some(r) => cpu.set_reg8(r, v),
        None => cpu.write_m(v),
    }
}

/// `LXI rp, d16` for `rp` in {BC,DE,HL,SP}.
pub fn lxi_rp(cpu: &mut Cpu8080State, op: u8) {
    let rp = decode_rp_sp((op >> 4) & 0b11);
    let v = cpu.fetch_imm16();
    cpu.set_pair(rp, v);
}

/// `STAX rp` for `rp` in {BC, DE} only.
pub fn stax_rp(cpu: &mut Cpu8080State, op: u8) {
    let rp = match (op >> 4) & 0b11 {
        0 => RegPair::Bc,
        1 => RegPair::De,
        _ => unreachable!("stax dispatch"),
    };
    let addr = cpu.get_pair(rp);
    cpu.ram.write(addr, cpu.a);
}

/// `LDAX rp` for `rp` in {BC, DE} only.
pub fn ldax_rp(cpu: &mut Cpu8080State, op: u8) {
    let rp = match (op >> 4) & 0b11 {
        0 => RegPair::Bc,
        1 => RegPair::De,
        _ => unreachable!("ldax dispatch"),
    };
    let addr = cpu.get_pair(rp);
    cpu.a = cpu.ram.read(addr);
}

/// `INX rp` — does not affect flags.
pub fn inx_rp(cpu: &mut Cpu8080State, op: u8) {
    let rp = decode_rp_sp((op >> 4) & 0b11);
    let v = cpu.get_pair(rp).wrapping_add(1);
    cpu.set_pair(rp, v);
}

/// `DCX rp` — does not affect flags.
pub fn dcx_rp(cpu: &mut Cpu8080State, op: u8) {
    let rp = decode_rp_sp((op >> 4) & 0b11);
    let v = cpu.get_pair(rp).wrapping_sub(1);
    cpu.set_pair(rp, v);
}

/// `DAD rp` — only CY is modified.
pub fn dad_rp(cpu: &mut Cpu8080State, op: u8) {
    let rp = decode_rp_sp((op >> 4) & 0b11);
    let hl = cpu.hl() as u32;
    let v = cpu.get_pair(rp) as u32;
    let r = hl + v;
    cpu.set_pair(RegPair::Hl, r as u16);
    cpu.flags.cy = (r & 0x1_0000) != 0;
}

/// `STA a16` — store A at the immediate address.
pub fn sta(cpu: &mut Cpu8080State) {
    let addr = cpu.fetch_imm16();
    cpu.ram.write(addr, cpu.a);
}

/// `LDA a16` — load A from the immediate address.
pub fn lda(cpu: &mut Cpu8080State) {
    let addr = cpu.fetch_imm16();
    cpu.a = cpu.ram.read(addr);
}

/// `SHLD a16` — store HL (L→addr, H→addr+1).
pub fn shld(cpu: &mut Cpu8080State) {
    let addr = cpu.fetch_imm16();
    cpu.ram.write(addr, cpu.l);
    cpu.ram.write(addr.wrapping_add(1), cpu.h);
}

/// `LHLD a16` — load HL (L from addr, H from addr+1).
pub fn lhld(cpu: &mut Cpu8080State) {
    let addr = cpu.fetch_imm16();
    cpu.l = cpu.ram.read(addr);
    cpu.h = cpu.ram.read(addr.wrapping_add(1));
}

/// `XCHG` — swap HL and DE.
pub fn xchg(cpu: &mut Cpu8080State) {
    std::mem::swap(&mut cpu.h, &mut cpu.d);
    std::mem::swap(&mut cpu.l, &mut cpu.e);
}

/// `XTHL` — swap HL with the top of the stack.
pub fn xthl(cpu: &mut Cpu8080State) {
    let lo = cpu.ram.read(cpu.sp);
    let hi = cpu.ram.read(cpu.sp.wrapping_add(1));
    cpu.ram.write(cpu.sp, cpu.l);
    cpu.ram.write(cpu.sp.wrapping_add(1), cpu.h);
    cpu.l = lo;
    cpu.h = hi;
}

/// `SPHL` — copy HL into SP.
pub fn sphl(cpu: &mut Cpu8080State) {
    cpu.sp = cpu.hl();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::NullIoBus;

    #[test]
    fn lxi_h_loads_immediate_word() {
        let mut c = Cpu8080State::new();
        c.ram.write(0, 0x21); // LXI H,d16
        c.ram.write(1, 0x34);
        c.ram.write(2, 0x12);
        let mut bus = NullIoBus;
        c.step_instruction(&mut bus).unwrap();
        assert_eq!(c.h, 0x12);
        assert_eq!(c.l, 0x34);
        assert_eq!(c.pc, 3);
    }

    #[test]
    fn shld_lhld_roundtrip() {
        let mut c = Cpu8080State::new();
        c.h = 0xAB;
        c.l = 0xCD;
        // SHLD 0x4000
        c.ram.write(0, 0x22);
        c.ram.write(1, 0x00);
        c.ram.write(2, 0x40);
        // LHLD 0x4000
        c.ram.write(3, 0x2A);
        c.ram.write(4, 0x00);
        c.ram.write(5, 0x40);
        let mut bus = NullIoBus;
        c.step_instruction(&mut bus).unwrap();
        c.h = 0;
        c.l = 0;
        c.step_instruction(&mut bus).unwrap();
        assert_eq!(c.h, 0xAB);
        assert_eq!(c.l, 0xCD);
    }

    #[test]
    fn xchg_swaps_hl_and_de() {
        let mut c = Cpu8080State::new();
        c.h = 0x11;
        c.l = 0x22;
        c.d = 0x33;
        c.e = 0x44;
        c.ram.write(0, 0xEB);
        let mut bus = NullIoBus;
        c.step_instruction(&mut bus).unwrap();
        assert_eq!((c.h, c.l), (0x33, 0x44));
        assert_eq!((c.d, c.e), (0x11, 0x22));
    }

    #[test]
    fn dad_only_touches_cy() {
        let mut c = Cpu8080State::new();
        c.h = 0xFF;
        c.l = 0xFF;
        c.b = 0x00;
        c.c = 0x01;
        c.flags.z = true;
        c.flags.s = true;
        c.flags.ac = true;
        c.flags.p = true;
        c.ram.write(0, 0x09); // DAD B
        let mut bus = NullIoBus;
        c.step_instruction(&mut bus).unwrap();
        assert_eq!(c.h, 0x00);
        assert_eq!(c.l, 0x00);
        assert!(c.flags.cy);
        // S/Z/P/AC should be unchanged.
        assert!(c.flags.z);
        assert!(c.flags.s);
        assert!(c.flags.ac);
        assert!(c.flags.p);
    }
}
