use crate::app::settings_modal::SettingsDialog;
use crate::app::{DesktopApp, Message};
use crate::persistence::NetworkSettings;

#[test]
fn memory_operand_highlighting_live_change_updates_app_state() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.memory_operand_highlighting = false;
    app.settings_dialog = Some(SettingsDialog::new(
        app.lang,
        app.default_speed,
        true,
        false,
        None,
        None,
        NetworkSettings::default(),
    ));

    let _ = app.update(Message::SettingsDraftMemoryOperandHighlightingSet(true));

    assert!(app.memory_operand_highlighting);
}

#[test]
fn cancel_rolls_back_memory_operand_highlighting_to_pre_open_snapshot() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.memory_operand_highlighting = false;
    app.settings_dialog = Some(SettingsDialog::new(
        app.lang,
        app.default_speed,
        true,
        false,
        None,
        None,
        NetworkSettings::default(),
    ));

    let _ = app.update(Message::SettingsDraftMemoryOperandHighlightingSet(true));
    let _ = app.update(Message::CloseSettings);

    assert!(!app.memory_operand_highlighting);
    assert!(app.settings_dialog.is_none());
}
