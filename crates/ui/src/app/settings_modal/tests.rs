use super::dialog::SettingsDialog;
use super::focus::ResetConfirmFocus;
use crate::app::messages::SpeedTier;
use crate::app::{DesktopApp, Message, StatusKind};
use crate::i18n::{Key, Lang};
use crate::persistence::{
    ColorScheme, NetworkSettings, PrinterDialogMode, ShortcutAction, ShortcutBinding, ShortcutKey,
};
use crate::settings_storage::lang_from_language;
use k580_ui::system_locale::default_language;

mod general;
mod initialization;
mod navigation;
mod printer;
mod routing;
mod saving;
mod shortcuts;

#[test]
fn live_speed_change_updates_active_tier_immediately() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.execution.speed_tier = SpeedTier::Slow;
    app.preferences.default_speed = SpeedTier::Slow;
    app.preferences.settings_dialog = Some(SettingsDialog::new(
        crate::app::settings_modal::SettingsInitialState {
            lang: app.preferences.lang,
            speed: app.preferences.default_speed,
            follow_pc: true,
            memory_operand_highlighting: true,
            floppy_image_path: None,
            hdd_directory: None,
            network: NetworkSettings::default(),
            ..Default::default()
        },
    ));

    let _ = app.update(Message::SettingsDraftSpeedChanged(SpeedTier::Max));

    assert_eq!(app.execution.speed_tier, SpeedTier::Max);
    assert_eq!(app.preferences.default_speed, SpeedTier::Max);
}

#[test]
fn cancel_rolls_back_live_speed_to_pre_open_snapshot() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.execution.speed_tier = SpeedTier::Slow;
    app.preferences.default_speed = SpeedTier::Slow;
    app.preferences.settings_dialog = Some(SettingsDialog::new(
        crate::app::settings_modal::SettingsInitialState {
            lang: app.preferences.lang,
            speed: app.preferences.default_speed,
            follow_pc: true,
            memory_operand_highlighting: true,
            floppy_image_path: None,
            hdd_directory: None,
            network: NetworkSettings::default(),
            ..Default::default()
        },
    ));

    let _ = app.update(Message::SettingsDraftSpeedChanged(SpeedTier::Max));
    let _ = app.update(Message::CloseSettings);

    assert_eq!(app.execution.speed_tier, SpeedTier::Slow);
    assert_eq!(app.preferences.default_speed, SpeedTier::Slow);
    assert!(app.preferences.settings_dialog.is_none());
}

#[test]
fn closing_settings_preserves_active_speed_before_and_after_save() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.preferences.default_speed = SpeedTier::High;
    app.execution.speed_tier = SpeedTier::High;
    app.preferences.follow_pc = false;

    let _ = app.update(Message::SpeedTierChanged(SpeedTier::Slow));
    let _ = app.update(Message::OpenSettings);
    let _ = app.update(Message::SettingsDraftFollowPcSet(true));
    let _ = app.update(Message::CloseSettings);

    assert_eq!(app.execution.speed_tier, SpeedTier::Slow);
    assert_eq!(app.preferences.default_speed, SpeedTier::High);
    assert!(!app.preferences.follow_pc);

    let _ = app.update(Message::OpenSettings);
    let _ = app.update(Message::SettingsDraftFollowPcSet(true));
    app.commit_settings_dialog_state(
        &app.preferences.settings_dialog.as_ref().unwrap().clone(),
        app.execution.speed_tier,
    );
    let _ = app.update(Message::CloseSettings);

    assert_eq!(app.execution.speed_tier, SpeedTier::Slow);
    assert_eq!(app.preferences.default_speed, SpeedTier::High);
    assert!(app.preferences.follow_pc);
    assert!(app.preferences.settings_dialog.is_none());
}

