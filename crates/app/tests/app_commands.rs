use k580_app::{AppCommand, AppEvent, Emulator, RunMode, spawn_emulator};
use k580_core::{Cpu8080State, RegisterName};
use k580_persistence::{ExportOptions, ExportTextSection, ExportXlsxPage};
use std::path::PathBuf;
use std::time::Duration;

#[test]
fn emulator_commands_mutate_only_through_command_api() {
    let mut emulator = Emulator::default();
    emulator.handle_command(AppCommand::SetRegister(RegisterName::A, 0x42));
    emulator.handle_command(AppCommand::SetMemory(0, 0x76));
    let events = emulator.handle_command(AppCommand::StepInstruction);
    assert!(
        events
            .iter()
            .any(|event| matches!(event, AppEvent::InstructionBoundaryReached(_)))
    );
    assert!(emulator.cpu().halted);
}

#[test]
fn actor_publishes_state_changes() {
    let handle = spawn_emulator();
    std::thread::sleep(Duration::from_millis(20));
    handle
        .send(AppCommand::SetRegister(RegisterName::B, 0x33))
        .unwrap();
    std::thread::sleep(Duration::from_millis(20));
    let events = handle.drain_events();
    assert!(events.iter().any(|event| matches!(event, AppEvent::StateChanged(snapshot) if snapshot.cpu.registers.b == 0x33)));
    handle.send(AppCommand::Shutdown).unwrap();
}

#[test]
fn actor_paced_run_progresses_when_step_interval_exceeds_device_poll() {
    let handle = spawn_emulator();
    std::thread::sleep(Duration::from_millis(20));
    handle.drain_events();

    handle.send(AppCommand::SetMemory(0, 0x00)).unwrap();
    handle
        .send(AppCommand::SetStepInterval(Duration::from_millis(200)))
        .unwrap();
    handle.send(AppCommand::SetRunMode(RunMode::Paced)).unwrap();
    handle.send(AppCommand::Run).unwrap();

    std::thread::sleep(Duration::from_millis(650));
    let events = handle.drain_events();
    handle.send(AppCommand::Shutdown).unwrap();

    assert!(
        events
            .iter()
            .any(|event| matches!(event, AppEvent::InstructionBoundaryReached(_))),
        "paced run must not be starved by the 50 ms device poll, got {events:?}"
    );
}

/// Regression for "program keeps running after a register reset":
/// clicking ResetCpu while the actor is in `Run` must flip `running` to false *and* publish `Stopped` so the UI's
/// play/pause toggle returns to its idle state. Same contract for
/// ResetRam – wiping code under a running worker would otherwise
/// keep stepping into zero bytes (NOPs) until the per-session budget
/// hit. We hit the path directly through `Emulator` rather than the
/// actor so the test stays deterministic and doesn't depend on
/// timer-quantum effects.
#[test]
fn reset_cpu_during_run_emits_stopped() {
    let mut emulator = Emulator::default();
    // Fill memory with NOPs so `Run` doesn't immediately hit HLT.
    emulator.handle_command(AppCommand::SetMemory(0, 0x00));
    emulator.handle_command(AppCommand::Run);
    assert!(emulator.is_running());

    let events = emulator.handle_command(AppCommand::ResetCpu);
    assert!(
        !emulator.is_running(),
        "ResetCpu must clear the running flag"
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, AppEvent::Stopped)),
        "ResetCpu during Run must publish Stopped, got {events:?}"
    );
}

#[test]
fn reset_ram_during_run_emits_stopped() {
    let mut emulator = Emulator::default();
    emulator.handle_command(AppCommand::SetMemory(0, 0x00));
    emulator.handle_command(AppCommand::Run);
    assert!(emulator.is_running());

    let events = emulator.handle_command(AppCommand::ResetRam);
    assert!(
        !emulator.is_running(),
        "ResetRam must clear the running flag"
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, AppEvent::Stopped)),
        "ResetRam during Run must publish Stopped, got {events:?}"
    );
}

/// Counter-case: ResetCpu while the worker is *idle* must not
/// fabricate a `Stopped` event – the UI consumes `Stopped` to flip
/// the play/pause toggle, and a spurious one would briefly flash
/// the "stopped" status in a session that was never running.
#[test]
fn reset_cpu_while_idle_does_not_emit_stopped() {
    let mut emulator = Emulator::default();
    let events = emulator.handle_command(AppCommand::ResetCpu);
    assert!(!emulator.is_running());
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, AppEvent::Stopped)),
        "ResetCpu on an idle emulator must not publish Stopped, got {events:?}"
    );
}

