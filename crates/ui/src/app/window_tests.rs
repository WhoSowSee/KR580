use iced::{Size, window};

use super::windows::{detached_monitor_size, detached_storage_size};
use super::{DesktopApp, Message, ToolWindowKind};

#[test]
fn detached_monitor_matches_attached_dialog_size() {
    assert_eq!(
        detached_monitor_size(Size::new(1180.0, 720.0)),
        Size::new(1060.0, 600.0)
    );
}

#[test]
fn detached_storage_matches_attached_dialog_size() {
    assert_eq!(detached_storage_size(), Size::new(760.0, 340.0));
}

#[cfg(windows)]
#[test]
fn second_startup_frame_prepares_hidden_tool_windows() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.main_window_id = Some(window::Id::unique());
    app.startup_frames_seen = 1;

    let _task = app.update(Message::FrameRendered);

    assert!(app.monitor_window.id.is_some());
    assert!(app.floppy_window.id.is_some());
    assert!(app.hdd_window.id.is_some());
    assert!(app.network_window.id.is_some());
    assert!(!app.monitor_window.ready);
    assert!(!app.floppy_window.ready);
    assert!(!app.hdd_window.ready);
    assert!(!app.network_window.ready);
}

#[cfg(windows)]
#[test]
fn detaching_storage_reuses_prepared_native_windows() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    let floppy = window::Id::unique();
    let hdd = window::Id::unique();
    app.floppy_open = true;
    app.floppy_window.id = Some(floppy);
    app.floppy_window.ready = true;
    app.hdd_open = true;
    app.hdd_window.id = Some(hdd);
    app.hdd_window.ready = true;

    let _task = app.update(Message::DetachToolWindow(ToolWindowKind::Floppy));
    let _task = app.update(Message::DetachToolWindow(ToolWindowKind::Hdd));

    assert_eq!(app.floppy_window.id, Some(floppy));
    assert!(app.floppy_window.detached);
    assert_eq!(app.hdd_window.id, Some(hdd));
    assert!(app.hdd_window.detached);
}

#[cfg(windows)]
#[test]
fn detaching_monitor_reuses_prepared_native_window() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    let monitor = window::Id::unique();
    app.monitor_open = true;
    app.monitor_window.id = Some(monitor);
    app.monitor_window.ready = true;

    let _task = app.update(Message::DetachToolWindow(ToolWindowKind::Monitor));

    assert!(app.monitor_open);
    assert_eq!(app.monitor_window.id, Some(monitor));
    assert!(app.monitor_window.detached);
}

#[cfg(windows)]
#[test]
fn attaching_monitor_hides_native_window_and_restores_overlay() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    let monitor = window::Id::unique();
    app.monitor_open = true;
    app.monitor_window.id = Some(monitor);
    app.monitor_window.ready = true;
    app.monitor_window.detached = true;
    app.monitor_window.always_on_top = true;

    let _task = app.update(Message::AttachToolWindow(ToolWindowKind::Monitor));

    assert!(app.monitor_open);
    assert_eq!(app.monitor_window.id, Some(monitor));
    assert!(!app.monitor_window.detached);
    assert!(!app.monitor_window.always_on_top);
}

#[test]
fn detached_monitor_pin_toggles_always_on_top() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    let monitor = window::Id::unique();
    app.monitor_open = true;
    app.monitor_window.detached = true;
    app.monitor_window.ready = true;
    app.monitor_window.id = Some(monitor);

    let _task = app.update(Message::ToggleToolWindowAlwaysOnTop(
        ToolWindowKind::Monitor,
    ));

    assert!(app.monitor_window.always_on_top);
    assert!(app.monitor_open);
    assert!(app.monitor_window.detached);
    assert!(app.monitor_window.ready);
    assert_eq!(app.monitor_window.id, Some(monitor));

    let _task = app.update(Message::ToggleToolWindowAlwaysOnTop(
        ToolWindowKind::Monitor,
    ));

    assert!(!app.monitor_window.always_on_top);
    assert!(app.monitor_open);
    assert!(app.monitor_window.detached);
    assert!(app.monitor_window.ready);
    assert_eq!(app.monitor_window.id, Some(monitor));
}

