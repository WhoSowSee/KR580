use super::*;
use crate::backend::{ChangeDirection, CommandResult};
use k580_core::RegisterName;

#[test]
fn delayed_worker_returns_each_requests_actual_cpu_transition() {
    let (command_tx, command_rx) = crossbeam_channel::bounded(256);
    let (event_tx, event_rx) = crossbeam_channel::bounded(256);
    let (critical_tx, critical_rx) = crossbeam_channel::bounded(1024);
    let mailbox = Arc::new(Mutex::new(None));
    let handle = EmulatorHandle {
        command_tx,
        event_rx,
        critical_rx,
        state_mailbox: mailbox.clone(),
        next_request_id: AtomicU64::new(1),
        stopped_reported: AtomicBool::new(false),
    };
    let (begin, wait) = crossbeam_channel::bounded(1);
    let worker = thread::spawn(move || {
        wait.recv().unwrap();
        run_worker(command_rx, event_tx, critical_tx, mailbox);
    });
    let first = handle
        .send_request(AppCommand::Edit(Box::new(AppCommand::SetRegister(
            RegisterName::A,
            0x41,
        ))))
        .unwrap();
    let second = handle
        .send_request(AppCommand::Edit(Box::new(AppCommand::SetRegister(
            RegisterName::A,
            0x42,
        ))))
        .unwrap();
    assert!(matches!(
        handle.critical_rx.recv_timeout(Duration::from_millis(80)),
        Err(crossbeam_channel::RecvTimeoutError::Timeout)
    ));
    begin.send(()).unwrap();
    let events = handle.drain_until_request_finished(second, Duration::from_secs(2));
    let changes: Vec<_> = events
        .into_iter()
        .filter_map(|event| match event {
            AppEvent::CommandFinished {
                id,
                result: Ok(CommandResult::CpuChanged { revision, change }),
            } => Some((id, revision, change)),
            _ => None,
        })
        .collect();
    assert_eq!(changes.len(), 2);
    assert_eq!(changes[0].0, first);
    assert_eq!(changes[1].0, second);
    assert!(changes[0].1 < changes[1].1);
    assert_eq!(changes[0].2.before().registers.a, 0);
    assert_eq!(changes[1].2.before().registers.a, 0x41);
    let (undo, _) = changes[1].2.replay(ChangeDirection::Undo);
    assert_eq!(undo.registers.a, 0x41);
    handle.send(AppCommand::Shutdown).unwrap();
    worker.join().unwrap();
}
