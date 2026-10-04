use crate::app::settings_modal::SettingsDialog;
use crate::app::{DesktopApp, Message};
use crate::persistence::NetworkSettings;

#[test]
fn memory_operand_highlighting_live_change_updates_app_state() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.preferences.memory_operand_highlighting = false;
    app.preferences.settings_dialog = Some(SettingsDialog::new(
        crate::app::settings_modal::SettingsInitialState {
            lang: app.preferences.lang,
            speed: app.preferences.default_speed,
            follow_pc: true,
            memory_operand_highlighting: false,
            floppy_image_path: None,
            hdd_directory: None,
            network: NetworkSettings::default(),
            ..Default::default()
        },
    ));

    let _ = app.update(Message::SettingsDraftMemoryOperandHighlightingSet(true));

    assert!(app.preferences.memory_operand_highlighting);
}

#[test]
fn cancel_rolls_back_live_general_toggles_to_pre_open_snapshot() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.preferences.memory_operand_highlighting = false;
    app.preferences.show_file_name = false;
    app.preferences.settings_dialog = Some(SettingsDialog::new(
        crate::app::settings_modal::SettingsInitialState {
            lang: app.preferences.lang,
            speed: app.preferences.default_speed,
            follow_pc: true,
            memory_operand_highlighting: false,
            floppy_image_path: None,
            hdd_directory: None,
            network: NetworkSettings::default(),
            ..Default::default()
        },
    ));

    let _ = app.update(Message::SettingsDraftMemoryOperandHighlightingSet(true));
    let _ = app.update(Message::SettingsDraftShowFileNameSet(true));
    assert!(app.preferences.memory_operand_highlighting);
    assert!(app.preferences.show_file_name);
    let _ = app.update(Message::CloseSettings);

    assert!(!app.preferences.memory_operand_highlighting);
    assert!(!app.preferences.show_file_name);
    assert!(app.preferences.settings_dialog.is_none());
}