#[test]
fn detached_storage_pin_and_attach_are_independent() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    let floppy = window::Id::unique();
    let hdd = window::Id::unique();
    app.floppy_open = true;
    app.floppy_window.id = Some(floppy);
    app.floppy_window.ready = true;
    app.floppy_window.detached = true;
    app.hdd_window.id = Some(hdd);
    app.hdd_window.ready = true;

    let _task = app.update(Message::ToggleToolWindowAlwaysOnTop(ToolWindowKind::Floppy));

    assert!(app.floppy_window.always_on_top);
    assert!(!app.hdd_window.always_on_top);

    let _task = app.update(Message::AttachToolWindow(ToolWindowKind::Floppy));

    assert!(app.floppy_open);
    assert!(!app.floppy_window.detached);
    assert!(!app.floppy_window.always_on_top);
    assert_eq!(app.floppy_window.id, Some(floppy));
    assert_eq!(app.hdd_window.id, Some(hdd));
}

#[test]
fn detached_network_pin_and_attach_are_independent() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    let network = window::Id::unique();
    app.network_open = true;
    app.network_window.id = Some(network);
    app.network_window.ready = true;
    app.network_window.detached = true;

    let _task = app.update(Message::ToggleToolWindowAlwaysOnTop(
        ToolWindowKind::Network,
    ));

    assert!(app.network_window.always_on_top);

    let _task = app.update(Message::AttachToolWindow(ToolWindowKind::Network));

    assert!(app.network_open);
    assert!(!app.network_window.detached);
    assert!(!app.network_window.always_on_top);
    assert_eq!(app.network_window.id, Some(network));
}

#[test]
fn closing_detached_monitor_does_not_close_main_window() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    let main = window::Id::unique();
    let monitor = window::Id::unique();
    app.main_window_id = Some(main);
    app.monitor_window.id = Some(monitor);
    app.monitor_window.ready = true;
    app.monitor_window.detached = true;
    app.monitor_open = true;

    let _task = app.update(Message::WindowCloseRequested(monitor));

    assert_eq!(app.main_window_id, Some(main));
    #[cfg(windows)]
    assert_eq!(app.monitor_window.id, Some(monitor));
    #[cfg(not(windows))]
    assert_eq!(app.monitor_window.id, None);
    assert!(!app.monitor_window.detached);
    assert!(!app.monitor_open);
}

#[test]
fn closing_detached_hdd_does_not_close_main_window() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    let main = window::Id::unique();
    let hdd = window::Id::unique();
    app.main_window_id = Some(main);
    app.hdd_window.id = Some(hdd);
    app.hdd_window.ready = true;
    app.hdd_window.detached = true;
    app.hdd_open = true;

    let _task = app.update(Message::WindowCloseRequested(hdd));

    assert_eq!(app.main_window_id, Some(main));
    #[cfg(windows)]
    assert_eq!(app.hdd_window.id, Some(hdd));
    #[cfg(not(windows))]
    assert_eq!(app.hdd_window.id, None);
    assert!(!app.hdd_window.detached);
    assert!(!app.hdd_open);
}

#[test]
fn opening_detached_monitor_does_not_replace_main_window_id() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    let main = window::Id::unique();
    let monitor = window::Id::unique();
    app.main_window_id = Some(main);
    app.monitor_window.id = Some(monitor);
    app.monitor_window.detached = true;

    let _task = app.update(Message::WindowOpened(monitor));

    assert_eq!(app.main_window_id, Some(main));
    assert_eq!(app.monitor_window.id, Some(monitor));
    assert!(app.monitor_window.ready);
}
