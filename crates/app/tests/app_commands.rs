use k580_app::{AppCommand, AppEvent, Emulator, spawn_emulator};
use k580_core::RegisterName;
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

/// Regression for «программа не перестаёт выполняться при сбросе
/// регистров»: clicking ResetCpu while the actor is in `Run` must
/// flip `running` to false *and* publish `Stopped` so the UI's
/// play/pause toggle returns to its idle state. Same contract for
/// ResetRam — wiping code under a running worker would otherwise
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
/// fabricate a `Stopped` event — the UI consumes `Stopped` to flip
/// the play/pause toggle, and a spurious one would briefly flash
/// the «остановлено» status in a session that was never running.
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
