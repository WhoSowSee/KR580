use super::*;

#[test]
fn detached_setup_uses_dedicated_window_without_replacing_printer() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    app.panels.printer_open = true;
    let printer_window = iced::window::Id::unique();
    app.panels
        .printer_window
        .set_native(crate::app::windows::NativeWindow::Opening(printer_window));
    app.panels
        .printer_window
        .detach(app.panels.printer_window.native().unwrap());

    let _ = app.open_printer_setup_dialog(PrinterSetupTarget::Session);
    assert!(
        !app.printer_setup
            .printer_setup_dialog
            .as_ref()
            .unwrap()
            .owner_ready
    );

    let _ = app.update(Message::PrinterSetupWindowPositionLoaded(Some(Point::new(
        680.0, 560.0,
    ))));
    let setup_window = app.printer_setup.printer_setup_window_id.unwrap();
    assert_ne!(setup_window, printer_window);
    assert_eq!(app.panels.printer_window.id(), Some(printer_window));
    let setup = app.printer_setup.printer_setup_dialog.as_ref().unwrap();
    assert!(!setup.owner_ready);
    assert_eq!(setup.owner_position, Some(Point::new(680.0, 560.0)));

    let _ = app.update(Message::WindowOpened(setup_window));
    assert!(
        app.printer_setup
            .printer_setup_dialog
            .as_ref()
            .unwrap()
            .owner_ready
    );
}

#[test]
fn detached_printer_dialogs_are_centered_on_printer() {
    assert_eq!(
        detached_printer_setup_position(Point::new(680.0, 560.0)),
        Point::new(700.0, 480.0)
    );
    assert_eq!(
        detached_printer_properties_position(Point::new(680.0, 560.0)),
        Point::new(540.0, 390.0)
    );
}

#[test]
fn detached_properties_use_their_own_window() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    app.panels.printer_open = true;
    let printer_window = iced::window::Id::unique();
    let setup_window = iced::window::Id::unique();
    app.panels
        .printer_window
        .set_native(crate::app::windows::NativeWindow::Opening(printer_window));
    app.panels
        .printer_window
        .detach(app.panels.printer_window.native().unwrap());
    app.printer_setup.printer_setup_window_id = Some(setup_window);

    let mut setup = PrinterSetupDialog::new(PrinterSetupTarget::Session, None);
    setup.owner_position = Some(Point::new(680.0, 560.0));
    setup.properties = Some(PrinterPropertiesDialog::new(String::new(), Vec::new()));
    setup.properties_surface_ready = false;
    app.printer_setup.printer_setup_dialog = Some(setup);

    let _ = app.open_detached_printer_properties_window();
    let properties_window = app.printer_setup.printer_properties_window_id.unwrap();
    assert_ne!(properties_window, setup_window);
    assert_ne!(properties_window, printer_window);
    assert_eq!(
        app.printer_setup.printer_setup_window_id,
        Some(setup_window)
    );
    assert_eq!(app.panels.printer_window.id(), Some(printer_window));

    let _ = app.update(Message::WindowOpened(properties_window));
    assert!(
        app.printer_setup
            .printer_setup_dialog
            .as_ref()
            .unwrap()
            .properties_surface_ready
    );

    let _ = app.update(Message::WindowCloseRequested(setup_window));
    assert_eq!(
        app.printer_setup.printer_setup_window_id,
        Some(setup_window)
    );
    assert_eq!(
        app.printer_setup.printer_properties_window_id,
        Some(properties_window)
    );
    assert!(
        app.printer_setup
            .printer_setup_dialog
            .as_ref()
            .unwrap()
            .properties
            .is_some()
    );

    let _ = app.update(Message::ClosePrinterProperties);
    assert_eq!(app.printer_setup.printer_properties_window_id, None);
    assert_eq!(
        app.printer_setup.printer_setup_window_id,
        Some(setup_window)
    );
    assert!(
        app.printer_setup
            .printer_setup_dialog
            .as_ref()
            .unwrap()
            .properties
            .is_none()
    );
}

#[test]
fn property_attention_pulse_rises_fades_and_expires() {
    let started_at = Instant::now();
    let mut properties = PrinterPropertiesDialog::new(String::new(), Vec::new());
    properties.restart_attention(started_at);

    assert_eq!(properties.attention_strength(started_at), 0.0);
    let rising = properties.attention_strength(started_at + Duration::from_millis(130));
    let peak = properties.attention_strength(started_at + Duration::from_millis(260));
    let falling = properties.attention_strength(started_at + Duration::from_millis(390));
    assert!((0.70..0.71).contains(&rising));
    assert!(peak > 0.99);
    assert!((0.70..0.71).contains(&falling));

    properties.expire_attention(started_at + Duration::from_millis(520));
    assert_eq!(properties.attention_strength(started_at), 0.0);
}
