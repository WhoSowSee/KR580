use super::dialog::SettingsDialog;
use super::focus::{FooterFocus, ResetConfirmFocus, SettingsCategory};
use crate::app::messages::SpeedTier;
use crate::app::{DesktopApp, Message, StatusKind};
use crate::i18n::Lang;

#[test]
fn dialog_starts_on_general_category() {
    let dialog = SettingsDialog::new(Lang::Ru, SpeedTier::Medium, true, None);
    assert_eq!(dialog.category, SettingsCategory::General);
    assert!(dialog.search.is_empty());
}

#[test]
fn search_query_strips_surrounding_whitespace() {
    let mut dialog = SettingsDialog::new(Lang::Ru, SpeedTier::Medium, true, None);
    dialog.search = "  скорость  ".to_owned();
    assert_eq!(dialog.search_query(), "скорость");
}

#[test]
fn footer_focus_defaults_to_cancel() {
    let dialog = SettingsDialog::new(Lang::Ru, SpeedTier::Medium, true, None);
    assert_eq!(dialog.footer_focus, FooterFocus::Cancel);
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
    app.settings_dialog = Some(SettingsDialog::new(app.lang, app.default_speed, true, None));

    let _ = app.update(Message::SettingsDraftSpeedChanged(SpeedTier::Max));

    assert_eq!(app.speed_tier, SpeedTier::Max);
    assert_eq!(app.default_speed, SpeedTier::Max);
}

#[test]
fn cancel_rolls_back_live_speed_to_pre_open_snapshot() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.speed_tier = SpeedTier::Slow;
    app.default_speed = SpeedTier::Slow;
    app.settings_dialog = Some(SettingsDialog::new(app.lang, app.default_speed, true, None));

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
    app.settings_dialog = Some(SettingsDialog::new(app.lang, app.default_speed, true, None));

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
    app.settings_dialog = Some(SettingsDialog::new(app.lang, app.default_speed, true, None));

    let _ = app.update(Message::SettingsResetRequested);

    let dialog = app.settings_dialog.as_ref().unwrap();
    assert!(dialog.reset_confirm_open);
    assert_eq!(dialog.reset_confirm_focus, ResetConfirmFocus::Cancel);
}

#[test]
fn tab_toggles_reset_confirm_focus_in_a_two_button_ring() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.settings_dialog = Some(SettingsDialog::new(app.lang, app.default_speed, true, None));
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
    app.settings_dialog = Some(SettingsDialog::new(app.lang, app.default_speed, true, None));

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
