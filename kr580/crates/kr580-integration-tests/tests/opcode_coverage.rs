//! Coverage test that drives every documented 8080 opcode through the core
//! and asserts:
//!
//! * documented opcodes execute without raising `DecodeError`;
//! * undocumented slots (per `prompt/opcode_dispatch.md`) raise
//!   `DecodeError::UndocumentedOpcode` and stop execution.
//!
//! This is a coarse, *table-driven* check. Per-opcode semantic tests live in
//! the unit-test modules of `kr580-core` (`flags`, `alu`, `data`, …).

use kr580_core::{Cpu8080State, IoBus};

fn null_bus() -> impl IoBus {
    kr580_core::NullIoBus
}

/// All undocumented 8080 slots from the prompt opcode dispatch table.
const UNDOCUMENTED: &[u8] = &[
    0x08, 0x10, 0x18, 0x20, 0x28, 0x30, 0x38, 0xCB, 0xD9, 0xDD, 0xED, 0xFD,
];

#[test]
fn every_documented_opcode_executes() {
    let mut bus = null_bus();
    for op in 0u8..=255u8 {
        if UNDOCUMENTED.contains(&op) {
            continue;
        }
        let mut cpu = Cpu8080State::new();
        cpu.sp = 0x4000;
        // Place the opcode plus two filler bytes so any 3-byte instruction
        // can fetch its operands without reading garbage past RAM end.
        cpu.ram.write(0, op);
        cpu.ram.write(1, 0x00);
        cpu.ram.write(2, 0x00);
        // Some opcodes branch through HL or via `RET`. Pre-load HL=0 and
        // ensure there is something at the top of the stack to pop from.
        cpu.ram.write(0x4000, 0x00);
        cpu.ram.write(0x4001, 0x00);
        cpu.h = 0;
        cpu.l = 0;
        let res = cpu.step_instruction(&mut bus);
        assert!(
            res.is_ok(),
            "opcode {op:#04X} unexpectedly raised {:?}",
            res.err()
        );
    }
}

#[test]
fn every_undocumented_slot_raises_decode_error() {
    let mut bus = null_bus();
    for &op in UNDOCUMENTED {
        let mut cpu = Cpu8080State::new();
        cpu.ram.write(0, op);
        let res = cpu.step_instruction(&mut bus);
        assert!(
            res.is_err(),
            "undocumented opcode {op:#04X} did not raise an error"
        );
        assert!(
            cpu.halted,
            "undocumented opcode {op:#04X} must stop execution"
        );
    }
}
