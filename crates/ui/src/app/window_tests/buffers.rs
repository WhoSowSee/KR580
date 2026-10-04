use super::*;

#[test]
fn printer_buffer_view_toggles_between_hex_and_text() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);

    assert!(!app.panels.printer_text_view);

    let _task = app.update(Message::TogglePrinterBufferView);
    assert!(app.panels.printer_text_view);

    let _task = app.update(Message::TogglePrinterBufferView);
    assert!(!app.panels.printer_text_view);
}

#[test]
fn network_buffer_view_toggles_between_hex_and_text() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);

    assert!(!app.panels.network_text_view);

    let _task = app.update(Message::ToggleNetworkBufferView);
    assert!(app.panels.network_text_view);

    let _task = app.update(Message::ToggleNetworkBufferView);
    assert!(!app.panels.network_text_view);
}
