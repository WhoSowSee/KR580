use super::*;

#[test]
fn reset_confirm_restores_defaults_and_clears_dialog_snapshot() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.preferences.lang = Lang::En;
    app.preferences.default_speed = SpeedTier::Max;
    app.execution.speed_tier = SpeedTier::Max;
    app.printer_setup.printer_dialog_mode = PrinterDialogMode::System;
    app.panels.monitor_split = true;
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
    let dialog = app.preferences.settings_dialog.as_mut().unwrap();
    dialog.draft_printer_dialog_mode = PrinterDialogMode::System;
    dialog.original_printer_dialog_mode = PrinterDialogMode::System;
    dialog.draft_monitor_split = true;
    dialog.original_monitor_split = true;

    let _ = app.update(Message::SettingsResetRequested);
    assert!(
        app.preferences
            .settings_dialog
            .as_ref()
            .unwrap()
            .reset_confirm_open
    );
    let _ = app.update(Message::SettingsResetConfirmed);
    crate::app::test_support::settle_files(&mut app);

    let expected_lang = lang_from_language(default_language());
    assert_eq!(app.preferences.lang, expected_lang);
    assert_eq!(app.preferences.default_speed, SpeedTier::High);
    assert_eq!(app.execution.speed_tier, SpeedTier::High);
    let dialog = app.preferences.settings_dialog.as_ref().unwrap();
    assert!(!dialog.reset_confirm_open);
    assert_eq!(dialog.original_lang, expected_lang);
    assert_eq!(dialog.original_speed, SpeedTier::High);
    assert_eq!(dialog.original_active_speed, SpeedTier::High);
    assert!(!app.preferences.follow_pc);
    assert!(!dialog.original_follow_pc);
    assert!(app.preferences.memory_operand_highlighting);
    assert!(dialog.original_memory_operand_highlighting);
    assert!(!app.panels.monitor_split);
    assert!(!dialog.draft_monitor_split);
    assert!(!dialog.original_monitor_split);
    assert_eq!(
        app.printer_setup.printer_dialog_mode,
        PrinterDialogMode::Custom
    );
    assert_eq!(dialog.draft_printer_dialog_mode, PrinterDialogMode::Custom);
    assert_eq!(
        dialog.original_printer_dialog_mode,
        PrinterDialogMode::Custom
    );
    assert_eq!(
        app.preferences
            .shortcut_settings
            .binding(ShortcutAction::OpenMonitor),
        Some(ShortcutBinding::new(true, false, false, ShortcutKey::M))
    );
    assert_eq!(
        app.preferences.settings_notice.unwrap().message_key(),
        Key::SettingsResetNotice
    );
}

#[test]
fn reset_confirm_opens_with_cancel_focused() {
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

    let _ = app.update(Message::SettingsResetRequested);

    let dialog = app.preferences.settings_dialog.as_ref().unwrap();
    assert!(dialog.reset_confirm_open);
    assert_eq!(dialog.reset_confirm_focus, ResetConfirmFocus::Cancel);
    assert!(!dialog.reset_confirm_keyboard_focus_visible);
}

#[test]
fn tab_toggles_reset_confirm_focus_in_a_two_button_ring() {
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
    let _ = app.update(Message::SettingsResetRequested);

    let _ = app.update(Message::FocusCycle { backward: false });
    assert_eq!(
        app.preferences
            .settings_dialog
            .as_ref()
            .unwrap()
            .reset_confirm_focus,
        ResetConfirmFocus::Confirm
    );
    assert!(
        app.preferences
            .settings_dialog
            .as_ref()
            .unwrap()
            .reset_confirm_keyboard_focus_visible
    );

    let _ = app.update(Message::FocusCycle { backward: false });
    assert_eq!(
        app.preferences
            .settings_dialog
            .as_ref()
            .unwrap()
            .reset_confirm_focus,
        ResetConfirmFocus::Cancel
    );

    let _ = app.update(Message::FocusCycle { backward: true });
    assert_eq!(
        app.preferences
            .settings_dialog
            .as_ref()
            .unwrap()
            .reset_confirm_focus,
        ResetConfirmFocus::Confirm
    );
}

#[test]
fn enter_in_reset_confirm_activates_focused_button() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.preferences.lang = Lang::En;
    app.preferences.default_speed = SpeedTier::Max;
    app.execution.speed_tier = SpeedTier::Max;
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

    let _ = app.update(Message::SettingsResetRequested);
    assert_eq!(
        app.preferences
            .settings_dialog
            .as_ref()
            .unwrap()
            .reset_confirm_focus,
        ResetConfirmFocus::Cancel
    );
    let _ = app.update(Message::SettingsResetCancelled);
    assert!(
        !app.preferences
            .settings_dialog
            .as_ref()
            .unwrap()
            .reset_confirm_open
    );
    assert_eq!(app.preferences.lang, Lang::En);
    assert_eq!(app.execution.speed_tier, SpeedTier::Max);

    let _ = app.update(Message::SettingsResetRequested);
    let _ = app.update(Message::FocusCycle { backward: false });
    assert_eq!(
        app.preferences
            .settings_dialog
            .as_ref()
            .unwrap()
            .reset_confirm_focus,
        ResetConfirmFocus::Confirm
    );
    let _ = app.update(Message::SettingsResetConfirmed);
    crate::app::test_support::settle_files(&mut app);
    assert_eq!(app.preferences.lang, lang_from_language(default_language()));
    assert_eq!(app.execution.speed_tier, SpeedTier::High);
}