/// `ApplyCpuState` is the worker side of the UI undo/redo timeline:
/// the UI captures a `Cpu8080State` before a mutation, lets the user
/// keep editing, and on Ctrl+Z replays the captured snapshot through
/// this command. Two contracts to verify here.
///
/// 1. **Replace, not merge.** Every observable field on the CPU must
///    match the snapshot byte-for-byte after the command – registers,
///    PC, the full 64 KiB of RAM, halt bit, the lot. A partial swap
///    would leak post-mutation state into the rewound view, which is
///    exactly the kind of "Ctrl+Z half-worked" surprise the timeline
///    exists to prevent.
/// 2. **Idle counterpart.** Mirroring `reset_cpu_while_idle_does_not_emit_stopped`,
///    rewinding a CPU that was never running must not fabricate a
///    `Stopped` event. Otherwise every Ctrl+Z would briefly flash
///    "Stopped" in the status bar and toggle the cosmetic running
///    flag for no reason.
#[test]
fn apply_cpu_state_replaces_full_snapshot() {
    let mut emulator = Emulator::default();
    let mut snapshot = Cpu8080State::default();
    snapshot.registers.set(RegisterName::A, 0xAB);
    snapshot.registers.set(RegisterName::B, 0xCD);
    snapshot.pc = 0x1234;
    snapshot.sp = 0xFFEE;
    snapshot.memory.write(0x0100, 0x76); // HLT byte at a known address
    snapshot.memory.write(0xBEEF, 0x42);

    let events = emulator.handle_command(AppCommand::ApplyCpuState(Box::new(snapshot.clone())));

    assert_eq!(
        emulator.cpu(),
        &snapshot,
        "ApplyCpuState must replace CPU verbatim"
    );
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, AppEvent::Stopped)),
        "ApplyCpuState on an idle emulator must not publish Stopped, got {events:?}"
    );
}

/// Symmetrical to `reset_cpu_during_run_emits_stopped`: when the user
/// hits Ctrl+Z while a run is armed, the worker has to publish
/// `Stopped` so the play/pause icon snaps back to the idle state.
/// Without this the UI would keep showing the red pause glyph and
/// `Message::Tick` would keep chasing PC even though the worker
/// already disarmed itself.
#[test]
fn apply_cpu_state_during_run_emits_stopped() {
    let mut emulator = Emulator::default();
    emulator.handle_command(AppCommand::SetMemory(0, 0x00));
    emulator.handle_command(AppCommand::Run);
    assert!(emulator.is_running());

    let events = emulator.handle_command(AppCommand::ApplyCpuState(Box::default()));

    assert!(
        !emulator.is_running(),
        "ApplyCpuState must clear the running flag"
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, AppEvent::Stopped)),
        "ApplyCpuState during Run must publish Stopped, got {events:?}"
    );
}

/// Rewinding past a HLT must clear the halt notice on the UI side.
/// The contract: when the restored snapshot has `halted == false`,
/// the worker emits `HaltStateChanged(false)` so the floating "CPU
/// halted" frame disappears as soon as the user undoes the
/// instruction that halted the CPU.
#[test]
fn apply_cpu_state_unhalts_emits_halt_event() {
    let mut emulator = Emulator::default();
    // Drive the CPU into a HLT first so its current state has the
    // halt bit set; then ApplyCpuState a default (un-halted) snapshot.
    emulator.handle_command(AppCommand::SetMemory(0, 0x76));
    emulator.handle_command(AppCommand::StepInstruction);
    assert!(emulator.cpu().halted, "test setup expects HLT to halt CPU");

    let events = emulator.handle_command(AppCommand::ApplyCpuState(Box::default()));

    assert!(
        events
            .iter()
            .any(|event| matches!(event, AppEvent::HaltStateChanged(false))),
        "ApplyCpuState rewinding past HLT must emit HaltStateChanged(false), got {events:?}"
    );
    assert!(!emulator.cpu().halted);
}

#[test]
fn export_txt_with_options_writes_all_text_sections() {
    let mut emulator = Emulator::default();
    emulator.handle_command(AppCommand::SetMemory(0x0100, 0xAA));
    emulator.handle_command(AppCommand::SetMemory(0x0200, 0xBB));
    let path = unique_temp_file("k580-text-sections.txt");
    let options = ExportOptions {
        text_sections: vec![
            text_section("Подпрограмма 1", 0x0100, 0x0100),
            text_section("Подпрограмма 2", 0x0200, 0x0200),
        ],
        ..ExportOptions::default()
    };

    let events = emulator.handle_command(AppCommand::ExportTxtWithOptions(path.clone(), options));
    let text = std::fs::read_to_string(&path).unwrap();

    assert!(
        !events
            .iter()
            .any(|event| matches!(event, AppEvent::ErrorRaised(_)))
    );
    assert!(text.contains("[Подпрограмма 1]\n[Registers]\n\n[Flags]\n\n[Memory]\n0100=AA\n"));
    assert!(text.contains("[Подпрограмма 2]\n[Registers]\n\n[Flags]\n\n[Memory]\n0200=BB\n"));
    std::fs::remove_file(path).ok();
}

