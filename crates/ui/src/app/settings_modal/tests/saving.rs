use std::time::{Duration, Instant};

use super::*;
use crate::app::SettingsNotice;
use crate::i18n::Key;
use crate::persistence::{ShortcutAction, ShortcutBinding, ShortcutKey};

#[test]
fn committed_settings_stay_open_and_advance_cancel_snapshot() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.execution.speed_tier = SpeedTier::Slow;
    app.preferences.default_speed = SpeedTier::Slow;
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

    let _ = app.update(Message::SettingsDraftSpeedChanged(SpeedTier::Max));
    let _ = app.update(Message::SettingsDraftColorSchemeChanged(
        ColorScheme::GruvboxDark,
    ));
    let _ = app.update(Message::SettingsDraftFollowPcSet(false));
    let _ = app.update(Message::SettingsDraftMemoryOperandHighlightingSet(false));
    let _ = app.update(Message::SettingsDraftShowFileNameSet(true));
    let _ = app.update(Message::SettingsDraftMonitorSplitSet(true));
    let _ = app.update(Message::SettingsDraftPrinterDialogModeSet(
        PrinterDialogMode::System,
    ));
    let _ = app.update(Message::SettingsShortcutCaptureStarted(
        ShortcutAction::OpenMonitor,
    ));
    let _ = app.update(Message::SettingsShortcutCaptured(ShortcutBinding::new(
        true,
        true,
        false,
        ShortcutKey::M,
    )));

    app.commit_settings_dialog_state(
        &app.preferences.settings_dialog.as_ref().unwrap().clone(),
        app.execution.speed_tier,
    );

    let dialog = app.preferences.settings_dialog.as_ref().unwrap();
    assert_eq!(dialog.original_speed, SpeedTier::Max);
    assert_eq!(dialog.original_color_scheme, ColorScheme::GruvboxDark);
    assert!(!dialog.original_follow_pc);
    assert!(!dialog.original_memory_operand_highlighting);
    assert!(dialog.original_show_file_name);
    assert!(dialog.original_monitor_split);
    assert_eq!(
        dialog.original_printer_dialog_mode,
        PrinterDialogMode::System
    );

    let _ = app.update(Message::SettingsDraftSpeedChanged(SpeedTier::Slow));
    let _ = app.update(Message::SettingsDraftColorSchemeChanged(
        ColorScheme::MaterialOcean,
    ));
    let _ = app.update(Message::SettingsDraftFollowPcSet(true));
    let _ = app.update(Message::SettingsDraftMemoryOperandHighlightingSet(true));
    let _ = app.update(Message::SettingsDraftShowFileNameSet(false));
    let _ = app.update(Message::SettingsDraftMonitorSplitSet(false));
    let _ = app.update(Message::SettingsDraftPrinterDialogModeSet(
        PrinterDialogMode::Custom,
    ));
    let _ = app.update(Message::CloseSettings);

    assert_eq!(app.execution.speed_tier, SpeedTier::Max);
    assert_eq!(app.preferences.default_speed, SpeedTier::Max);
    assert_eq!(app.preferences.color_scheme, ColorScheme::GruvboxDark);
    assert!(!app.preferences.follow_pc);
    assert!(!app.preferences.memory_operand_highlighting);
    assert!(app.preferences.show_file_name);
    assert!(app.panels.monitor_split);
    assert_eq!(
        app.printer_setup.printer_dialog_mode,
        PrinterDialogMode::System
    );
    assert_eq!(
        app.preferences
            .shortcut_settings
            .binding(ShortcutAction::OpenMonitor),
        Some(ShortcutBinding::new(true, true, false, ShortcutKey::M))
    );
    assert!(app.preferences.settings_dialog.is_none());
}

#[test]
fn settings_router_allows_notice_dismissal() {
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
    app.preferences.settings_notice = Some(SettingsNotice::new(
        Key::SettingsSavedNotice,
        Instant::now(),
    ));

    let _ = app.update(Message::DismissSettingsNotice);

    assert!(app.preferences.settings_notice.is_none());
    assert!(app.preferences.settings_dialog.is_some());
}

#[test]
fn rejected_save_clears_previous_success_notice() {
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
    app.preferences
        .settings_dialog
        .as_mut()
        .unwrap()
        .draft_network_client_port = "invalid".to_owned();
    app.preferences.settings_notice = Some(SettingsNotice::new(
        Key::SettingsSavedNotice,
        Instant::now(),
    ));

    let _ = app.update(Message::SaveSettings);

    assert!(app.preferences.settings_notice.is_none());
    assert!(
        app.preferences
            .settings_dialog
            .as_ref()
            .unwrap()
            .network_error
            .is_some()
    );
}

#[test]
fn tick_removes_notice_at_two_second_deadline() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.preferences.settings_notice = Some(SettingsNotice::new(
        Key::SettingsSavedNotice,
        Instant::now() - Duration::from_secs(2),
    ));

    let _ = app.handle_tick();

    assert!(app.preferences.settings_notice.is_none());
}