#[test]
fn live_theme_change_updates_active_scheme_immediately() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.preferences.color_scheme = ColorScheme::TokyoNight;
    app.preferences.settings_dialog = Some(SettingsDialog::new(
        crate::app::settings_modal::SettingsInitialState {
            lang: app.preferences.lang,
            speed: app.preferences.default_speed,
            follow_pc: true,
            memory_operand_highlighting: true,
            floppy_image_path: None,
            hdd_directory: None,
            network: NetworkSettings::default(),
            ..Default::default()
        },
    ));

    let _ = app.update(Message::SettingsDraftColorSchemeChanged(
        ColorScheme::GruvboxDark,
    ));

    assert_eq!(app.preferences.color_scheme, ColorScheme::GruvboxDark);
    assert_eq!(
        app.preferences
            .settings_dialog
            .as_ref()
            .unwrap()
            .draft_color_scheme,
        ColorScheme::GruvboxDark
    );
}

#[test]
fn cancel_rolls_back_live_theme_to_pre_open_snapshot() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.preferences.color_scheme = ColorScheme::TokyoNight;
    app.preferences.settings_dialog = Some(SettingsDialog::new(
        crate::app::settings_modal::SettingsInitialState {
            lang: app.preferences.lang,
            speed: app.preferences.default_speed,
            color_scheme: app.preferences.color_scheme,
            follow_pc: true,
            memory_operand_highlighting: true,
            floppy_image_path: None,
            hdd_directory: None,
            network: NetworkSettings::default(),
            shortcuts: app.preferences.shortcut_settings.clone(),
            ..Default::default()
        },
    ));

    let _ = app.update(Message::SettingsDraftColorSchemeChanged(
        ColorScheme::MaterialOcean,
    ));
    let _ = app.update(Message::CloseSettings);

    assert_eq!(app.preferences.color_scheme, ColorScheme::TokyoNight);
    assert!(app.preferences.settings_dialog.is_none());
}

#[test]
fn opening_settings_dismisses_open_device_panel() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.panels.monitor_split = true;
    let _ = app.update(Message::OpenMonitor);
    assert!(app.panels.monitor_open);

    let _ = app.update(Message::OpenSettings);
    assert!(
        app.preferences
            .settings_dialog
            .as_ref()
            .unwrap()
            .original_monitor_split
    );
    assert!(!app.panels.monitor_open);

    let _ = app.update(Message::CloseSettings);
    assert!(app.preferences.settings_dialog.is_none());
    assert!(!app.panels.monitor_open);
    assert!(app.panels.monitor_split);
}

#[test]
fn language_change_re_renders_canonical_status_string() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.preferences.lang = Lang::Ru;
    app.set_status(StatusKind::Ready);
    assert_eq!(app.shell.status, "Готов");

    app.preferences.lang = Lang::En;
    app.refresh_localized_status();
    assert_eq!(app.shell.status, "Ready");

    app.set_status_custom("entity not found".to_owned());
    app.preferences.lang = Lang::Ru;
    app.refresh_localized_status();
    assert_eq!(app.shell.status, "entity not found");
}

#[test]
fn file_association_task_tracks_pending_state_until_completion() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.preferences.settings_dialog = Some(SettingsDialog::new(
        crate::app::settings_modal::SettingsInitialState {
            lang: app.preferences.lang,
            speed: app.preferences.default_speed,
            follow_pc: true,
            memory_operand_highlighting: true,
            floppy_image_path: None,
            hdd_directory: None,
            network: NetworkSettings::default(),
            ..Default::default()
        },
    ));

    let task = app.update(Message::SettingsFileAssociationRegister);
    drop(task);

    assert!(app.preferences.file_association_pending);
    let _ = app.update(Message::CloseSettings);
    let _ = app.update(Message::OpenSettings);
    assert!(app.preferences.file_association_pending);
    assert!(app.update(Message::SettingsFileAssociationRegister).units() == 0);

    let _ = app.update(Message::SettingsFileAssociationFinished(Ok(())));
    assert!(!app.preferences.file_association_pending);
}

mod reset;
