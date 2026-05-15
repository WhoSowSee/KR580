use crate::SettingsError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub settings_version: u32,
    pub network: NetworkSettings,
    pub storage: StorageSettings,
    pub export: ExportSettings,
    pub ui: UiSettings,
    pub recent_files: Vec<PathBuf>,
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
            settings_version: 1,
            network: NetworkSettings::default(),
            storage: StorageSettings::default(),
            export: ExportSettings::default(),
            ui: UiSettings::default(),
            recent_files: Vec::new(),
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
        let settings: Settings = serde_json::from_str(json)?;
        if settings.settings_version != 1 {
            return Err(SettingsError::UnsupportedVersion(settings.settings_version));
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
