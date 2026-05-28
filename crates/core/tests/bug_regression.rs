//! Integration test: run the 16 bug-tests from `D:/kr/Examples/bug-tests/`
//! against the Rust core and compare with Intel 8080 reference values.
//! Tests are taken verbatim from the KR580 RE project; a correct
//! emulator MUST produce the Intel-reference value of A after HLT.
//!
//! `.580` layout: 65536 bytes RAM + 13-byte CPU trailer; we only
//! need the RAM portion.

use std::path::PathBuf;

use k580_core::{Cpu8080State, NullBus, RegisterName};

const BUG_TESTS_DIR: &str = r"D:\kr\Examples\bug-tests";

fn load_580(name: &str) -> Vec<u8> {
    let path = PathBuf::from(BUG_TESTS_DIR).join(name);
    let bytes = std::fs::read(&path).unwrap_or_else(|err| {
        panic!("could not read {}: {err}", path.display());
    });
    assert!(
        bytes.len() >= 65_536,
        "{} too short: {} bytes",
        path.display(),
        bytes.len()
    );
    bytes[..65_536].to_vec()
}

fn run_until_halt(cpu: &mut Cpu8080State) {
    let mut bus = NullBus::default();
    let executed = cpu
        .run_until_halt(&mut bus, 10_000)
        .expect("CPU error during bug-test execution");
    assert!(
        cpu.halted,
        "program did not reach HLT within {executed} instructions; pc={:#06X}",
        cpu.pc
    );
}

fn run_bug_test(filename: &str, expected_a: u8) {
    let ram = load_580(filename);
    let mut cpu = Cpu8080State::default();
    cpu.memory.as_mut_slice().copy_from_slice(&ram);
    run_until_halt(&mut cpu);
    let actual = cpu.get_register(RegisterName::A);
    assert_eq!(
        actual, expected_a,
        "{filename}: expected A={:#04X}, got A={:#04X}",
        expected_a, actual
    );
}

// Group A — Rcc by Z, CY, P (test1..test12).

#[test]
fn bug_test1_rz_z1_must_return() {
    run_bug_test("test1_RZ_Z1_must_return.580", 0x55);
}
#[test]
fn bug_test2_rz_z0_must_not_return() {
    run_bug_test("test2_RZ_Z0_must_NOT_return.580", 0xFF);
}
#[test]
fn bug_test3_rnz_z0_must_return() {
    run_bug_test("test3_RNZ_Z0_must_return.580", 0x55);
}
#[test]
fn bug_test4_rnz_z1_must_not_return() {
    run_bug_test("test4_RNZ_Z1_must_NOT_return.580", 0xFF);
}
#[test]
fn bug_test5_rc_cy1_must_return() {
    run_bug_test("test5_RC_CY1_must_return.580", 0x55);
}
#[test]
fn bug_test6_rc_cy0_must_not_return() {
    run_bug_test("test6_RC_CY0_must_NOT_return.580", 0xFF);
}
#[test]
fn bug_test7_rnc_cy0_must_return() {
    run_bug_test("test7_RNC_CY0_must_return.580", 0x55);
}
#[test]
fn bug_test8_rnc_cy1_must_not_return() {
    run_bug_test("test8_RNC_CY1_must_NOT_return.580", 0xFF);
}
#[test]
fn bug_test9_rpe_p1_must_return() {
    run_bug_test("test9_RPE_P1_must_return.580", 0x55);
}
#[test]
fn bug_test10_rpe_p0_must_not_return() {
    run_bug_test("test10_RPE_P0_must_NOT_return.580", 0xFF);
}
#[test]
fn bug_test11_rpo_p0_must_return() {
    run_bug_test("test11_RPO_P0_must_return.580", 0x55);
}
#[test]
fn bug_test12_rpo_p1_must_not_return() {
    run_bug_test("test12_RPO_P1_must_NOT_return.580", 0xFF);
}

// Group B — Ccc by S: CP/CM (test13..test16).
// Original `KP580.exe` FAILS these (BUG-09: read_flag for S is constant 0).

#[test]
fn bug_test13_cp_s0_p1_must_call() {
    run_bug_test("test13_CP_S0_P1_must_call.580", 0x22);
}
#[test]
fn bug_test14_cm_s1_p0_must_call() {
    run_bug_test("test14_CM_S1_P0_must_call.580", 0x22);
}
#[test]
fn bug_test15_cm_s0_p1_must_not_call() {
    run_bug_test("test15_CM_S0_P1_must_NOT_call.580", 0x11);
}
#[test]
fn bug_test16_cp_s1_p0_must_not_call() {
    run_bug_test("test16_CP_S1_P0_must_NOT_call.580", 0x11);
}

// Direct synthetic regressions for BUG-01..05, BUG-07 from the
// original `KP580.exe`. Built in-memory; each test asserts the
// correct Intel-8080 behaviour.

