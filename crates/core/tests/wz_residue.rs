//! Behavioural tests for the internal `W`/`Z` scratch pair.
//!
//! These cells are programmer-invisible (no instruction reads or
//! writes them directly), but the microsequencer parks the address
//! operand of every memory-addressing or control-transfer command in
//! WZ on its way to the final destination. The reference emulator
//! we visually match against displays that residue in its
//! multiplexer panel.
//!
//! Conventions:
//! * For 16-bit operands the high byte goes into `W`, the low byte
//!   into `Z` (textbook microcode order).
//! * Opcodes that operate on already-resident register pairs
//!   (`STAX BC`, `LDAX DE`, …) intentionally leave WZ untouched.

use k580_core::{Cpu8080State, NullBus};

fn run_program(bytes: &[u8]) -> Cpu8080State {
    let mut cpu = Cpu8080State::default();
    cpu.sp = 0x4000;
    for (offset, byte) in bytes.iter().copied().enumerate() {
        cpu.memory.write(offset as u16, byte);
    }
    let mut bus = NullBus::default();
    for _ in 0..bytes.len() {
        if cpu.pc as usize >= bytes.len() {
            break;
        }
        cpu.step_instruction(&mut bus).unwrap();
    }
    cpu
}

fn step(cpu: &mut Cpu8080State) {
    let mut bus = NullBus::default();
    cpu.step_instruction(&mut bus).unwrap();
}

#[test]
fn sta_records_address_in_wz() {
    // STA 2000h: reference emulator shows W=20, Z=00 after.
    let cpu = run_program(&[0x3E, 0x46, 0x32, 0x00, 0x20]);
    assert_eq!(cpu.registers.w, 0x20);
    assert_eq!(cpu.registers.z, 0x00);
    assert_eq!(cpu.memory.read(0x2000), 0x46);
}

#[test]
fn lda_records_address_in_wz() {
    let mut cpu = Cpu8080State::default();
    cpu.memory.write(0x1234, 0x99);
    cpu.memory.write(0x0000, 0x3A);
    cpu.memory.write(0x0001, 0x34);
    cpu.memory.write(0x0002, 0x12);
    step(&mut cpu);
    assert_eq!(cpu.registers.a, 0x99);
    assert_eq!(cpu.registers.w, 0x12);
    assert_eq!(cpu.registers.z, 0x34);
}

#[test]
fn lhld_records_address_in_wz() {
    let mut cpu = Cpu8080State::default();
    cpu.memory.write(0x1234, 0xAA);
    cpu.memory.write(0x1235, 0xBB);
    cpu.memory.write(0x0000, 0x2A);
    cpu.memory.write(0x0001, 0x34);
    cpu.memory.write(0x0002, 0x12);
    step(&mut cpu);
    assert_eq!(cpu.registers.l, 0xAA);
    assert_eq!(cpu.registers.h, 0xBB);
    assert_eq!(cpu.registers.w, 0x12);
    assert_eq!(cpu.registers.z, 0x34);
}

#[test]
fn shld_records_address_in_wz() {
    let mut cpu = Cpu8080State::default();
    cpu.registers.h = 0xBB;
    cpu.registers.l = 0xAA;
    cpu.memory.write(0x0000, 0x22);
    cpu.memory.write(0x0001, 0x34);
    cpu.memory.write(0x0002, 0x12);
    step(&mut cpu);
    assert_eq!(cpu.memory.read(0x1234), 0xAA);
    assert_eq!(cpu.memory.read(0x1235), 0xBB);
    assert_eq!(cpu.registers.w, 0x12);
    assert_eq!(cpu.registers.z, 0x34);
}

#[test]
fn lxi_records_immediate_in_wz() {
    // LXI B, 1234h
    let mut cpu = Cpu8080State::default();
    cpu.memory.write(0x0000, 0x01);
    cpu.memory.write(0x0001, 0x34);
    cpu.memory.write(0x0002, 0x12);
    step(&mut cpu);
    assert_eq!(cpu.registers.b, 0x12);
    assert_eq!(cpu.registers.c, 0x34);
    assert_eq!(cpu.registers.w, 0x12);
    assert_eq!(cpu.registers.z, 0x34);
}

#[test]
fn jmp_records_target_in_wz() {
    let mut cpu = Cpu8080State::default();
    cpu.memory.write(0x0000, 0xC3);
    cpu.memory.write(0x0001, 0x78);
    cpu.memory.write(0x0002, 0x56);
    step(&mut cpu);
    assert_eq!(cpu.pc, 0x5678);
    assert_eq!(cpu.registers.w, 0x56);
    assert_eq!(cpu.registers.z, 0x78);
}

#[test]
fn jcond_records_target_even_when_not_taken() {
    // Microcode fetches both operand bytes before the flag test, so
    // WZ holds the operand even on the not-taken branch.
    let mut cpu = Cpu8080State::default();
    cpu.flags.zero = false;
    cpu.memory.write(0x0000, 0xCA);
    cpu.memory.write(0x0001, 0x34);
    cpu.memory.write(0x0002, 0x12);
    step(&mut cpu);
    assert_eq!(cpu.pc, 0x0003); // not taken
    assert_eq!(cpu.registers.w, 0x12);
    assert_eq!(cpu.registers.z, 0x34);
}

#[test]
fn call_records_target_in_wz() {
    let mut cpu = Cpu8080State::default();
    cpu.sp = 0x4000;
    cpu.memory.write(0x0000, 0xCD);
    cpu.memory.write(0x0001, 0x78);
    cpu.memory.write(0x0002, 0x56);
    step(&mut cpu);
    assert_eq!(cpu.pc, 0x5678);
    assert_eq!(cpu.registers.w, 0x56);
    assert_eq!(cpu.registers.z, 0x78);
}

