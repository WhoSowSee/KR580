use super::{DesktopApp, Message, ShutdownState};
use crate::app::{BackendAction, PendingRequest};
use crate::backend::{AppError, AppEvent, RequestId};
use crate::runtime::file_work::{FileRequest, FileResult};
use std::time::{Duration, Instant};

fn app() -> DesktopApp {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    crate::app::test_support::settle_backend(&mut app);
    app.shell.main_window_id = Some(iced::window::Id::unique());
    app.mark_saved();
    app
}

fn await_close_result(app: &mut DesktopApp, expected: ShutdownState) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while app.shell.shutdown != expected {
        let _task = app.update(Message::Tick);
        assert!(
            Instant::now() < deadline,
            "close did not reach {expected:?}"
        );
        std::thread::yield_now();
    }
}

#[test]
fn close_waits_for_file_work_and_handles_its_result() {
    for (failed, expected) in [(false, ShutdownState::Ready), (true, ShutdownState::Idle)] {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("accepted-output.bin");
        let output = path.clone();
        let mut app = app();
        let main = app.shell.main_window_id;
        let (release, wait) = crossbeam_channel::bounded(1);
        app.queue_file_work(FileRequest::MonitorSaved(path.clone()), move || {
            wait.recv().unwrap();
            if failed {
                return Err(AppError::Io("injected write failure".into()));
            }
            crate::persistence::write_file_atomic(&output, b"accepted output")?;
            Ok(FileResult::Saved(output))
        });
        let _task = app.update(Message::WindowClose);
        assert_eq!(app.shell.shutdown, ShutdownState::Draining);
        assert_eq!(app.shell.main_window_id, main);
        assert!(!path.exists());
        let _task = app.update(Message::ToggleRun);
        assert!(!app.execution.running);
        let _task = app.update(Message::WindowClose);
        assert_eq!(app.shell.shutdown, ShutdownState::Draining);
        release.send(()).unwrap();
        await_close_result(&mut app, expected);
        if !failed {
            assert_eq!(std::fs::read(path).unwrap(), b"accepted output");
        }
        assert!(app.requests.pending_requests.is_empty());
        assert_eq!(app.shell.main_window_id, main);
        assert_eq!(app.shell.error_notice.is_some(), failed);
    }
}

#[test]
fn backend_flush_failure_cancels_close() {
    let mut app = app();
    let main = app.shell.main_window_id;
    let id = RequestId(99_999);
    app.shell.shutdown = ShutdownState::Pending;
    app.requests.pending_requests.insert(
        id,
        PendingRequest::Command {
            action: BackendAction::Shutdown,
        },
    );
    let error = AppError::Io("injected flush failure".into());
    app.consume_event(AppEvent::ErrorRaised(error.clone()));
    app.consume_event(AppEvent::CommandFinished {
        id,
        result: Err(error),
    });
    assert_eq!(app.shell.shutdown, ShutdownState::Idle);
    assert_eq!(app.shell.main_window_id, main);
    assert!(app.shell.error_notice.is_some());
}

#[test]
fn dirty_close_still_requires_discard_confirmation() {
    let mut app = app();
    app.document.dirty = true;
    let main = app.shell.main_window_id.unwrap();
    let _task = app.update(Message::WindowCloseRequested(main));
    assert_eq!(app.shell.shutdown, ShutdownState::Idle);
    assert!(app.document.pending_action.is_some());
}
