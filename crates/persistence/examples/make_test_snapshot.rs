//! One-shot helper that emits `test_program.580` at the workspace root.
//!
//! Run with `cargo run --example make_test_snapshot -p k580-persistence`.
//!
//! The encoded program is:
//!
//! ```text
//! 0000  3E 0F        MVI A, 0Fh
//! 0002  06 37        MVI B, 37h
//! 0004  80           ADD B           ; A <- 46h, AC=1, S=0, Z=0, P=0, CY=0
//! 0005  32 00 20     STA 2000h
//! 0008  76           HLT
//! ```
//!
//! 0x0F + 0x37 = 0x46. Low nibble overflows (F + 7 = 0x16) so AC is
//! set; the result has three set bits (0100 0110b), so parity is odd
//! and P stays clear. CY/S/Z all clear. AC alone is enough to show
//! that flag updates are wired up — and the user can then single-step
//! to see Z/CY light up on other bytes.
//!
//! A non-zero "canary" byte is preloaded at 0x2000 (the STA target)
//! so the user can see the byte change after running the program —
//! distinguishing "STA wrote here" from "this address happened to be
//! zero anyway".
//!
//! After loading and pressing the run button the user should see:
//!   * A = 46h, B = 37h, PC = 0009h, halted = true
//!   * RAM[2000h] = 46h (was CCh before run)
//!   * flags: AC; S/Z/P/CY = 0
//!   * cycle_count = 38 (MVI 7 + MVI 7 + ADD 4 + STA 13 + HLT 7)
//!
//! Tact-stepping through it is also a good smoke test for the timing
//! engine.
//!
//! The output path is workspace-root–relative so `cargo run` from any
//! crate places the file where the user can see it next to `Cargo.toml`.

use std::path::PathBuf;

use k580_core::Cpu8080State;
use k580_persistence::Snapshot580Serializer;

fn main() -> std::io::Result<()> {
    let program: [u8; 9] = [
        0x3E, 0x0F, // MVI A, 0Fh
        0x06, 0x37, // MVI B, 37h
        0x80, // ADD B
        0x32, 0x00, 0x20, // STA 2000h
        0x76, // HLT
    ];

    let mut state = Cpu8080State::default();
    for (offset, byte) in program.iter().enumerate() {
        state.memory.write(offset as u16, *byte);
    }
    // Canary: shows up in the memory view before the program runs and
    // gets overwritten with 0x46 once STA fires.
    state.memory.write(0x2000, 0xCC);
    state.pc = 0x0000;
    state.sp = 0xFFFF;

    let bytes = Snapshot580Serializer::to_bytes(&state);

    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()?;
    let target = workspace_root.join("test_program.580");
    std::fs::write(&target, bytes)?;
    println!("wrote {} ({} bytes)", target.display(), program.len());
    Ok(())
}
