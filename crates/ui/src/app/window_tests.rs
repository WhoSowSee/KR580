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

#[test]
fn detached_tool_dialog_uses_tool_window_as_parent() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    let main = window::Id::unique();
    let floppy = window::Id::unique();
    app.shell.main_window_id = Some(main);
    app.panels
        .floppy_window
        .set_native(crate::app::windows::NativeWindow::Opening(floppy));
    app.panels.floppy_window.mark_ready();
    app.panels
        .floppy_window
        .detach(app.panels.floppy_window.native().unwrap());

    assert_eq!(
        app.dialog_parent(Some(ToolWindowKind::Floppy)),
        Some(floppy)
    );
}

#[test]
fn main_and_attached_tool_dialogs_use_main_window() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    let main = window::Id::unique();
    app.shell.main_window_id = Some(main);
    app.panels
        .hdd_window
        .set_native(crate::app::windows::NativeWindow::Opening(
            window::Id::unique(),
        ));
    app.panels.hdd_window.mark_ready();

    assert_eq!(app.dialog_parent(None), Some(main));
    assert_eq!(app.dialog_parent(Some(ToolWindowKind::Hdd)), Some(main));
}

#[cfg(windows)]
#[test]
fn second_startup_frame_does_not_prepare_hidden_tool_windows() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.shell.main_window_id = Some(window::Id::unique());
    app.shell.startup_frames_seen = 1;

    let _task = app.update(Message::FrameRendered);

    assert!(app.panels.monitor_window.id().is_none());
    assert!(app.panels.floppy_window.id().is_none());
    assert!(app.panels.hdd_window.id().is_none());
    assert!(app.panels.network_window.id().is_none());
    assert!(app.panels.printer_window.id().is_none());
}

#[cfg(windows)]
#[test]
fn detaching_storage_lazily_opens_native_window() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);

    let _task = app.update(Message::DetachToolWindow(ToolWindowKind::Floppy));

    assert!(app.panels.floppy_open);
    assert!(app.panels.floppy_window.id().is_some());
    assert!(app.panels.floppy_window.detached());
    assert!(!app.panels.floppy_window.ready());
}

#[cfg(windows)]
#[test]
fn detaching_storage_reuses_prepared_native_windows() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    let floppy = window::Id::unique();
    let hdd = window::Id::unique();
    app.panels.floppy_open = true;
    app.panels
        .floppy_window
        .set_native(crate::app::windows::NativeWindow::Opening(floppy));
    app.panels.floppy_window.mark_ready();
    app.panels.hdd_open = true;
    app.panels
        .hdd_window
        .set_native(crate::app::windows::NativeWindow::Opening(hdd));
    app.panels.hdd_window.mark_ready();

    let _task = app.update(Message::DetachToolWindow(ToolWindowKind::Floppy));
    let _task = app.update(Message::DetachToolWindow(ToolWindowKind::Hdd));

    assert_eq!(app.panels.floppy_window.id(), Some(floppy));
    assert!(app.panels.floppy_window.detached());
    assert_eq!(app.panels.hdd_window.id(), Some(hdd));
    assert!(app.panels.hdd_window.detached());
}

#[cfg(windows)]
#[test]
fn detaching_monitor_reuses_prepared_native_window() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    let monitor = window::Id::unique();
    app.panels.monitor_open = true;
    app.panels
        .monitor_window
        .set_native(crate::app::windows::NativeWindow::Opening(monitor));

    let _task = app.update(Message::DetachToolWindow(ToolWindowKind::Monitor));

    assert!(app.panels.monitor_open);
    assert_eq!(app.panels.monitor_window.id(), Some(monitor));
    assert!(app.panels.monitor_window.detached());
}

