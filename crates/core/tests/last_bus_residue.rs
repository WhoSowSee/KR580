//! Regression coverage for the bus-latch residues that the schematic
//! reads out: `last_fetched_opcode` (IR), `last_data_bus_byte` (data
//! buffer) and `last_address_bus` (address buffer). Before the fix
//! these four panels showed a `memory.read(pc)` look-ahead and drifted
//! from the reference after `HLT`, after writes, and after operand
//! fetches. The tests pin the semantics down:
//!
//! - after an opcode fetch the IR holds **that** byte, not RAM[PC];
//! - after a memory write the data buffer holds the written byte and
//!   the address buffer holds the write target (not PC);
//! - after `HLT` PC has advanced but the IR keeps `0x76` until the
//!   next M1 (which never happens until the halt clears).

use k580_core::{Cpu8080State, NullBus};

fn step(cpu: &mut Cpu8080State) {
    let mut bus = NullBus::default();
    cpu.step_instruction(&mut bus).unwrap();
}

fn put_program(cpu: &mut Cpu8080State, bytes: &[u8]) {
    for (offset, byte) in bytes.iter().copied().enumerate() {
        cpu.memory.write(offset as u16, byte);
    }
}

#[test]
fn mvi_records_opcode_in_ir_and_immediate_in_data_buffer() {
    let mut cpu = Cpu8080State::default();
    put_program(&mut cpu, &[0x3E, 0x42]); // MVI A, 0x42
    step(&mut cpu);
    assert_eq!(cpu.registers.a, 0x42);
    assert_eq!(
        cpu.last_fetched_opcode, 0x3E,
        "IR holds the MVI A opcode, not the byte at the new PC"
    );
    assert_eq!(
        cpu.last_data_bus_byte, 0x42,
        "data buffer holds the immediate operand (last byte across the bus)"
    );
    assert_eq!(
        cpu.last_address_bus, 0x0001,
        "address buffer holds the immediate operand address, not the new PC"
    );
}

#[test]
fn sta_records_written_byte_and_target_address() {
    let mut cpu = Cpu8080State::default();
    cpu.registers.a = 0x77;
    put_program(&mut cpu, &[0x32, 0x00, 0x40]); // STA 0x4000
    step(&mut cpu);
    assert_eq!(cpu.memory.read(0x4000), 0x77);
    assert_eq!(
        cpu.last_data_bus_byte, 0x77,
        "data buffer holds the written byte"
    );
    assert_eq!(
        cpu.last_address_bus, 0x4000,
        "address buffer holds the destination, not PC"
    );
    assert_eq!(
        cpu.last_fetched_opcode, 0x32,
        "IR keeps the STA opcode until the next M1"
    );
}

/// After `HLT` (opcode `0x76`) PC advances one byte, but the next M1
/// will not happen until an interrupt or reset arrives – so the IR
/// must keep `0x76`. The old readout used `memory.read(pc)` and
/// showed `0x00` (NOP from blank RAM at the new PC).
#[test]
fn hlt_freezes_ir_at_seventy_six() {
    let mut cpu = Cpu8080State::default();
    put_program(
        &mut cpu,
        &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x76],
    );
    for _ in 0..8 {
        step(&mut cpu);
    }
    assert_eq!(cpu.pc, 0x0008);
    step(&mut cpu);
    assert!(cpu.halted, "HLT raised the halt flag");
    assert_eq!(cpu.pc, 0x0009, "PC stepped past HLT (datasheet behaviour)");
    assert_eq!(
        cpu.last_fetched_opcode, 0x76,
        "IR still holds 0x76 after HLT (no further M1 until clear)"
    );
    assert_eq!(
        cpu.last_address_bus, 0x0008,
        "address buffer = HLT address, not the new PC"
    );
    assert_eq!(
        cpu.last_data_bus_byte, 0x76,
        "data buffer = HLT opcode (last byte across the bus)"
    );
}

/// `MOV A, B` is purely internal between registers. The bus only sees
/// the M1 fetch, so all latches must reflect that fetch. Catches
/// regressions where `read_reg_code` would touch the bus latches for
/// register codes other than `M=110`.
#[test]
fn mov_register_to_register_does_not_touch_bus_beyond_m1() {
    let mut cpu = Cpu8080State::default();
    cpu.registers.b = 0xAB;
    put_program(&mut cpu, &[0x78]); // MOV A, B
    step(&mut cpu);
    assert_eq!(cpu.registers.a, 0xAB);
    assert_eq!(cpu.last_fetched_opcode, 0x78);
    assert_eq!(cpu.last_address_bus, 0x0000);
    assert_eq!(cpu.last_data_bus_byte, 0x78);
}

/// `MOV A, (HL)` – indirect read. HL goes onto the address buffer
/// and the fetched byte goes onto the data buffer; latches must
/// reflect that, not the M1 fetch.
#[test]
fn mov_a_from_hl_indirect_records_hl_and_byte() {
    let mut cpu = Cpu8080State::default();
    cpu.registers.set_hl(0x2000);
    cpu.memory.write(0x2000, 0x5A);
    put_program(&mut cpu, &[0x7E]); // MOV A, (HL)
    step(&mut cpu);
    assert_eq!(cpu.registers.a, 0x5A);
    assert_eq!(cpu.last_fetched_opcode, 0x7E, "IR = MOV A,(HL) opcode");
    assert_eq!(
        cpu.last_address_bus, 0x2000,
        "address buffer holds HL, not the opcode PC"
    );
    assert_eq!(
        cpu.last_data_bus_byte, 0x5A,
        "data buffer holds the byte read from (HL)"
    );
}

/// `Reset` must zero the bus latches; otherwise loading a fresh
/// program leaves stale residues from the previous session in the
/// IR and bus buffers. `Cpu8080State::default()` also zeroes them, so
/// both paths are covered.
#[test]
fn reset_clears_bus_latches() {
    let mut cpu = Cpu8080State::default();
    put_program(&mut cpu, &[0x3E, 0x42]); // MVI A
    step(&mut cpu);
    assert_ne!(cpu.last_fetched_opcode, 0);
    cpu.reset_cpu();
    assert_eq!(cpu.last_fetched_opcode, 0);
    assert_eq!(cpu.last_data_bus_byte, 0);
    assert_eq!(cpu.last_address_bus, 0);
}
