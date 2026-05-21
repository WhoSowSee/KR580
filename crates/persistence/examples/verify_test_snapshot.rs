//! Loads `test_program.580` written by `make_test_snapshot` and runs the
//! program through the core, asserting the post-HLT state matches the
//! values promised in `make_test_snapshot.rs`. Pure smoke test for the
//! generated snapshot — not part of the workspace test target because
//! the file is generated on demand.

use std::path::PathBuf;

use k580_core::Cpu8080State;
use k580_devices::IoBus;
use k580_persistence::Snapshot580Serializer;

fn main() -> std::io::Result<()> {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()?;
    let path = workspace_root.join("test_program.580");
    let bytes = std::fs::read(&path)?;
    let mut state: Cpu8080State =
        Snapshot580Serializer::from_bytes(&bytes).expect("snapshot decode");
    let mut bus = IoBus::default();
    let executed = state
        .run_until_halt(&mut bus, 100)
        .expect("program runs to halt");

    println!("executed instructions: {executed}");
    println!(
        "A={:02X} B={:02X} PC={:04X} halted={} cycles={}",
        state.registers.a, state.registers.b, state.pc, state.halted, state.cycle_count
    );
    println!("flags: {:?}", state.flags);
    println!("RAM[2000h] = {:02X}", state.memory.read(0x2000));

    assert_eq!(state.registers.a, 0x46, "A should be 0x0F+0x37 = 0x46");
    assert_eq!(state.registers.b, 0x37, "B should be 0x37");
    assert_eq!(state.pc, 0x0009, "PC should land just after HLT");
    assert!(state.halted, "CPU should be halted");
    assert_eq!(state.memory.read(0x2000), 0x46, "STA wrote 0x46 to 0x2000");
    assert!(
        state.flags.auxiliary_carry,
        "AC set: 0x0F + 0x07 nibble overflow"
    );
    assert!(!state.flags.parity, "P clear: 0x46 has odd parity");
    assert!(!state.flags.sign);
    assert!(!state.flags.zero);
    assert!(!state.flags.carry);
    assert_eq!(state.cycle_count, 38, "T-states: 7+7+4+13+7 = 38");
    println!("OK");
    Ok(())
}
