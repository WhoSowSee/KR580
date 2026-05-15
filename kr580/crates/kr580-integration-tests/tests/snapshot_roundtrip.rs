//! End-to-end snapshot roundtrip: run a short program, save, restore, and
//! verify the new state matches.

use kr580_core::{Cpu8080State, NullIoBus};
use kr580_persistence::Snapshot580Serializer;

#[test]
fn snapshot_after_program_restores_exact_state() {
    let mut cpu = Cpu8080State::new();
    cpu.sp = 0x2000;
    // Program: MVI A,0x42; STA 0x0100; HLT
    cpu.ram.write(0, 0x3E);
    cpu.ram.write(1, 0x42);
    cpu.ram.write(2, 0x32);
    cpu.ram.write(3, 0x00);
    cpu.ram.write(4, 0x01);
    cpu.ram.write(5, 0x76); // HLT

    let mut bus = NullIoBus;
    cpu.run_until_halt(&mut bus, 16).unwrap();
    assert!(cpu.halted);
    assert_eq!(cpu.a, 0x42);
    assert_eq!(cpu.ram.read(0x0100), 0x42);

    let bytes = Snapshot580Serializer::save(&cpu);
    let back = Snapshot580Serializer::load(&bytes).expect("load snapshot");
    assert_eq!(back.a, 0x42);
    assert_eq!(back.ram.read(0x0100), 0x42);
    assert!(back.halted);
    assert_eq!(back.cycle_count, cpu.cycle_count);
}

#[test]
fn snapshot_format_is_deterministic() {
    let mut cpu = Cpu8080State::new();
    cpu.a = 0x01;
    cpu.pc = 0x0100;
    let a = Snapshot580Serializer::save(&cpu);
    let b = Snapshot580Serializer::save(&cpu);
    assert_eq!(a, b, "snapshot must be byte-identical for identical state");
}
