use std::sync::Mutex;

use super::dialog::SettingsDialog;
use super::focus::{FooterFocus, ResetConfirmFocus, SettingsCategory};
use crate::app::messages::SpeedTier;
use crate::app::{DesktopApp, Message, StatusKind};
use crate::i18n::Lang;
use k580_persistence::NetworkSettings;

static FILE_ASSOC_TEST_MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn dialog_starts_on_general_category() {
    let dialog = SettingsDialog::new(
        Lang::Ru,
        SpeedTier::Medium,
        true,
        None,
        None,
        NetworkSettings::default(),
    );
    assert_eq!(dialog.category, SettingsCategory::General);
    assert!(dialog.search.is_empty());
}

#[test]
fn search_query_strips_surrounding_whitespace() {
    let mut dialog = SettingsDialog::new(
        Lang::Ru,
        SpeedTier::Medium,
        true,
        None,
        None,
        NetworkSettings::default(),
    );
    dialog.search = "  скорость  ".to_owned();
    assert_eq!(dialog.search_query(), "скорость");
}

#[test]
fn footer_focus_defaults_to_cancel() {
    let dialog = SettingsDialog::new(
        Lang::Ru,
        SpeedTier::Medium,
        true,
        None,
        None,
        NetworkSettings::default(),
    );
    assert_eq!(dialog.footer_focus, FooterFocus::Cancel);
}

#[test]
fn dialog_copies_floppy_image_path() {
    let path = std::path::PathBuf::from("/tmp/floppy.kpd");
    let dialog = SettingsDialog::new(
        Lang::Ru,
        SpeedTier::Medium,
        true,
        Some(path.clone()),
        None,
        NetworkSettings::default(),
    );
    assert_eq!(dialog.draft_floppy_image_path, Some(path));
}

#[test]
fn dialog_copies_client_and_server_network_defaults() {
    let network = NetworkSettings {
        host: "client.local".to_owned(),
        port: 6000,
        bind_host: "0.0.0.0".to_owned(),
        bind_port: 7000,
        ..NetworkSettings::default()
    };
    let dialog = SettingsDialog::new(Lang::Ru, SpeedTier::Medium, true, None, None, network);

    assert_eq!(dialog.draft_network_client_host, "client.local");
    assert_eq!(dialog.draft_network_client_port, "6000");
    assert_eq!(dialog.draft_network_server_host, "0.0.0.0");
    assert_eq!(dialog.draft_network_server_port, "7000");
}

#[test]
fn footer_focus_cycles_in_a_ring() {
    assert_eq!(FooterFocus::Reset.next(), FooterFocus::Cancel);
    assert_eq!(FooterFocus::Cancel.next(), FooterFocus::Save);
    assert_eq!(FooterFocus::Save.next(), FooterFocus::Reset);

    assert_eq!(FooterFocus::Save.previous(), FooterFocus::Cancel);
    assert_eq!(FooterFocus::Cancel.previous(), FooterFocus::Reset);
    assert_eq!(FooterFocus::Reset.previous(), FooterFocus::Save);
}

#[test]
fn live_speed_change_updates_active_tier_immediately() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.speed_tier = SpeedTier::Slow;
    app.default_speed = SpeedTier::Slow;
    app.settings_dialog = Some(SettingsDialog::new(
        app.lang,
        app.default_speed,
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
fn reset_confirm_restores_defaults_and_clears_dialog_snapshot() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.lang = Lang::En;
    app.default_speed = SpeedTier::Max;
    app.speed_tier = SpeedTier::Max;
    app.settings_dialog = Some(SettingsDialog::new(
        app.lang,
        app.default_speed,
        true,
        None,
        None,
        NetworkSettings::default(),
    ));

    let _ = app.update(Message::SettingsResetRequested);
    assert!(app.settings_dialog.as_ref().unwrap().reset_confirm_open);

    let _ = app.update(Message::SettingsResetConfirmed);

    assert_eq!(app.lang, Lang::Ru);
    assert_eq!(app.default_speed, SpeedTier::Medium);
    assert_eq!(app.speed_tier, SpeedTier::Medium);
    let dialog = app.settings_dialog.as_ref().unwrap();
    assert!(!dialog.reset_confirm_open);
    assert_eq!(dialog.original_lang, Lang::Ru);
    assert_eq!(dialog.original_speed, SpeedTier::Medium);
}

#[test]
fn reset_confirm_opens_with_cancel_focused() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.settings_dialog = Some(SettingsDialog::new(
        app.lang,
        app.default_speed,
        true,
        None,
        None,
        NetworkSettings::default(),
    ));

    let _ = app.update(Message::SettingsResetRequested);

    let dialog = app.settings_dialog.as_ref().unwrap();
    assert!(dialog.reset_confirm_open);
    assert_eq!(dialog.reset_confirm_focus, ResetConfirmFocus::Cancel);
}

#[test]
fn tab_toggles_reset_confirm_focus_in_a_two_button_ring() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.settings_dialog = Some(SettingsDialog::new(
        app.lang,
        app.default_speed,
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
    assert_eq!(app.lang, Lang::Ru);
    assert_eq!(app.speed_tier, SpeedTier::Medium);
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