#[cfg(windows)]
#[test]
fn attaching_monitor_hides_native_window_and_restores_overlay() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    let monitor = window::Id::unique();
    app.panels.monitor_open = true;
    app.panels
        .monitor_window
        .set_native(crate::app::windows::NativeWindow::Opening(monitor));
    app.panels.monitor_window.mark_ready();
    app.panels.monitor_window.mark_ready();
    app.panels
        .monitor_window
        .detach(app.panels.monitor_window.native().unwrap());
    app.panels.monitor_window.toggle_pin();

    let _task = app.update(Message::AttachToolWindow(ToolWindowKind::Monitor));

    assert!(app.panels.monitor_open);
    assert_eq!(app.panels.monitor_window.id(), Some(monitor));
    assert!(!app.panels.monitor_window.detached());
    assert!(!app.panels.monitor_window.always_on_top());
}

#[test]
fn detached_monitor_pin_toggles_always_on_top() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    let monitor = window::Id::unique();
    app.panels.monitor_open = true;
    app.panels
        .monitor_window
        .set_native(crate::app::windows::NativeWindow::Opening(monitor));
    app.panels
        .monitor_window
        .detach(app.panels.monitor_window.native().unwrap());
    app.panels.monitor_window.mark_ready();

    let _task = app.update(Message::ToggleToolWindowAlwaysOnTop(
        ToolWindowKind::Monitor,
    ));

    assert!(app.panels.monitor_window.always_on_top());
    assert!(app.panels.monitor_open);
    assert!(app.panels.monitor_window.detached());
    assert!(app.panels.monitor_window.ready());
    assert_eq!(app.panels.monitor_window.id(), Some(monitor));

    let _task = app.update(Message::ToggleToolWindowAlwaysOnTop(
        ToolWindowKind::Monitor,
    ));

    assert!(!app.panels.monitor_window.always_on_top());
    assert!(app.panels.monitor_open);
    assert!(app.panels.monitor_window.detached());
    assert!(app.panels.monitor_window.ready());
    assert_eq!(app.panels.monitor_window.id(), Some(monitor));
}

#[test]
fn detached_storage_pin_and_attach_are_independent() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    let floppy = window::Id::unique();
    let hdd = window::Id::unique();
    app.panels.floppy_open = true;
    app.panels
        .floppy_window
        .set_native(crate::app::windows::NativeWindow::Opening(floppy));
    app.panels.floppy_window.mark_ready();
    app.panels
        .floppy_window
        .detach(app.panels.floppy_window.native().unwrap());
    app.panels
        .hdd_window
        .set_native(crate::app::windows::NativeWindow::Opening(hdd));
    app.panels.hdd_window.mark_ready();

    let _task = app.update(Message::ToggleToolWindowAlwaysOnTop(ToolWindowKind::Floppy));

    assert!(app.panels.floppy_window.always_on_top());
    assert!(!app.panels.hdd_window.always_on_top());

    let _task = app.update(Message::AttachToolWindow(ToolWindowKind::Floppy));

    assert!(app.panels.floppy_open);
    assert!(!app.panels.floppy_window.detached());
    assert!(!app.panels.floppy_window.always_on_top());
    assert_eq!(
        app.panels.floppy_window.id(),
        cfg!(windows).then_some(floppy)
    );
    assert_eq!(app.panels.hdd_window.id(), Some(hdd));
}

#[test]
fn detached_network_pin_and_attach_are_independent() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    let network = window::Id::unique();
    app.panels.network_open = true;
    app.panels
        .network_window
        .set_native(crate::app::windows::NativeWindow::Opening(network));
    app.panels.network_window.mark_ready();
    app.panels
        .network_window
        .detach(app.panels.network_window.native().unwrap());

    let _task = app.update(Message::ToggleToolWindowAlwaysOnTop(
        ToolWindowKind::Network,
    ));

    assert!(app.panels.network_window.always_on_top());

    let _task = app.update(Message::AttachToolWindow(ToolWindowKind::Network));

    assert!(app.panels.network_open);
    assert!(!app.panels.network_window.detached());
    assert!(!app.panels.network_window.always_on_top());
    assert_eq!(
        app.panels.network_window.id(),
        cfg!(windows).then_some(network)
    );
}

