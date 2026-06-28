use crate::app::messages::SpeedTier;
use crate::app::settings_modal::SettingsDialog;
use crate::app::{DesktopApp, Message};
use crate::i18n::Lang;
use crate::persistence::{
    NetworkSettings, ShortcutAction, ShortcutBinding, ShortcutKey, ShortcutSettings,
};

#[test]
fn dialog_copies_shortcut_settings() {
    let mut shortcuts = ShortcutSettings::default();
    shortcuts.assign(
        ShortcutAction::OpenMonitor,
        ShortcutBinding::new(true, true, true, ShortcutKey::M),
    );

    let dialog = SettingsDialog::new_with_shortcuts(
        Lang::Ru,
        SpeedTier::Medium,
        true,
        true,
        None,
        None,
        NetworkSettings::default(),
        shortcuts.clone(),
    );

    assert_eq!(dialog.draft_shortcuts, shortcuts);
    assert_eq!(dialog.original_shortcuts, shortcuts);
}

#[test]
fn shortcut_capture_updates_live_preview_and_draft() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.shortcut_settings = ShortcutSettings::default();
    app.settings_dialog = Some(SettingsDialog::new_with_shortcuts(
        app.lang,
        app.default_speed,
        true,
        true,
        None,
        None,
        NetworkSettings::default(),
        app.shortcut_settings.clone(),
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

    let dialog = app.settings_dialog.as_ref().unwrap();
    assert_eq!(
        dialog.draft_shortcuts.binding(ShortcutAction::OpenMonitor),
        Some(ShortcutBinding::new(true, true, true, ShortcutKey::M))
    );
    assert_eq!(
        app.shortcut_settings.binding(ShortcutAction::OpenMonitor),
        Some(ShortcutBinding::new(true, true, true, ShortcutKey::M))
    );
    assert_eq!(dialog.recording_shortcut, None);

    let _ = app.update(Message::CloseSettings);

    assert_eq!(
        app.shortcut_settings.binding(ShortcutAction::OpenMonitor),
        Some(ShortcutBinding::new(true, false, false, ShortcutKey::M))
    );
    assert!(app.settings_dialog.is_none());
}

#[test]
fn shortcut_reset_updates_live_preview_and_rolls_back_on_cancel() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    let mut live_shortcuts = ShortcutSettings::default();
    live_shortcuts.assign(
        ShortcutAction::OpenMonitor,
        ShortcutBinding::new(true, true, true, ShortcutKey::M),
    );
    app.shortcut_settings = live_shortcuts.clone();
    app.settings_dialog = Some(SettingsDialog::new_with_shortcuts(
        app.lang,
        app.default_speed,
        true,
        true,
        None,
        None,
        NetworkSettings::default(),
        live_shortcuts,
    ));

    let _ = app.update(Message::SettingsShortcutsReset);

    let dialog = app.settings_dialog.as_ref().unwrap();
    assert_eq!(
        dialog.draft_shortcuts.binding(ShortcutAction::OpenMonitor),
        Some(ShortcutBinding::new(true, false, false, ShortcutKey::M))
    );
    assert_eq!(
        app.shortcut_settings.binding(ShortcutAction::OpenMonitor),
        Some(ShortcutBinding::new(true, false, false, ShortcutKey::M))
    );
    assert_eq!(dialog.recording_shortcut, None);

    let _ = app.update(Message::CloseSettings);

    assert_eq!(
        app.shortcut_settings.binding(ShortcutAction::OpenMonitor),
        Some(ShortcutBinding::new(true, true, true, ShortcutKey::M))
    );
    assert!(app.settings_dialog.is_none());
}
