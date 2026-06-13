use crate::SettingsError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const SETTINGS_VERSION: u32 = 2;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub settings_version: u32,
    pub network: NetworkSettings,
    pub storage: StorageSettings,
    pub export: ExportSettings,
    pub ui: UiSettings,
    pub general: GeneralSettings,
    pub recent_files: Vec<PathBuf>,
}

/// General-purpose user preferences exposed via the in-app Settings dialog.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneralSettings {
    pub language: Language,
    pub default_speed: SpeedPreset,
    pub follow_pc: bool,
    pub hdd_directory: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Language {
    Ru,
    En,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SpeedPreset {
    Slow,
    Medium,
    High,
    Max,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkSettings {
    pub mode: NetworkMode,
    pub host: String,
    pub port: u16,
    pub bind_host: String,
    pub bind_port: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NetworkMode {
    Client,
    Server,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageSettings {
    pub floppy_path: PathBuf,
    pub hdd_path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportSettings {
    pub default_directory: PathBuf,
    pub line_endings: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiSettings {
    pub theme: String,
    pub ram_view_base: u16,
}

pub struct SettingsStore;

impl Default for Settings {
    fn default() -> Self {
        Self {
            settings_version: SETTINGS_VERSION,
            network: NetworkSettings::default(),
            storage: StorageSettings::default(),
            export: ExportSettings::default(),
            ui: UiSettings::default(),
            general: GeneralSettings::default(),
            recent_files: Vec::new(),
        }
    }
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            language: Language::Ru,
            default_speed: SpeedPreset::Medium,
            follow_pc: true,
            hdd_directory: None,
        }
    }
}

impl Default for NetworkSettings {
    fn default() -> Self {
        Self {
            mode: NetworkMode::Client,
            host: "127.0.0.1".to_owned(),
            port: 5800,
            bind_host: "127.0.0.1".to_owned(),
            bind_port: 5800,
        }
    }
}

impl Default for StorageSettings {
    fn default() -> Self {
        Self {
            floppy_path: PathBuf::from("floppy.kpd"),
            hdd_path: PathBuf::from("hdd.kpd"),
        }
    }
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            default_directory: PathBuf::from("."),
            line_endings: "LF".to_owned(),
        }
    }
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            theme: "dark".to_owned(),
            ram_view_base: 0,
        }
    }
}

impl SettingsStore {
    pub fn to_json(settings: &Settings) -> Result<String, SettingsError> {
        Ok(serde_json::to_string_pretty(settings)?)
    }

    pub fn from_json(json: &str) -> Result<Settings, SettingsError> {
        let mut settings: Settings = serde_json::from_str(json)?;
        match settings.settings_version {
            SETTINGS_VERSION => {}
            1 => {
                settings.settings_version = SETTINGS_VERSION;
                settings.network = NetworkSettings::default();
            }
            version => return Err(SettingsError::UnsupportedVersion(version)),
        }
        Ok(settings)
    }

    pub fn save(path: impl AsRef<Path>, settings: &Settings) -> Result<(), SettingsError> {
        std::fs::write(path, Self::to_json(settings)?)?;
        Ok(())
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Settings, SettingsError> {
        Self::from_json(&std::fs::read_to_string(path)?)
    }
}
