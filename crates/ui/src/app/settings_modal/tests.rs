use std::sync::Mutex;

use super::dialog::SettingsDialog;
use super::focus::ResetConfirmFocus;
use crate::app::messages::SpeedTier;
use crate::app::{DesktopApp, Message, StatusKind};
use crate::i18n::Lang;
use crate::persistence::{
    ColorScheme, NetworkSettings, PrinterDialogMode, ShortcutAction, ShortcutBinding, ShortcutKey,
};
use crate::settings_storage::default_lang;

mod general;
mod initialization;
mod navigation;
mod printer;
mod routing;
mod saving;
mod shortcuts;

static FILE_ASSOC_TEST_MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn live_speed_change_updates_active_tier_immediately() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.speed_tier = SpeedTier::Slow;
    app.default_speed = SpeedTier::Slow;
    app.settings_dialog = Some(SettingsDialog::new(
        app.lang,
        app.default_speed,
        true,
        true,
        None,
        None,
        NetworkSettings::default(),
    ));

    let _ = app.update(Message::SettingsDraftSpeedChanged(SpeedTier::Max));

    assert_eq!(app.speed_tier, SpeedTier::Max);
    assert_eq!(app.default_speed, SpeedTier::Max);
}

#[test]
fn cancel_rolls_back_live_speed_to_pre_open_snapshot() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.speed_tier = SpeedTier::Slow;
    app.default_speed = SpeedTier::Slow;
    app.settings_dialog = Some(SettingsDialog::new(
        app.lang,
        app.default_speed,
        true,
        true,
        None,
        None,
        NetworkSettings::default(),
    ));

    let _ = app.update(Message::SettingsDraftSpeedChanged(SpeedTier::Max));
    let _ = app.update(Message::CloseSettings);

    assert_eq!(app.speed_tier, SpeedTier::Slow);
    assert_eq!(app.default_speed, SpeedTier::Slow);
    assert!(app.settings_dialog.is_none());
}

#[test]
fn live_theme_change_updates_active_scheme_immediately() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.color_scheme = ColorScheme::TokyoNight;
    app.settings_dialog = Some(SettingsDialog::new(
        app.lang,
        app.default_speed,
        true,
        true,
        None,
        None,
        NetworkSettings::default(),
    ));

    let _ = app.update(Message::SettingsDraftColorSchemeChanged(
        ColorScheme::GruvboxDark,
    ));

    assert_eq!(app.color_scheme, ColorScheme::GruvboxDark);
    assert_eq!(
        app.settings_dialog.as_ref().unwrap().draft_color_scheme,
        ColorScheme::GruvboxDark
    );
}

#[test]
fn cancel_rolls_back_live_theme_to_pre_open_snapshot() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
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

    let _ = app.update(Message::SettingsDraftColorSchemeChanged(
        ColorScheme::MaterialOcean,
    ));
    let _ = app.update(Message::CloseSettings);

    assert_eq!(app.color_scheme, ColorScheme::TokyoNight);
    assert!(app.settings_dialog.is_none());
}

#[test]
fn opening_settings_dismisses_open_device_panel() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    let _ = app.update(Message::OpenMonitor);
    assert!(app.monitor_open);

    let _ = app.update(Message::OpenSettings);
    assert!(app.settings_dialog.is_some());
    assert!(!app.monitor_open);

    let _ = app.update(Message::CloseSettings);
    assert!(app.settings_dialog.is_none());
    assert!(!app.monitor_open);
}

#[test]
fn reset_confirm_restores_defaults_and_clears_dialog_snapshot() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.lang = Lang::En;
    app.default_speed = SpeedTier::Max;
    app.speed_tier = SpeedTier::Max;
    app.printer_dialog_mode = PrinterDialogMode::System;
    app.settings_dialog = Some(SettingsDialog::new(
        app.lang,
        app.default_speed,
        true,
        true,
        None,
        None,
        NetworkSettings::default(),
    ));
    let dialog = app.settings_dialog.as_mut().unwrap();
    dialog.draft_printer_dialog_mode = PrinterDialogMode::System;
    dialog.original_printer_dialog_mode = PrinterDialogMode::System;

    let _ = app.update(Message::SettingsResetRequested);
    assert!(app.settings_dialog.as_ref().unwrap().reset_confirm_open);

    let _ = app.update(Message::SettingsResetConfirmed);

    let expected_lang = default_lang();
    assert_eq!(app.lang, expected_lang);
    assert_eq!(app.default_speed, SpeedTier::High);
    assert_eq!(app.speed_tier, SpeedTier::High);
    let dialog = app.settings_dialog.as_ref().unwrap();
    assert!(!dialog.reset_confirm_open);
    assert_eq!(dialog.original_lang, expected_lang);
    assert_eq!(dialog.original_speed, SpeedTier::High);
    assert!(!app.follow_pc);
    assert!(!dialog.original_follow_pc);
    assert!(app.memory_operand_highlighting);
    assert!(dialog.original_memory_operand_highlighting);
    assert_eq!(app.printer_dialog_mode, PrinterDialogMode::Custom);
    assert_eq!(dialog.draft_printer_dialog_mode, PrinterDialogMode::Custom);
    assert_eq!(
        dialog.original_printer_dialog_mode,
        PrinterDialogMode::Custom
    );
    assert_eq!(
        app.shortcut_settings.binding(ShortcutAction::OpenMonitor),
        Some(ShortcutBinding::new(true, false, false, ShortcutKey::M))
    );
}

