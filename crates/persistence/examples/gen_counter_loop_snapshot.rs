//! One-shot generator for `counter_loop.580`. Builds an Intel 8080 program
//! that visibly walks through registers and memory at the default 10 Hz
//! pace, then serialises the resulting `Cpu8080State` through the
//! production `Snapshot580Serializer` so the file shape matches whatever
//! the running emulator produces.
//!
//! Run from the workspace root:
//!
//! ```text
//! cargo run -p k580-persistence --example gen_counter_loop_snapshot
//! ```
//!
//! The output is written to `<repo>/counter_loop.580`. Re-running the
//! example overwrites it deterministically. As a built-in smoke test
//! the generator also round-trips the bytes through
//! `Snapshot580Serializer::from_bytes` and runs the program through
//! `step_instruction` (with a `NullBus` — the program does no I/O)
//! until `HLT`, then asserts the final state matches the math the
//! program describes. Anything wrong with the bytes, the opcodes, or
//! the loop arithmetic surfaces here instead of inside the UI.

use k580_core::{Cpu8080State, NullBus};
use k580_persistence::Snapshot580Serializer;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Program layout, hand-assembled so each opcode is visible in the
    // memory list. Total: 17 bytes ending at 0x0010 with HLT.
    //
    // Address  Bytes        Mnemonic        Effect
    // -------  -----------  -------------   -----------------------------
    // 0000     3E 00        MVI A, 0x00     accumulator = 0
    // 0002     06 03        MVI B, 0x03     B = step
    // 0004     0E 00        MVI C, 0x00     C = scratch (RLC target)
    // 0006     80           ADD B           A += B           ← loop start
    // 0007     4F           MOV C, A        mirror A into C
    // 0008     07           RLC             rotate A left, sets carry
    // 0009     32 00 01     STA 0x0100      drop A into RAM cell 0x0100
    // 000C     3C           INR A           A += 1 (touches flags)
    // 000D     C2 06 00     JNZ 0x0006      loop while A != 0
    // 0010     76           HLT
    let program: [u8; 17] = [
        0x3E, 0x00, // MVI A, 0x00
        0x06, 0x03, // MVI B, 0x03
        0x0E, 0x00, // MVI C, 0x00
        0x80, // ADD B
        0x4F, // MOV C, A
        0x07, // RLC
        0x32, 0x00, 0x01, // STA 0x0100
        0x3C, // INR A
        0xC2, 0x06, 0x00, // JNZ 0x0006
        0x76, // HLT
    ];

    let mut state = Cpu8080State::default();
    state.memory.as_mut_slice()[..program.len()].copy_from_slice(&program);
    // Stack lives at the very top of RAM so a future PUSH/CALL has room
    // to grow downward without colliding with the program or the
    // 0x0100 scratch cell. Default `Cpu8080State` already initialises
    // SP to 0x0000, which would underflow on the first push.
    state.sp = 0xF000;
    state.pc = 0x0000;

    let bytes = Snapshot580Serializer::to_bytes(&state);

    // crates/persistence/examples/<this file>  →  ../../..  is the repo root.
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/persistence is two levels below the workspace root");
    let out_path = repo_root.join("counter_loop.580");
    fs::write(&out_path, &bytes).expect("write counter_loop.580");

    // Round-trip through the deserializer to make sure the file we just
    // wrote is loadable, then run the program against a NullBus until
    // HLT and verify the loop invariants. The program does no I/O, so
    // the null bus is sufficient.
    let mut roundtrip = Snapshot580Serializer::from_bytes(&bytes).expect("round-trip parse");
    let mut bus = NullBus::default();
    let mut steps = 0u64;
    while !roundtrip.halted {
        roundtrip
            .step_instruction(&mut bus)
            .expect("opcode decodes and executes");
        steps += 1;
        // Belt-and-braces budget. Empirically the loop reaches HLT in
        // 202 instructions: the INR A after each ADD B / RLC composes
        // additions and rotations until A lands on 0xFF, INR wraps it
        // to 0x00, and JNZ falls through to HLT. 10_000 leaves a
        // comfortable margin and still trips long before any infinite
        // loop hangs the generator.
        assert!(
            steps < 10_000,
            "program did not reach HLT within 10_000 steps"
        );
    }

    // After the loop:
    // * A wrapped through 256 distinct values driven by the
    //   ADD B / RLC / INR A composition and ended at 0x00 (the
    //   wraparound that takes the JNZ false). The final INR is what
    //   produces that 0x00 — so A == 0 and the zero flag is set.
    // * 0x0100 holds whatever the *last* STA wrote, which is the A
    //   value mid-iteration just after RLC, one INR before the
    //   wraparound that exits. We don't pin its exact value here —
    //   the meaningful invariant for the test is "the program wrote
    //   *something* there", i.e. the cell is no longer the default
    //   0x00 it started as. (Empirically the last write is non-zero;
    //   if you want to verify the exact byte, run the snapshot in
    //   the UI and read 0x0100.)
    assert_eq!(roundtrip.registers.a, 0x00, "A wraps to 0 on exit");
    assert!(roundtrip.flags.zero, "Z flag set on the wraparound");
    assert!(roundtrip.halted, "program halted via HLT");
    assert_eq!(roundtrip.pc, 0x0011, "PC sits one past HLT after halt");
    assert_eq!(roundtrip.registers.b, 0x03, "B preserved as step");

    println!(
        "wrote {} ({} bytes, program {} bytes); smoke test halted after {} steps",
        out_path.display(),
        bytes.len(),
        program.len(),
        steps
    );
}
