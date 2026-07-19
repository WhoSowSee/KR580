use crate::app::messages::SpeedTier;
use crate::app::settings_modal::{SettingsCategory, SettingsDialog};
use crate::i18n::Lang;
use crate::persistence::NetworkSettings;

#[test]
fn dialog_starts_on_general_category() {
    let dialog = SettingsDialog::new(
        Lang::Ru,
        SpeedTier::Medium,
        true,
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
        true,
        None,
        None,
        NetworkSettings::default(),
    );
    dialog.search = "  скорость  ".to_owned();
    assert_eq!(dialog.search_query(), "скорость");
}

#[test]
fn dialog_copies_floppy_image_path() {
    let path = std::path::PathBuf::from("/tmp/floppy.kpd");
    let dialog = SettingsDialog::new(
        Lang::Ru,
        SpeedTier::Medium,
        true,
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
    let dialog = SettingsDialog::new(Lang::Ru, SpeedTier::Medium, true, true, None, None, network);

    assert_eq!(dialog.draft_network_client_host, "client.local");
    assert_eq!(dialog.draft_network_client_port, "6000");
    assert_eq!(dialog.draft_network_server_host, "0.0.0.0");
    assert_eq!(dialog.draft_network_server_port, "7000");
}