#[test]
fn ret_records_popped_address_in_wz() {
    // Stack holds 0x1234 (lo=0x34 at SP, hi=0x12 at SP+1).
    let mut cpu = Cpu8080State::default();
    cpu.sp = 0x4000;
    cpu.memory.write(0x4000, 0x34);
    cpu.memory.write(0x4001, 0x12);
    cpu.memory.write(0x0000, 0xC9);
    step(&mut cpu);
    assert_eq!(cpu.pc, 0x1234);
    assert_eq!(cpu.registers.w, 0x12);
    assert_eq!(cpu.registers.z, 0x34);
}

#[test]
fn rst_records_target_in_wz() {
    // RST 5 → target 0x0028.
    let mut cpu = Cpu8080State::default();
    cpu.sp = 0x4000;
    cpu.memory.write(0x0010, 0xEF); // RST 5 sits at 0x10
    cpu.pc = 0x0010;
    step(&mut cpu);
    assert_eq!(cpu.pc, 0x0028);
    assert_eq!(cpu.registers.w, 0x00);
    assert_eq!(cpu.registers.z, 0x28);
}

#[test]
fn pchl_records_hl_in_wz() {
    let mut cpu = Cpu8080State::default();
    cpu.registers.h = 0xAB;
    cpu.registers.l = 0xCD;
    cpu.memory.write(0x0000, 0xE9);
    step(&mut cpu);
    assert_eq!(cpu.pc, 0xABCD);
    assert_eq!(cpu.registers.w, 0xAB);
    assert_eq!(cpu.registers.z, 0xCD);
}

#[test]
fn sphl_records_hl_in_wz() {
    let mut cpu = Cpu8080State::default();
    cpu.registers.h = 0x20;
    cpu.registers.l = 0x00;
    cpu.memory.write(0x0000, 0xF9);
    step(&mut cpu);
    assert_eq!(cpu.sp, 0x2000);
    assert_eq!(cpu.registers.w, 0x20);
    assert_eq!(cpu.registers.z, 0x00);
}

#[test]
fn xchg_parks_previous_hl_in_wz() {
    // After XCHG the WZ residue is the previous HL (it travelled
    // HL → WZ → DE during the swap).
    let mut cpu = Cpu8080State::default();
    cpu.registers.h = 0x11;
    cpu.registers.l = 0x22;
    cpu.registers.d = 0x33;
    cpu.registers.e = 0x44;
    cpu.memory.write(0x0000, 0xEB);
    step(&mut cpu);
    assert_eq!(cpu.registers.hl(), 0x3344);
    assert_eq!(cpu.registers.de(), 0x1122);
    assert_eq!(cpu.registers.w, 0x11);
    assert_eq!(cpu.registers.z, 0x22);
}

#[test]
fn xthl_records_top_of_stack_in_wz() {
    let mut cpu = Cpu8080State::default();
    cpu.sp = 0x4000;
    cpu.registers.h = 0x11;
    cpu.registers.l = 0x22;
    cpu.memory.write(0x4000, 0xCD);
    cpu.memory.write(0x4001, 0xAB);
    cpu.memory.write(0x0000, 0xE3);
    step(&mut cpu);
    assert_eq!(cpu.registers.h, 0xAB);
    assert_eq!(cpu.registers.l, 0xCD);
    assert_eq!(cpu.memory.read(0x4000), 0x22);
    assert_eq!(cpu.memory.read(0x4001), 0x11);
    assert_eq!(cpu.registers.w, 0xAB);
    assert_eq!(cpu.registers.z, 0xCD);
}

#[test]
fn ldax_does_not_touch_wz() {
    // LDAX uses the BC pair directly; WZ must keep the prior LXI residue.
    let mut cpu = Cpu8080State::default();
    cpu.memory.write(0x1234, 0x77);
    cpu.memory.write(0x0000, 0x01);
    cpu.memory.write(0x0001, 0x34);
    cpu.memory.write(0x0002, 0x12);
    cpu.memory.write(0x0003, 0x0A); // LDAX B
    step(&mut cpu); // LXI B, 1234h
    step(&mut cpu); // LDAX B
    assert_eq!(cpu.registers.a, 0x77);
    assert_eq!(cpu.registers.w, 0x12);
    assert_eq!(cpu.registers.z, 0x34);
}

#[test]
fn rcond_not_taken_does_not_touch_wz() {
    // Not-taken Rcond runs no memory cycle, so it must leave WZ alone.
    let mut cpu = Cpu8080State::default();
    cpu.sp = 0x4000;
    cpu.flags.zero = false;
    cpu.memory.write(0x0000, 0x01); // LXI B, 1234h
    cpu.memory.write(0x0001, 0x34);
    cpu.memory.write(0x0002, 0x12);
    cpu.memory.write(0x0003, 0xC8); // RZ
    step(&mut cpu);
    step(&mut cpu);
    assert_eq!(cpu.registers.w, 0x12);
    assert_eq!(cpu.registers.z, 0x34);
}

#[test]
fn reset_zeroes_wz() {
    let mut cpu = Cpu8080State::default();
    cpu.registers.w = 0xFF;
    cpu.registers.z = 0xEE;
    cpu.reset_cpu();
    assert_eq!(cpu.registers.w, 0x00);
    assert_eq!(cpu.registers.z, 0x00);
}
