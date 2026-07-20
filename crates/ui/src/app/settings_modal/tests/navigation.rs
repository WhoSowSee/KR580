use super::super::dialog::SettingsDialog;
use super::super::focus::{ContentFocus, FooterFocus, SettingsCategory, SettingsSection};
use crate::app::messages::SpeedTier;
use crate::app::{DesktopApp, Message};
use crate::i18n::Lang;
use crate::persistence::NetworkSettings;

#[test]
fn settings_focus_defaults_to_language_without_visible_ring() {
    let dialog = SettingsDialog::new(
        Lang::Ru,
        SpeedTier::Medium,
        true,
        true,
        None,
        None,
        NetworkSettings::default(),
    );
    assert_eq!(dialog.footer_focus, FooterFocus::Cancel);
    assert_eq!(dialog.section, SettingsSection::Content);
    assert_eq!(dialog.content_focus, Some(ContentFocus::LanguageAnchor));
    assert_eq!(dialog.sidebar_focus, SettingsCategory::General);
    assert!(!dialog.keyboard_focus_visible);
}

#[test]
fn general_toggle_segments_are_individually_tab_indexed() {
    let dialog = SettingsDialog::new(
        Lang::Ru,
        SpeedTier::Medium,
        true,
        true,
        None,
        None,
        NetworkSettings::default(),
    );

    assert_eq!(
        dialog.next_content_focus(ContentFocus::SpeedMax),
        Some(ContentFocus::FollowPcOn)
    );
    assert_eq!(
        dialog.next_content_focus(ContentFocus::FollowPcOn),
        Some(ContentFocus::FollowPcOff)
    );
    assert_eq!(
        dialog.next_content_focus(ContentFocus::FollowPcOff),
        Some(ContentFocus::MemoryOperandHighlightingOn)
    );
    assert_eq!(
        dialog.next_content_focus(ContentFocus::MemoryOperandHighlightingOn),
        Some(ContentFocus::MemoryOperandHighlightingOff)
    );
    assert_eq!(
        dialog.next_content_focus(ContentFocus::MemoryOperandHighlightingOff),
        Some(ContentFocus::FileAssociation)
    );
    assert_eq!(dialog.last_content_focus(), ContentFocus::FileAssociation);
}

#[test]
fn sidebar_tab_moves_cursor_without_activating_category() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    let mut dialog = SettingsDialog::new(
        Lang::Ru,
        SpeedTier::Medium,
        true,
        true,
        None,
        None,
        NetworkSettings::default(),
    );
    dialog.section = SettingsSection::Sidebar;
    app.settings_dialog = Some(dialog);

    let _ = app.update(Message::FocusCycle { backward: false });
    let dialog = app.settings_dialog.as_ref().unwrap();
    assert_eq!(dialog.category, SettingsCategory::General);
    assert_eq!(dialog.sidebar_focus, SettingsCategory::ExternalDevices);
    assert!(dialog.keyboard_focus_visible);

    let _ = app.update(Message::EnterPressed);
    assert!(!app.settings_dialog.as_ref().unwrap().keyboard_focus_visible);
    let _ = app.update(Message::SettingsCategorySelected(
        SettingsCategory::ExternalDevices,
    ));
    let dialog = app.settings_dialog.as_ref().unwrap();
    assert_eq!(dialog.category, SettingsCategory::ExternalDevices);
    assert_eq!(dialog.content_focus, Some(ContentFocus::FloppyImage));
}

#[test]
fn external_devices_focus_keeps_printer_before_network() {
    let mut dialog = SettingsDialog::new(
        Lang::Ru,
        SpeedTier::Medium,
        true,
        true,
        None,
        None,
        NetworkSettings::default(),
    );
    dialog.category = SettingsCategory::ExternalDevices;

    assert_eq!(dialog.first_content_focus(), ContentFocus::FloppyImage);
    assert_eq!(dialog.last_content_focus(), ContentFocus::NetworkDefaults);
    assert_eq!(
        dialog.next_content_focus(ContentFocus::HddDirectory),
        Some(ContentFocus::PrinterDefault)
    );
    assert_eq!(
        dialog.next_content_focus(ContentFocus::PrinterDefault),
        Some(ContentFocus::PrinterDialogModeCustom)
    );
    assert_eq!(
        dialog.next_content_focus(ContentFocus::PrinterDialogModeCustom),
        Some(ContentFocus::PrinterDialogModeSystem)
    );
    assert_eq!(
        dialog.next_content_focus(ContentFocus::PrinterDialogModeSystem),
        Some(ContentFocus::NetworkDefaults)
    );
    assert_eq!(
        dialog.previous_content_focus(ContentFocus::NetworkDefaults),
        Some(ContentFocus::PrinterDialogModeSystem)
    );
    assert_eq!(
        dialog.previous_content_focus(ContentFocus::PrinterDialogModeSystem),
        Some(ContentFocus::PrinterDialogModeCustom)
    );
    assert_eq!(
        dialog.previous_content_focus(ContentFocus::PrinterDialogModeCustom),
        Some(ContentFocus::PrinterDefault)
    );
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
fn shortcut_footer_focus_includes_shortcut_reset() {
    assert_eq!(
        FooterFocus::Reset.next_with_shortcuts(true),
        FooterFocus::ShortcutReset
    );
    assert_eq!(
        FooterFocus::ShortcutReset.next_with_shortcuts(true),
        FooterFocus::Cancel
    );
    assert_eq!(
        FooterFocus::Cancel.previous_with_shortcuts(true),
        FooterFocus::ShortcutReset
    );
    assert_eq!(
        FooterFocus::ShortcutReset.previous_with_shortcuts(true),
        FooterFocus::Reset
    );
}
