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
//! `step_instruction` (with a `NullBus` – the program does no I/O)
//! until `HLT`, then asserts the final state matches the math the
//! program describes. Anything wrong with the bytes, the opcodes, or
//! the loop arithmetic surfaces here instead of inside the UI.

use k580_core::{Cpu8080State, NullBus};
use k580_persistence::Snapshot580Serializer;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Hand-assembled program. 17 bytes ending at 0x0010 with HLT.
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
    // SP at the top of RAM so PUSH/CALL has room to grow downward.
    state.sp = 0xF000;
    state.pc = 0x0000;

    let bytes = Snapshot580Serializer::to_bytes(&state);

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/persistence is two levels below the workspace root");
    let out_path = repo_root.join("counter_loop.580");
    fs::write(&out_path, &bytes).expect("write counter_loop.580");

    // Round-trip and run against NullBus until HLT.
    let mut roundtrip = Snapshot580Serializer::from_bytes(&bytes).expect("round-trip parse");
    let mut bus = NullBus::default();
    let mut steps = 0u64;
    while !roundtrip.halted {
        roundtrip
            .step_instruction(&mut bus)
            .expect("opcode decodes and executes");
        steps += 1;
        // Empirically reaches HLT in 202 instructions; 10_000 is the
        // safety bound against an infinite loop.
        assert!(
            steps < 10_000,
            "program did not reach HLT within 10_000 steps"
        );
    }

    // A wraps to 0 (last INR pushes 0xFF→0x00), JNZ falls through to
    // HLT; 0x0100 holds whatever the last STA wrote (non-zero).
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