#[test]
fn detached_printer_pin_and_attach_are_independent() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    let printer = window::Id::unique();
    app.panels.printer_open = true;
    app.panels
        .printer_window
        .set_native(crate::app::windows::NativeWindow::Opening(printer));
    app.panels.printer_window.mark_ready();
    app.panels
        .printer_window
        .detach(app.panels.printer_window.native().unwrap());

    let _task = app.update(Message::ToggleToolWindowAlwaysOnTop(
        ToolWindowKind::Printer,
    ));

    assert!(app.panels.printer_window.always_on_top());

    let _task = app.update(Message::AttachToolWindow(ToolWindowKind::Printer));

    assert!(app.panels.printer_open);
    assert!(!app.panels.printer_window.detached());
    assert!(!app.panels.printer_window.always_on_top());
    assert_eq!(
        app.panels.printer_window.id(),
        cfg!(windows).then_some(printer)
    );
}

#[test]
fn closing_detached_monitor_does_not_close_main_window() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    let main = window::Id::unique();
    let monitor = window::Id::unique();
    app.shell.main_window_id = Some(main);
    app.panels
        .monitor_window
        .set_native(crate::app::windows::NativeWindow::Opening(monitor));
    app.panels.monitor_window.mark_ready();
    app.panels
        .monitor_window
        .detach(app.panels.monitor_window.native().unwrap());
    app.panels.monitor_open = true;

    let _task = app.update(Message::WindowCloseRequested(monitor));

    assert_eq!(app.shell.main_window_id, Some(main));
    #[cfg(windows)]
    assert_eq!(app.panels.monitor_window.id(), Some(monitor));
    #[cfg(not(windows))]
    assert_eq!(app.panels.monitor_window.id(), None);
    assert!(!app.panels.monitor_window.detached());
    assert!(!app.panels.monitor_open);
}

#[test]
fn closing_detached_hdd_does_not_close_main_window() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    let main = window::Id::unique();
    let hdd = window::Id::unique();
    app.shell.main_window_id = Some(main);
    app.panels
        .hdd_window
        .set_native(crate::app::windows::NativeWindow::Opening(hdd));
    app.panels.hdd_window.mark_ready();
    app.panels
        .hdd_window
        .detach(app.panels.hdd_window.native().unwrap());
    app.panels.hdd_open = true;

    let _task = app.update(Message::WindowCloseRequested(hdd));

    assert_eq!(app.shell.main_window_id, Some(main));
    #[cfg(windows)]
    assert_eq!(app.panels.hdd_window.id(), Some(hdd));
    #[cfg(not(windows))]
    assert_eq!(app.panels.hdd_window.id(), None);
    assert!(!app.panels.hdd_window.detached());
    assert!(!app.panels.hdd_open);
}

#[test]
fn closing_detached_printer_does_not_close_main_window() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    let main = window::Id::unique();
    let printer = window::Id::unique();
    app.shell.main_window_id = Some(main);
    app.panels
        .printer_window
        .set_native(crate::app::windows::NativeWindow::Opening(printer));
    app.panels.printer_window.mark_ready();
    app.panels
        .printer_window
        .detach(app.panels.printer_window.native().unwrap());
    app.panels.printer_open = true;

    let _task = app.update(Message::WindowCloseRequested(printer));

    assert_eq!(app.shell.main_window_id, Some(main));
    #[cfg(windows)]
    assert_eq!(app.panels.printer_window.id(), Some(printer));
    #[cfg(not(windows))]
    assert_eq!(app.panels.printer_window.id(), None);
    assert!(!app.panels.printer_window.detached());
    assert!(!app.panels.printer_open);
}

#[test]
fn opening_detached_monitor_does_not_replace_main_window_id() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    let main = window::Id::unique();
    let monitor = window::Id::unique();
    app.shell.main_window_id = Some(main);
    app.panels
        .monitor_window
        .set_native(crate::app::windows::NativeWindow::Opening(monitor));
    app.panels
        .monitor_window
        .detach(app.panels.monitor_window.native().unwrap());

    let _task = app.update(Message::WindowOpened(monitor));

    assert_eq!(app.shell.main_window_id, Some(main));
    assert_eq!(app.panels.monitor_window.id(), Some(monitor));
    assert!(app.panels.monitor_window.ready());
}

mod buffers;
