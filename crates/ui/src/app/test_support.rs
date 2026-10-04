use super::{DesktopApp, Message};
use std::time::{Duration, Instant};

pub(crate) fn settle_backend(app: &mut DesktopApp) {
    app.dispatch_request(crate::backend::AppCommand::RequestSnapshot);
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        app.pull_events();
        let _task = app.update(Message::Tick);
        settle_files(app);
        if app.requests.pending_requests.is_empty() {
            return;
        }
        assert!(Instant::now() < deadline, "backend requests did not finish");
        std::thread::sleep(Duration::from_millis(1));
    }
}

pub(crate) fn settle_files(app: &mut DesktopApp) {
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        app.pull_file_completions();
        if app
            .requests
            .file_worker
            .as_ref()
            .is_none_or(|worker| worker.pending() == 0)
        {
            return;
        }
        assert!(Instant::now() < deadline, "UI file work did not finish");
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn cpu_edit_confirmed_after_eighty_ms_has_correct_undo() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    settle_backend(&mut app);
    app.dispatch_with_undo(crate::backend::AppCommand::SetMemory(0x1234, 0xAA));
    assert!(app.document.undo_stack.pop_undo().is_none());
    std::thread::sleep(Duration::from_millis(80));
    settle_backend(&mut app);
    assert_eq!(app.snapshot.cpu.memory.read(0x1234), 0xAA);
    let _task = app.apply_undo();
    settle_backend(&mut app);
    assert_eq!(app.snapshot.cpu.memory.read(0x1234), 0);
    assert!(!app.document.dirty);
}
