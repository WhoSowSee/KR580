use super::*;
use crate::app::DesktopApp;
use crate::backend::error::AppErrorKind;
use std::time::Duration;

#[test]
fn queued_save_keeps_ui_responsive_and_reports_success_after_write() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    let path = std::env::temp_dir().join(format!("kr580-delayed-save-{}.png", std::process::id()));
    let (entered_tx, entered_rx) = std::sync::mpsc::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let saved_path = path.clone();
    app.queue_file_work(FileRequest::MonitorSaved(path.clone()), move || {
        entered_tx.send(()).unwrap();
        release_rx.recv().unwrap();
        crate::persistence::write_file_atomic(&saved_path, b"image")?;
        Ok(FileResult::Saved(saved_path))
    });
    entered_rx.recv_timeout(Duration::from_secs(3)).unwrap();
    let _task = app.update(Message::MemoryAddressChanged("1234".into()));
    assert_eq!(app.memory.memory_address_input, "1234");
    assert!(!matches!(
        app.shell.status_kind,
        crate::app::StatusKind::MonitorImageSaved { .. }
    ));
    release_tx.send(()).unwrap();
    crate::app::test_support::settle_files(&mut app);
    assert!(matches!(
        app.shell.status_kind,
        crate::app::StatusKind::MonitorImageSaved { .. }
    ));
    assert_eq!(std::fs::read(&path).unwrap(), b"image");
    crate::app::test_support::settle_backend(&mut app);
    app.snapshot.devices.hdd.path = Some(path.clone());
    app.panels.hdd_generation = 2;
    app.apply_file_completion(FileCompletion {
        request: FileRequest::HddDeleted {
            path: path.clone(),
            generation: 1,
        },
        result: Ok(FileResult::Deleted),
    });
    assert!(app.requests.pending_requests.is_empty());
    std::fs::remove_file(path).unwrap();
}

#[test]
fn capacity_counts_running_queued_and_unconsumed_completions() {
    let mut worker = FileWorker::new().unwrap();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let _task = worker
        .enqueue(FileRequest::FloppySaved, move || {
            release_rx.recv().unwrap();
            Ok(FileResult::Saved("first".into()))
        })
        .unwrap_or_else(|_| panic!("first job refused"));
    for index in 1..CAPACITY {
        let _task = worker
            .enqueue(FileRequest::FloppySaved, move || {
                Ok(FileResult::Saved(index.to_string().into()))
            })
            .unwrap_or_else(|_| panic!("queued job refused"));
    }
    let Err(completion) = worker.enqueue(FileRequest::FloppySaved, || panic!("refused job ran"))
    else {
        panic!("capacity exceeded")
    };
    assert_eq!(
        completion.result.err().unwrap().kind(),
        AppErrorKind::DeviceBusy
    );
    release_tx.send(()).unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    let mut paths = Vec::new();
    while paths.len() < CAPACITY {
        if let Some(completion) = worker.take_completion() {
            let Ok(FileResult::Saved(path)) = completion.result else {
                panic!("job failed")
            };
            paths.push(path);
        } else {
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    assert_eq!(paths[0], std::path::Path::new("first"));
    assert_eq!(paths[CAPACITY - 1], std::path::Path::new("7"));
    assert_eq!(worker.pending(), 0);
}
