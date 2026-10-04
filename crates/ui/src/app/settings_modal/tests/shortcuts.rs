use crate::app::messages::SpeedTier;
use crate::app::settings_modal::SettingsDialog;
use crate::app::{DesktopApp, Message};
use crate::i18n::{Key, Lang};
use crate::persistence::{
    ColorScheme, NetworkSettings, ShortcutAction, ShortcutBinding, ShortcutKey, ShortcutSettings,
};

#[test]
fn dialog_copies_shortcut_settings() {
    let mut shortcuts = ShortcutSettings::default();
    shortcuts.assign(
        ShortcutAction::OpenMonitor,
        ShortcutBinding::new(true, true, true, ShortcutKey::M),
    );

    let dialog = SettingsDialog::new(crate::app::settings_modal::SettingsInitialState {
        lang: Lang::Ru,
        speed: SpeedTier::Medium,
        color_scheme: ColorScheme::DEFAULT,
        follow_pc: true,
        memory_operand_highlighting: true,
        floppy_image_path: None,
        hdd_directory: None,
        network: NetworkSettings::default(),
        shortcuts: shortcuts.clone(),
        ..Default::default()
    });

    assert_eq!(dialog.draft_shortcuts, shortcuts);
    assert_eq!(dialog.original_shortcuts, shortcuts);
}

#[test]
fn shortcut_capture_updates_live_preview_and_draft() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.preferences.shortcut_settings = ShortcutSettings::default();
    app.preferences.settings_dialog = Some(SettingsDialog::new(
        crate::app::settings_modal::SettingsInitialState {
            lang: app.preferences.lang,
            speed: app.preferences.default_speed,
            color_scheme: ColorScheme::DEFAULT,
            follow_pc: true,
            memory_operand_highlighting: true,
            floppy_image_path: None,
            hdd_directory: None,
            network: NetworkSettings::default(),
            shortcuts: app.preferences.shortcut_settings.clone(),
            ..Default::default()
        },
    ));

    let _ = app.update(Message::SettingsShortcutCaptureStarted(
        ShortcutAction::OpenMonitor,
    ));
    let _ = app.update(Message::SettingsShortcutCaptured(ShortcutBinding::new(
        true,
        true,
        true,
        ShortcutKey::M,
    )));

    let dialog = app.preferences.settings_dialog.as_ref().unwrap();
    assert_eq!(
        dialog.draft_shortcuts.binding(ShortcutAction::OpenMonitor),
        Some(ShortcutBinding::new(true, true, true, ShortcutKey::M))
    );
    assert_eq!(
        app.preferences
            .shortcut_settings
            .binding(ShortcutAction::OpenMonitor),
        Some(ShortcutBinding::new(true, true, true, ShortcutKey::M))
    );
    assert_eq!(dialog.recording_shortcut, None);

    let _ = app.update(Message::CloseSettings);

    assert_eq!(
        app.preferences
            .shortcut_settings
            .binding(ShortcutAction::OpenMonitor),
        Some(ShortcutBinding::new(true, false, false, ShortcutKey::M))
    );
    assert!(app.preferences.settings_dialog.is_none());
}

#[test]
fn shortcut_reset_updates_live_preview_and_rolls_back_on_cancel() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    let mut live_shortcuts = ShortcutSettings::default();
    live_shortcuts.assign(
        ShortcutAction::OpenMonitor,
        ShortcutBinding::new(true, true, true, ShortcutKey::M),
    );
    app.preferences.shortcut_settings = live_shortcuts.clone();
    app.preferences.settings_dialog = Some(SettingsDialog::new(
        crate::app::settings_modal::SettingsInitialState {
            lang: app.preferences.lang,
            speed: app.preferences.default_speed,
            color_scheme: ColorScheme::DEFAULT,
            follow_pc: true,
            memory_operand_highlighting: true,
            floppy_image_path: None,
            hdd_directory: None,
            network: NetworkSettings::default(),
            shortcuts: live_shortcuts,
            ..Default::default()
        },
    ));

    let _ = app.update(Message::SettingsShortcutsReset);

    let dialog = app.preferences.settings_dialog.as_ref().unwrap();
    assert_eq!(
        dialog.draft_shortcuts.binding(ShortcutAction::OpenMonitor),
        Some(ShortcutBinding::new(true, false, false, ShortcutKey::M))
    );
    assert_eq!(
        app.preferences
            .shortcut_settings
            .binding(ShortcutAction::OpenMonitor),
        Some(ShortcutBinding::new(true, false, false, ShortcutKey::M))
    );
    assert_eq!(dialog.recording_shortcut, None);
    assert_eq!(
        app.preferences.settings_notice.unwrap().message_key(),
        Key::SettingsShortcutsResetNotice
    );

    let _ = app.update(Message::CloseSettings);

    assert_eq!(
        app.preferences
            .shortcut_settings
            .binding(ShortcutAction::OpenMonitor),
        Some(ShortcutBinding::new(true, true, true, ShortcutKey::M))
    );
    assert!(app.preferences.settings_dialog.is_none());
}