/// BUG-01 reproducer (JP/JM read parity instead of sign).
/// Buggy: A=0x11 on the must-jump cases. Correct: A=0x22.
#[test]
fn bug01_jp_uses_sign_not_parity() {
    // A=0x03 → S=0, P=1. JP must jump (S==0).
    let mut cpu = Cpu8080State::default();
    let prog = [
        0x3E, 0x03, // MVI A, 0x03
        0xB7, // ORA A
        0xF2, 0x0C, 0x00, // JP 0x000C
        0x3E, 0x11, // MVI A, 0x11
        0x76, // HLT
        0x00, 0x00, 0x00, // padding
        0x3E, 0x22, // MVI A, 0x22
        0x76, // HLT
    ];
    for (i, b) in prog.iter().enumerate() {
        cpu.memory.write(i as u16, *b);
    }
    run_until_halt(&mut cpu);
    assert_eq!(
        cpu.get_register(RegisterName::A),
        0x22,
        "JP must jump on S=0"
    );
}

/// BUG-01 reproducer for JM with S=1, P=0.
#[test]
fn bug01_jm_uses_sign_not_parity() {
    let mut cpu = Cpu8080State::default();
    let prog = [
        0x3E, 0x80, // MVI A, 0x80  (S=1, P=0)
        0xB7, // ORA A
        0xFA, 0x0C, 0x00, // JM 0x000C
        0x3E, 0x11, 0x76, 0x00, 0x00, 0x00, 0x3E, 0x22, 0x76,
    ];
    for (i, b) in prog.iter().enumerate() {
        cpu.memory.write(i as u16, *b);
    }
    run_until_halt(&mut cpu);
    assert_eq!(
        cpu.get_register(RegisterName::A),
        0x22,
        "JM must jump on S=1"
    );
}

/// BUG-02 reproducer (RRC must use bit0, not old CY).
/// Setup: A=0x02, CY=1. Spec: A := rotr(0x02) = 0x01, CY := bit0(0x02) = 0.
/// Buggy KP580.exe: A=0x81 (puts old CY=1 in bit7).
#[test]
fn bug02_rrc_uses_bit0_not_old_carry() {
    let mut cpu = Cpu8080State::default();
    cpu.flags.carry = true;
    cpu.registers.a = 0x02;
    cpu.memory.write(0, 0x0F); // RRC
    cpu.memory.write(1, 0x76); // HLT
    run_until_halt(&mut cpu);
    assert_eq!(cpu.get_register(RegisterName::A), 0x01, "RRC must use bit0");
    assert!(!cpu.flags.carry, "RRC of 0x02 sets CY=0 (bit0 of 0x02)");
}

/// BUG-03 reproducer (RAR must rotate A through CY).
/// Setup: A=0xC3, CY=1. Spec: result = (CY<<7) | (A>>1) = 0x80 | 0x61 = 0xE1.
/// Buggy KP580.exe: A is lost (helper called without argument).
#[test]
fn bug03_rar_preserves_data_and_uses_old_carry() {
    let mut cpu = Cpu8080State::default();
    cpu.registers.a = 0xC3;
    cpu.flags.carry = true;
    cpu.memory.write(0, 0x1F); // RAR
    cpu.memory.write(1, 0x76); // HLT
    run_until_halt(&mut cpu);
    assert_eq!(cpu.get_register(RegisterName::A), 0xE1);
    assert!(cpu.flags.carry, "old bit0 of 0xC3 = 1 → new CY=1");
}

/// BUG-04 reproducer (DAA must adjust BCD addition).
/// 0x15 + 0x27 = 0x3C, AC=1; DAA → A=0x42.
#[test]
fn bug04_daa_adjusts_bcd_addition() {
    let mut cpu = Cpu8080State::default();
    cpu.registers.a = 0x15;
    cpu.registers.b = 0x27;
    cpu.memory.write(0, 0x80); // ADD B
    cpu.memory.write(1, 0x27); // DAA
    cpu.memory.write(2, 0x76); // HLT
    run_until_halt(&mut cpu);
    assert_eq!(cpu.get_register(RegisterName::A), 0x42);
}

/// BUG-05 reproducer (IN must not halt the CPU).
/// IN reads from a bus port; emulator must continue past it.
#[test]
fn bug05_in_returns_port_byte_and_advances() {
    let mut cpu = Cpu8080State::default();
    cpu.memory.write(0, 0xDB); // IN
    cpu.memory.write(1, 0x05); // port 0x05
    cpu.memory.write(2, 0x76); // HLT
    let mut bus = NullBus::default();
    bus.set_input(0x05, 0x99);
    cpu.run_until_halt(&mut bus, 10).unwrap();
    assert!(cpu.halted);
    assert_eq!(cpu.get_register(RegisterName::A), 0x99);
}

/// BUG-07 reproducer (8080 AC quirk in ANA: AC = ((A | operand) & 0x08) != 0).
/// A=0x08, B=0x00 → ANA B → A=0; AC must be 1 (because (0x08|0)&0x08 ≠ 0).
#[test]
fn bug07_ana_ac_follows_8080_quirk() {
    let mut cpu = Cpu8080State::default();
    cpu.registers.a = 0x08;
    cpu.registers.b = 0x00;
    cpu.memory.write(0, 0xA0); // ANA B
    cpu.memory.write(1, 0x76); // HLT
    run_until_halt(&mut cpu);
    assert_eq!(cpu.get_register(RegisterName::A), 0x00);
    assert!(cpu.flags.zero);
    assert!(
        cpu.flags.auxiliary_carry,
        "ANA must set AC per 8080 quirk: ((A|operand)&0x08)!=0"
    );
}
