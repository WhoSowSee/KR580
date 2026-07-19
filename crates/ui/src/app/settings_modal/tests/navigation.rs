use super::super::dialog::SettingsDialog;
use super::super::focus::{ContentFocus, FooterFocus, SettingsCategory};
use crate::app::messages::SpeedTier;
use crate::i18n::Lang;
use crate::persistence::NetworkSettings;

#[test]
fn footer_focus_defaults_to_cancel() {
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
        Some(ContentFocus::PrinterDialogMode)
    );
    assert_eq!(
        dialog.next_content_focus(ContentFocus::PrinterDialogMode),
        Some(ContentFocus::NetworkDefaults)
    );
    assert_eq!(
        dialog.previous_content_focus(ContentFocus::NetworkDefaults),
        Some(ContentFocus::PrinterDialogMode)
    );
    assert_eq!(
        dialog.previous_content_focus(ContentFocus::PrinterDialogMode),
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