#[test]
fn export_xlsx_with_options_accepts_all_excel_pages() {
    let mut emulator = Emulator::default();
    emulator.handle_command(AppCommand::SetMemory(0x0100, 0xAA));
    emulator.handle_command(AppCommand::SetMemory(0x0200, 0xBB));
    let path = unique_temp_file("k580-excel-pages.xlsx");
    let options = ExportOptions {
        xlsx_pages: vec![
            xlsx_page("Подпрограмма 1", 0x0100, 0x0100),
            xlsx_page("Подпрограмма 2", 0x0200, 0x0200),
        ],
        ..ExportOptions::default()
    };

    let events = emulator.handle_command(AppCommand::ExportXlsxWithOptions(path.clone(), options));

    assert!(
        !events
            .iter()
            .any(|event| matches!(event, AppEvent::ErrorRaised(_)))
    );
    assert!(std::fs::metadata(&path).unwrap().len() > 0);
    std::fs::remove_file(path).ok();
}

#[test]
fn import_xlsx_sheet_applies_only_selected_sheet() {
    let first = k580_persistence::ExportModel {
        registers: Vec::new(),
        flags: Vec::new(),
        memory: vec![(0x0100, 0xAA)],
    };
    let second = k580_persistence::ExportModel {
        registers: Vec::new(),
        flags: Vec::new(),
        memory: vec![(0x0200, 0xBB)],
    };
    let path = unique_temp_file("k580-import-pages.xlsx");
    k580_persistence::Exporters::write_xlsx_pages(
        &path,
        &[
            ("Подпрограмма 1".to_owned(), first, ExportOptions::default()),
            (
                "Подпрограмма 2".to_owned(),
                second,
                ExportOptions::default(),
            ),
        ],
    )
    .unwrap();
    let mut emulator = Emulator::default();

    let events = emulator.handle_command(AppCommand::ImportXlsxSheet(
        path.clone(),
        "Подпрограмма 2".to_owned(),
    ));

    assert!(
        !events
            .iter()
            .any(|event| matches!(event, AppEvent::ErrorRaised(_)))
    );
    assert_eq!(emulator.cpu().memory.read(0x0100), 0x00);
    assert_eq!(emulator.cpu().memory.read(0x0200), 0xBB);
    std::fs::remove_file(path).ok();
}

#[test]
fn import_txt_section_applies_only_selected_section() {
    let first = k580_persistence::ExportModel {
        registers: Vec::new(),
        flags: Vec::new(),
        memory: vec![(0x0100, 0xAA)],
    };
    let second = k580_persistence::ExportModel {
        registers: Vec::new(),
        flags: Vec::new(),
        memory: vec![(0x0200, 0xBB)],
    };
    let path = unique_temp_file("k580-import-sections.txt");
    std::fs::write(
        &path,
        k580_persistence::Exporters::to_text_sections(&[
            ("Раздел 1".to_owned(), first),
            ("Раздел 2".to_owned(), second),
        ]),
    )
    .unwrap();
    let mut emulator = Emulator::default();

    let events = emulator.handle_command(AppCommand::ImportTxtSection(
        path.clone(),
        "Раздел 2".to_owned(),
    ));

    assert!(
        !events
            .iter()
            .any(|event| matches!(event, AppEvent::ErrorRaised(_)))
    );
    assert_eq!(emulator.cpu().memory.read(0x0100), 0x00);
    assert_eq!(emulator.cpu().memory.read(0x0200), 0xBB);
    std::fs::remove_file(path).ok();
}

fn text_section(name: &str, memory_start: u16, memory_end: u16) -> ExportTextSection {
    ExportTextSection {
        name: name.to_owned(),
        memory_start,
        memory_end,
        include_memory_address: true,
        include_memory_value: true,
        include_memory_command: false,
        registers: Vec::new(),
        flags: Vec::new(),
    }
}

fn xlsx_page(name: &str, memory_start: u16, memory_end: u16) -> ExportXlsxPage {
    ExportXlsxPage {
        name: name.to_owned(),
        memory_start,
        memory_end,
        include_memory_address: true,
        include_memory_value: true,
        include_memory_command: false,
        include_comment_column: false,
        registers: Vec::new(),
        flags: Vec::new(),
    }
}

fn unique_temp_file(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{nanos}-{name}"))
}
