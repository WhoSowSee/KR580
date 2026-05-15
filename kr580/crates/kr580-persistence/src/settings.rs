//! Versioned UTF-8 JSON settings.
//!
//! Per `prompt/04_file_formats.md` and `prompt/08_peripheral_edge_cases.md`:
//!
//! * top-level `settingsVersion` integer (initial value `1`);
//! * explicit network mode (`client` or `server`);
//! * storage paths read from explicit settings — *not* from the process CWD;
//! * export defaults and UI preferences;
//! * recent files list.

use crate::error::PersistenceError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Initial settings schema version.
pub const CURRENT_VERSION: u32 = 1;

/// Network mode. Always explicit: there is no "auto".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NetworkMode {
    /// Connect to a configured host and port.
    Client,
    /// Bind to a configured host and port.
    Server,
}

/// Network configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkSettings {
    /// Active network mode.
    pub mode: NetworkMode,
    /// Host or bind address (depending on mode).
    pub host: String,
    /// Port.
    pub port: u16,
}

impl Default for NetworkSettings {
    fn default() -> Self {
        Self {
            mode: NetworkMode::Client,
            host: "127.0.0.1".to_string(),
            port: 5800,
        }
    }
}

/// Storage configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageSettings {
    /// Backing file for the floppy device.
    pub floppy_path: PathBuf,
    /// Backing file for the hard disk device.
    pub hdd_path: PathBuf,
}

impl Default for StorageSettings {
    fn default() -> Self {
        Self {
            floppy_path: PathBuf::from("floppy.kpd"),
            hdd_path: PathBuf::from("hdd.kpd"),
        }
    }
}

/// Export defaults.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportSettings {
    /// Default directory for exports.
    pub default_directory: PathBuf,
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            default_directory: PathBuf::from("."),
        }
    }
}

/// UI preferences.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiSettings {
    /// Whether to show debug panels by default.
    pub show_debug_panels: bool,
    /// Display memory in 16-byte rows by default.
    pub memory_row_width: u8,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            show_debug_panels: true,
            memory_row_width: 16,
        }
    }
}

/// Top-level settings document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    /// Schema version.
    #[serde(rename = "settingsVersion")]
    pub settings_version: u32,
    /// Network configuration.
    #[serde(default)]
    pub network: NetworkSettings,
    /// Storage configuration.
    #[serde(default)]
    pub storage: StorageSettings,
    /// Export configuration.
    #[serde(default)]
    pub export: ExportSettings,
    /// UI configuration.
    #[serde(default)]
    pub ui: UiSettings,
    /// Recently opened files (snapshots, subprograms, …).
    #[serde(default, rename = "recentFiles")]
    pub recent_files: Vec<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            settings_version: CURRENT_VERSION,
            network: NetworkSettings::default(),
            storage: StorageSettings::default(),
            export: ExportSettings::default(),
            ui: UiSettings::default(),
            recent_files: Vec::new(),
        }
    }
}

/// File-backed settings store.
pub struct SettingsStore {
    path: PathBuf,
}

impl SettingsStore {
    /// Build a store rooted at the given path.
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    /// Path of the settings file.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Load settings from disk. Returns defaults if the file does not exist.
    pub fn load(&self) -> Result<Settings, PersistenceError> {
        match std::fs::read_to_string(&self.path) {
            Ok(s) => Self::parse(&s),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Settings::default()),
            Err(e) => Err(PersistenceError::Io(e)),
        }
    }

    /// Save settings to disk (creates the parent directory if missing).
    pub fn save(&self, settings: &Settings) -> Result<(), PersistenceError> {
        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(PersistenceError::Io)?;
            }
        }
        let text = serde_json::to_string_pretty(settings)
            .map_err(|e| PersistenceError::Settings(e.to_string()))?;
        std::fs::write(&self.path, text).map_err(PersistenceError::Io)?;
        Ok(())
    }

    /// Parse a settings JSON document, applying schema migrations.
    pub fn parse(text: &str) -> Result<Settings, PersistenceError> {
        let mut value: serde_json::Value =
            serde_json::from_str(text).map_err(|e| PersistenceError::Settings(e.to_string()))?;
        // Ensure settingsVersion exists; migrate if needed.
        let version = value
            .get("settingsVersion")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        if version > CURRENT_VERSION {
            return Err(PersistenceError::Settings(format!(
                "unsupported settingsVersion {version} (max {CURRENT_VERSION})"
            )));
        }
        // Future: per-version migrations would go here.
        if version == 0 {
            value["settingsVersion"] = serde_json::json!(CURRENT_VERSION);
        }
        let settings: Settings =
            serde_json::from_value(value).map_err(|e| PersistenceError::Settings(e.to_string()))?;
        Ok(settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_serialises_with_version() {
        let s = Settings::default();
        let text = serde_json::to_string(&s).unwrap();
        assert!(text.contains("\"settingsVersion\":1"));
    }

    #[test]
    fn missing_version_is_migrated_to_current() {
        let text = r#"{
            "network": {"mode":"server","host":"0.0.0.0","port":4242},
            "storage": {"floppy_path":"a.kpd","hdd_path":"b.kpd"}
        }"#;
        let s = SettingsStore::parse(text).unwrap();
        assert_eq!(s.settings_version, CURRENT_VERSION);
        assert_eq!(s.network.mode, NetworkMode::Server);
    }

    #[test]
    fn future_version_is_rejected() {
        let text = r#"{ "settingsVersion": 999 }"#;
        let err = SettingsStore::parse(text).unwrap_err();
        assert!(matches!(err, PersistenceError::Settings(_)));
    }
}
