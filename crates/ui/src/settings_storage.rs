//! Bridge between `k580_persistence::Settings` on disk and the runtime
//! state used by `DesktopApp`.
//!
//! Two responsibilities live here, kept narrow on purpose:
//!
//! 1. **Path resolution** – portable installs keep settings under the
//!    install root, while system installs use the platform config
//!    directory. Falls back beside the executable for unpacked builds.
//! 2. **Type translation** – `SpeedPreset` ↔ `SpeedTier`, `Language` ↔
//!    `Lang`. Persistence has no concept of `SpeedTier` (UI-only) and the
//!    UI does not pull in serde, so the mapping is centralized here.
//!
//! Failures during load are non-fatal: the user gets defaults rather than
//! a blocked startup.

use crate::app::messages::SpeedTier;
use crate::i18n::Lang;
use k580_persistence::{Language, Settings, SettingsError, SettingsStore, SpeedPreset};
use k580_ui::install_mode::{InstallMode, manifest_for_executable};
use std::path::{Path, PathBuf};

const SETTINGS_FILENAME: &str = "settings.json";

pub(crate) fn settings_path() -> PathBuf {
    if let Ok(exe) = std::env::current_exe()
        && let Some(path) = settings_path_for_executable(&exe)
    {
        return path;
    }
    PathBuf::from(SETTINGS_FILENAME)
}

fn settings_path_for_executable(exe: &Path) -> Option<PathBuf> {
    match manifest_for_executable(exe) {
        Ok(Some((root, manifest))) => {
            return Some(settings_path_for_install(&root, manifest.mode));
        }
        Ok(None) => {}
        Err(error) => tracing::warn!(%error, "install manifest ignored"),
    }
    exe.parent().map(|parent| parent.join(SETTINGS_FILENAME))
}

fn settings_path_for_install(root: &Path, mode: InstallMode) -> PathBuf {
    match mode {
        InstallMode::Portable => root.join("data").join(SETTINGS_FILENAME),
        InstallMode::System => system_settings_path(),
    }
}

#[cfg(windows)]
fn system_settings_path() -> PathBuf {
    std::env::var_os("APPDATA")
        .or_else(|| std::env::var_os("LOCALAPPDATA"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("KR580")
        .join(SETTINGS_FILENAME)
}

#[cfg(target_os = "macos")]
fn system_settings_path() -> PathBuf {
    home_dir()
        .join("Library")
        .join("Application Support")
        .join("KR580")
        .join(SETTINGS_FILENAME)
}

#[cfg(all(unix, not(target_os = "macos")))]
fn system_settings_path() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join(".config"))
        .join("kr580")
        .join(SETTINGS_FILENAME)
}

#[cfg(any(unix, target_os = "macos"))]
fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Loads settings without panicking. A missing file silently returns
/// `Settings::default()`; unreadable or malformed files are logged and
/// replaced with defaults too.
pub(crate) fn load_settings() -> Settings {
    let path = settings_path();
    match SettingsStore::load(&path) {
        Ok(settings) => settings,
        Err(error) => {
            if should_log_settings_load_error(&error) {
                tracing::warn!(?path, %error, "settings load failed; using defaults");
            }
            default_settings()
        }
    }
}

fn should_log_settings_load_error(error: &SettingsError) -> bool {
    !matches!(error, SettingsError::Io(source) if source.kind() == std::io::ErrorKind::NotFound)
}

/// Saves settings best-effort. Errors are logged but not surfaced to the
/// user – losing a single settings write is recoverable (defaults next
/// time) and we do not want a popup for IO hiccups.
pub(crate) fn save_settings(settings: &Settings) {
    let path = settings_path();
    if let Some(parent) = path.parent()
        && let Err(error) = std::fs::create_dir_all(parent)
    {
        tracing::warn!(?path, %error, "settings directory create failed");
        return;
    }
    if let Err(error) = SettingsStore::save(&path, settings) {
        tracing::warn!(?path, %error, "settings save failed");
    }
}

pub(crate) fn speed_tier_from_preset(preset: SpeedPreset) -> SpeedTier {
    match preset {
        SpeedPreset::Slow => SpeedTier::Slow,
        SpeedPreset::Medium => SpeedTier::Medium,
        SpeedPreset::High => SpeedTier::High,
        SpeedPreset::Max => SpeedTier::Max,
    }
}

pub(crate) fn preset_from_speed_tier(tier: SpeedTier) -> SpeedPreset {
    match tier {
        SpeedTier::Slow => SpeedPreset::Slow,
        SpeedTier::Medium => SpeedPreset::Medium,
        SpeedTier::High => SpeedPreset::High,
        SpeedTier::Max => SpeedPreset::Max,
    }
}

pub(crate) fn lang_from_language(language: Language) -> Lang {
    Lang::from_persistence(language)
}

pub(crate) fn language_from_lang(lang: Lang) -> Language {
    lang.to_persistence()
}

pub(crate) fn default_settings() -> Settings {
    let mut settings = Settings::default();
    settings.general.language = k580_ui::system_locale::default_language();
    settings
}

pub(crate) fn default_lang() -> Lang {
    lang_from_language(k580_ui::system_locale::default_language())
}

#[cfg(test)]
mod tests {
    use super::{
        default_settings, lang_from_language, language_from_lang, preset_from_speed_tier,
        settings_path_for_install, should_log_settings_load_error, speed_tier_from_preset,
    };
    use crate::app::messages::SpeedTier;
    use crate::i18n::Lang;
    use k580_persistence::{Language, SettingsError, SpeedPreset};
    use k580_ui::install_mode::InstallMode;
    use std::path::Path;

    #[test]
    fn speed_tier_round_trips_through_preset() {
        for tier in [
            SpeedTier::Slow,
            SpeedTier::Medium,
            SpeedTier::High,
            SpeedTier::Max,
        ] {
            assert_eq!(speed_tier_from_preset(preset_from_speed_tier(tier)), tier);
        }
    }

    #[test]
    fn lang_round_trips_through_language() {
        for lang in [Lang::Ru, Lang::En] {
            assert_eq!(lang_from_language(language_from_lang(lang)), lang);
        }
        assert_eq!(language_from_lang(Lang::Ru), Language::Ru);
        assert_eq!(
            speed_tier_from_preset(SpeedPreset::Medium),
            SpeedTier::Medium
        );
    }

    #[test]
    fn missing_settings_file_uses_defaults_without_warning() {
        let missing = SettingsError::Io(std::io::Error::from(std::io::ErrorKind::NotFound));
        let denied = SettingsError::Io(std::io::Error::from(std::io::ErrorKind::PermissionDenied));
        let unsupported = SettingsError::UnsupportedVersion(2);

        assert!(!should_log_settings_load_error(&missing));
        assert!(should_log_settings_load_error(&denied));
        assert!(should_log_settings_load_error(&unsupported));
    }

    #[test]
    fn default_settings_use_supported_language() {
        assert!(matches!(
            default_settings().general.language,
            Language::Ru | Language::En
        ));
    }

    #[test]
    fn portable_install_keeps_settings_under_install_root() {
        assert_eq!(
            settings_path_for_install(Path::new("/opt/kr580"), InstallMode::Portable),
            Path::new("/opt/kr580").join("data").join("settings.json")
        );
    }
}
