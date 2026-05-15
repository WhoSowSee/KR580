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
