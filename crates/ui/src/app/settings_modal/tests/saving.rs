use std::time::{Duration, Instant};

use super::*;
use crate::app::SettingsSavedNotice;
use crate::persistence::{ShortcutAction, ShortcutBinding, ShortcutKey};

#[test]
fn committed_settings_stay_open_and_advance_cancel_snapshot() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.speed_tier = SpeedTier::Slow;
    app.default_speed = SpeedTier::Slow;
    app.color_scheme = ColorScheme::TokyoNight;
    app.settings_dialog = Some(SettingsDialog::new_with_shortcuts(
        app.lang,
        app.default_speed,
        app.color_scheme,
        true,
        true,
        None,
        None,
        NetworkSettings::default(),
        app.shortcut_settings.clone(),
    ));

    let _ = app.update(Message::SettingsDraftSpeedChanged(SpeedTier::Max));
    let _ = app.update(Message::SettingsDraftColorSchemeChanged(
        ColorScheme::GruvboxDark,
    ));
    let _ = app.update(Message::SettingsDraftFollowPcSet(false));
    let _ = app.update(Message::SettingsDraftMemoryOperandHighlightingSet(false));
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

    app.commit_settings_dialog_state();

    let dialog = app.settings_dialog.as_ref().unwrap();
    assert_eq!(dialog.original_speed, SpeedTier::Max);
    assert_eq!(dialog.original_color_scheme, ColorScheme::GruvboxDark);
    assert!(!dialog.original_follow_pc);
    assert!(!dialog.original_memory_operand_highlighting);
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
    let _ = app.update(Message::SettingsDraftPrinterDialogModeSet(
        PrinterDialogMode::Custom,
    ));
    let _ = app.update(Message::CloseSettings);

    assert_eq!(app.speed_tier, SpeedTier::Max);
    assert_eq!(app.default_speed, SpeedTier::Max);
    assert_eq!(app.color_scheme, ColorScheme::GruvboxDark);
    assert!(!app.follow_pc);
    assert!(!app.memory_operand_highlighting);
    assert_eq!(app.printer_dialog_mode, PrinterDialogMode::System);
    assert_eq!(
        app.shortcut_settings.binding(ShortcutAction::OpenMonitor),
        Some(ShortcutBinding::new(true, true, false, ShortcutKey::M))
    );
    assert!(app.settings_dialog.is_none());
}

#[test]
fn settings_router_allows_notice_dismissal() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.settings_dialog = Some(SettingsDialog::new(
        app.lang,
        app.default_speed,
        true,
        true,
        None,
        None,
        NetworkSettings::default(),
    ));
    app.settings_saved_notice = Some(SettingsSavedNotice::new(Instant::now()));

    let _ = app.update(Message::DismissSettingsSavedNotice);

    assert!(app.settings_saved_notice.is_none());
    assert!(app.settings_dialog.is_some());
}

#[test]
fn rejected_save_clears_previous_success_notice() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.settings_dialog = Some(SettingsDialog::new(
        app.lang,
        app.default_speed,
        true,
        true,
        None,
        None,
        NetworkSettings::default(),
    ));
    app.settings_dialog
        .as_mut()
        .unwrap()
        .draft_network_client_port = "invalid".to_owned();
    app.settings_saved_notice = Some(SettingsSavedNotice::new(Instant::now()));

    let _ = app.update(Message::SaveSettings);

    assert!(app.settings_saved_notice.is_none());
    assert!(
        app.settings_dialog
            .as_ref()
            .unwrap()
            .network_error
            .is_some()
    );
}

#[test]
fn tick_removes_notice_at_two_second_deadline() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.settings_saved_notice = Some(SettingsSavedNotice::new(
        Instant::now() - Duration::from_secs(2),
    ));

    let _ = app.handle_tick();

    assert!(app.settings_saved_notice.is_none());
}