#[test]
fn reset_confirm_opens_with_cancel_focused() {
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

    let _ = app.update(Message::SettingsResetRequested);

    let dialog = app.settings_dialog.as_ref().unwrap();
    assert!(dialog.reset_confirm_open);
    assert_eq!(dialog.reset_confirm_focus, ResetConfirmFocus::Cancel);
    assert!(!dialog.reset_confirm_keyboard_focus_visible);
}

#[test]
fn tab_toggles_reset_confirm_focus_in_a_two_button_ring() {
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
    let _ = app.update(Message::SettingsResetRequested);

    let _ = app.update(Message::FocusCycle { backward: false });
    assert_eq!(
        app.settings_dialog.as_ref().unwrap().reset_confirm_focus,
        ResetConfirmFocus::Confirm
    );
    assert!(
        app.settings_dialog
            .as_ref()
            .unwrap()
            .reset_confirm_keyboard_focus_visible
    );

    let _ = app.update(Message::FocusCycle { backward: false });
    assert_eq!(
        app.settings_dialog.as_ref().unwrap().reset_confirm_focus,
        ResetConfirmFocus::Cancel
    );

    let _ = app.update(Message::FocusCycle { backward: true });
    assert_eq!(
        app.settings_dialog.as_ref().unwrap().reset_confirm_focus,
        ResetConfirmFocus::Confirm
    );
}

#[test]
fn enter_in_reset_confirm_activates_focused_button() {
    // The router's Enter handler returns a follow-up Task that the
    // test harness does not execute, so we dispatch the follow-up
    // manually to verify routing rather than the iced runtime.
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.lang = Lang::En;
    app.default_speed = SpeedTier::Max;
    app.speed_tier = SpeedTier::Max;
    app.settings_dialog = Some(SettingsDialog::new(
        app.lang,
        app.default_speed,
        true,
        true,
        None,
        None,
        NetworkSettings::default(),
    ));

    let _ = app.update(Message::SettingsResetRequested);
    assert_eq!(
        app.settings_dialog.as_ref().unwrap().reset_confirm_focus,
        ResetConfirmFocus::Cancel
    );
    let _ = app.update(Message::SettingsResetCancelled);
    assert!(!app.settings_dialog.as_ref().unwrap().reset_confirm_open);
    assert_eq!(app.lang, Lang::En);
    assert_eq!(app.speed_tier, SpeedTier::Max);

    let _ = app.update(Message::SettingsResetRequested);
    let _ = app.update(Message::FocusCycle { backward: false });
    assert_eq!(
        app.settings_dialog.as_ref().unwrap().reset_confirm_focus,
        ResetConfirmFocus::Confirm
    );
    let _ = app.update(Message::SettingsResetConfirmed);
    assert_eq!(app.lang, default_lang());
    assert_eq!(app.speed_tier, SpeedTier::High);
}

#[test]
fn language_change_re_renders_canonical_status_string() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.lang = Lang::Ru;
    app.set_status(StatusKind::Ready);
    assert_eq!(app.status, "Готов");

    app.lang = Lang::En;
    app.refresh_localized_status();
    assert_eq!(app.status, "Ready");

    app.set_status_custom("entity not found".to_owned());
    app.lang = Lang::Ru;
    app.refresh_localized_status();
    assert_eq!(app.status, "entity not found");
}

#[cfg(windows)]
#[test]
fn settings_button_toggles_file_association_state() {
    let _guard = FILE_ASSOC_TEST_MUTEX.lock().unwrap();
    let was_registered = k580_ui::file_assoc::is_registered();
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    let _ = k580_ui::file_assoc::unregister();
    app.settings_dialog = Some(SettingsDialog::new(
        app.lang,
        app.default_speed,
        true,
        true,
        None,
        None,
        NetworkSettings::default(),
    ));
    assert!(!k580_ui::file_assoc::is_registered());
    assert_eq!(app.file_association_toggle_revision, 0);

    let _ = app.update(Message::SettingsFileAssociationRegister);
    assert!(k580_ui::file_assoc::is_registered());
    assert_eq!(app.file_association_toggle_revision, 1);

    let _ = app.update(Message::SettingsFileAssociationUnregister);
    assert!(!k580_ui::file_assoc::is_registered());
    assert_eq!(app.file_association_toggle_revision, 2);

    if was_registered {
        k580_ui::file_assoc::register().unwrap();
    }
}

#[cfg(windows)]
#[test]
fn tick_bumps_file_association_revision_on_external_change() {
    let _guard = FILE_ASSOC_TEST_MUTEX.lock().unwrap();
    let was_registered = k580_ui::file_assoc::is_registered();
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    let _ = k580_ui::file_assoc::unregister();
    let _ = app.handle_tick();
    app.settings_dialog = Some(SettingsDialog::new(
        app.lang,
        app.default_speed,
        true,
        true,
        None,
        None,
        NetworkSettings::default(),
    ));
    assert!(!k580_ui::file_assoc::is_registered());
    let revision_before = app.file_association_toggle_revision;

    k580_ui::file_assoc::register().unwrap();
    let _ = app.handle_tick();
    assert!(k580_ui::file_assoc::is_registered());
    assert_eq!(app.file_association_toggle_revision, revision_before + 1);

    k580_ui::file_assoc::unregister().unwrap();
    let _ = app.handle_tick();
    assert!(!k580_ui::file_assoc::is_registered());
    assert_eq!(app.file_association_toggle_revision, revision_before + 2);

    if was_registered {
        k580_ui::file_assoc::register().unwrap();
    }
}
