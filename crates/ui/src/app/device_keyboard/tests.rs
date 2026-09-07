use super::*;

#[test]
fn tab_activation_and_pointer_input_keep_one_selected_action() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    let main = window::Id::unique();
    app.main_window_id = Some(main);
    let _ = app.update(Message::OpenMonitor);
    app.select_memory(0x1234);
    let split = app.monitor_split;
    for _ in 0..2 {
        key(&mut app, main, keyboard::key::Named::Tab, false);
    }
    assert!(
        app.monitor_window
            .focus
            .matches(&Message::ToggleMonitorSplit)
    );
    assert!(app.monitor_window.focus.keyboard);
    for expected in [!split, split] {
        key(&mut app, main, keyboard::key::Named::Enter, false);
        assert_eq!(app.monitor_split, expected);
        assert!(
            app.monitor_window
                .focus
                .matches(&Message::ToggleMonitorSplit)
        );
        assert!(!app.monitor_window.focus.keyboard);
        assert_eq!(app.selected_memory_address(), Some(0x1234));
    }
    let _ = app.update(Message::DeviceButtonPressed(
        ToolWindowKind::Monitor,
        Box::new(Message::ToggleMonitorHexPopup),
    ));
    key(&mut app, main, keyboard::key::Named::Tab, true);
    assert!(
        app.monitor_window
            .focus
            .matches(&Message::ToggleMonitorHexPopup)
    );
    key(&mut app, main, keyboard::key::Named::Tab, false);
    key(&mut app, main, keyboard::key::Named::Space, false);
    assert_eq!(
        app.monitor_hex_filter,
        super::super::HexStreamFilter::Graphics
    );
    key(&mut app, main, keyboard::key::Named::Tab, true);
    key(&mut app, main, keyboard::key::Named::Enter, false);
    assert!(!app.monitor_hex_popup);
}

#[test]
fn device_navigation_respects_windows_modals_and_disabled_buttons() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    let main = window::Id::unique();
    let detached = window::Id::unique();
    app.main_window_id = Some(main);
    let _ = app.update(Message::OpenHdd);
    app.hdd_window.id = Some(detached);
    app.hdd_window.detached = true;
    assert_eq!(app.device_keyboard_owner(main), None);
    assert_eq!(
        app.device_keyboard_owner(detached),
        Some(ToolWindowKind::Hdd)
    );
    for exists in [false, true] {
        app.hdd_file_exists = exists;
        app.hdd_window.focus = DeviceFocus::default();
        key(&mut app, detached, keyboard::key::Named::Tab, true);
        assert!(app.hdd_window.focus.matches(&Message::CloseHdd));
        key(&mut app, detached, keyboard::key::Named::Tab, true);
        assert!(app.hdd_window.focus.matches(&if exists {
            Message::DeleteHddFile
        } else {
            Message::CreateHddFile
        }));
    }
    app.hdd_window.detached = false;
    app.open_discard_modal(super::super::PendingAction::DeleteHdd);
    assert_eq!(app.device_keyboard_owner(main), None);
    app.cancel_discard();
    let _ = app.update(Message::OpenPrinter);
    app.snapshot.devices.printer.status = DeviceStatus::Busy;
    key(&mut app, main, keyboard::key::Named::Tab, true);
    key(&mut app, main, keyboard::key::Named::Tab, true);
    key(&mut app, main, keyboard::key::Named::Tab, true);
    assert!(
        app.printer_window
            .focus
            .matches(&Message::ConfigurePrinterSession)
    );
}

fn key(app: &mut DesktopApp, window: window::Id, key: keyboard::key::Named, shift: bool) {
    let event = Event::Keyboard(keyboard::Event::KeyPressed {
        key: keyboard::Key::Named(key),
        modified_key: keyboard::Key::Named(key),
        physical_key: keyboard::key::Physical::Code(keyboard::key::Code::Tab),
        location: keyboard::Location::Standard,
        modifiers: if shift {
            keyboard::Modifiers::SHIFT
        } else {
            keyboard::Modifiers::default()
        },
        text: None,
        repeat: false,
    });
    let _ = app.handle_runtime_event(event, iced::event::Status::Captured, window);
}
