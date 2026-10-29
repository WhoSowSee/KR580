use crate::backend::{AppCommand, AppEvent, CommandResult, spawn_emulator};
use crate::persistence::ProgramSerializer;
use std::time::{Duration, Instant};

#[test]
fn request_shutdown_finishes_prior_file_jobs_and_stops_actor() {
    let directory = tempfile::tempdir().unwrap();
    let program = directory.path().join("saved.580");
    let floppy = directory.path().join("floppy.kpd");
    let handle = spawn_emulator();
    let attach = handle
        .send_request(AppCommand::AttachFloppyImage(floppy.clone()))
        .unwrap();
    let save = handle
        .send_request(AppCommand::SaveProgram(program.clone()))
        .unwrap();
    let shutdown = handle.send_request(AppCommand::Shutdown).unwrap();
    let events = handle.drain_until_request_finished(shutdown, Duration::from_secs(5));
    for request in [attach, save] {
        assert!(
            events.iter().any(|event| matches!(event,
                AppEvent::CommandFinished { id, result: Ok(_) } if *id == request
            )),
            "missing successful receipt for {request:?}"
        );
    }
    assert!(events.iter().any(|event| matches!(event,
        AppEvent::CommandFinished { id, result: Ok(CommandResult::Completed) } if *id == shutdown
    )));
    assert!(ProgramSerializer::load_file(program).is_ok());
    assert!(floppy.is_file());
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if handle
            .drain_events()
            .iter()
            .any(|event| matches!(event, AppEvent::WorkerStopped))
        {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "actor did not stop after Shutdown receipt"
        );
        std::thread::yield_now();
    }
}
