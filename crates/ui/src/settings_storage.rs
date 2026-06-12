//! Bridge between `k580_persistence::Settings` on disk and the runtime
//! state used by `DesktopApp`.
//!
//! Two responsibilities live here, kept narrow on purpose:
//!
//! 1. **Path resolution** – picks a fixed location next to the executable
//!    so users do not have to hunt for the config file. Falls back to the
//!    current working directory if `current_exe()` is unavailable
//!    (sandboxed environments, broken installs).
//! 2. **Type translation** – `SpeedPreset` ↔ `SpeedTier`, `Language` ↔
//!    `Lang`. Persistence has no concept of `SpeedTier` (UI-only) and the
//!    UI does not pull in serde, so the mapping is centralized here.
//!
//! Failures during load are non-fatal: the user gets defaults rather than
//! a blocked startup.

use crate::app::messages::SpeedTier;
use crate::i18n::Lang;
use k580_persistence::{Language, Settings, SettingsError, SettingsStore, SpeedPreset};
use std::path::PathBuf;

const SETTINGS_FILENAME: &str = "settings.json";

/// Picks a stable, predictable location for the settings file.
///
/// The executable directory is preferred so the settings travel with a
/// portable build. We accept the lossy assumption that `current_exe()`
/// can be canonicalised – on Windows that resolves the long path and on
/// Linux it follows symlinks, which is the right behaviour for a
/// portable install.
pub(crate) fn settings_path() -> PathBuf {
    if let Ok(exe) = std::env::current_exe()
        && let Some(parent) = exe.parent()
    {
        return parent.join(SETTINGS_FILENAME);
    }
    PathBuf::from(SETTINGS_FILENAME)
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
            Settings::default()
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

#[cfg(test)]
mod tests {
    use super::{
        lang_from_language, language_from_lang, preset_from_speed_tier,
        should_log_settings_load_error, speed_tier_from_preset,
    };
    use crate::app::messages::SpeedTier;
    use crate::i18n::Lang;
    use k580_persistence::{Language, SettingsError, SpeedPreset};

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
}
