use k580_core::RegisterName;
use k580_ui::backend::{AppCommand, AppEvent, CommandResult, spawn_emulator};
use std::time::Duration;

#[test]
fn actor_completes_a_request_after_publishing_its_state() {
    let handle = spawn_emulator();
    let request = handle
        .send_request(AppCommand::SetRegister(RegisterName::B, 0x33))
        .unwrap();
    let events = handle.drain_until_request_finished(request, Duration::from_secs(1));
    assert!(events.iter().any(|event| matches!(
        event,
        AppEvent::StateChanged(snapshot) if snapshot.cpu.registers.b == 0x33
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        AppEvent::CommandFinished {
            id,
            result: Ok(CommandResult::Completed),
        } if *id == request
    )));
    handle.send(AppCommand::Shutdown).unwrap();
}

#[test]
fn actor_runs_program_save_outside_the_emulator_loop() {
    let path = std::env::temp_dir().join(format!("k580-request-{}.580", std::process::id()));
    let handle = spawn_emulator();
    let request = handle
        .send_request(AppCommand::SaveProgram(path.clone()))
        .unwrap();
    let events = handle.drain_until_request_finished(request, Duration::from_secs(1));
    assert!(events.iter().any(|event| matches!(
        event,
        AppEvent::CommandFinished {
            id,
            result: Ok(CommandResult::SavedProgram),
        } if *id == request
    )));
    assert_eq!(std::fs::metadata(&path).unwrap().len(), 65_549);
    handle.send(AppCommand::Shutdown).unwrap();
    std::fs::remove_file(path).ok();
}

#[test]
fn successive_saves_to_one_path_keep_the_last_accepted_state() {
    let path = std::env::temp_dir().join(format!("k580-save-order-{}.580", std::process::id()));
    let handle = spawn_emulator();
    handle
        .send(AppCommand::SetRegister(RegisterName::A, 0x11))
        .unwrap();
    handle
        .send_request(AppCommand::SaveProgram(path.clone()))
        .unwrap();
    handle
        .send(AppCommand::SetRegister(RegisterName::A, 0x22))
        .unwrap();
    let last = handle
        .send_request(AppCommand::SaveProgram(path.clone()))
        .unwrap();
    let events = handle.drain_until_request_finished(last, Duration::from_secs(3));
    assert!(events.iter().any(|event| matches!(event, AppEvent::CommandFinished { id, result: Ok(CommandResult::SavedProgram) } if *id == last)));
    assert_eq!(
        k580_ui::persistence::ProgramSerializer::load_file(&path)
            .unwrap()
            .registers
            .a,
        0x22
    );
    handle.send(AppCommand::Shutdown).unwrap();
    std::fs::remove_file(path).unwrap();
}
