//! UI localization registry.
//!
//! All user-facing strings live in per-language tables (`ru.rs` /
//! `en.rs`). Views look up text by calling `lang.t(Key::...)`; the
//! `Lang` enum is owned by `DesktopApp` so the whole UI redraws once
//! the user picks a different language in the Settings dialog.
//!
//! Persistence stores the user's choice as `k580_persistence::Language`
//! and the two enums map to each other via [`Lang::from_persistence`] and
//! [`Lang::to_persistence`]. Keeping the on-disk type separate from the
//! UI type means the persistence crate never has to know about strings.

mod en;
mod keys;
mod ru;

pub(crate) use keys::Key;

use k580_persistence::Language as PersistedLanguage;

/// Active UI language. Mirrors `k580_persistence::Language` so the two
/// can be converted explicitly without persistence depending on UI.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Lang {
    Ru,
    En,
}

impl Lang {
    pub(crate) fn from_persistence(value: PersistedLanguage) -> Self {
        match value {
            PersistedLanguage::Ru => Self::Ru,
            PersistedLanguage::En => Self::En,
        }
    }

    pub(crate) fn to_persistence(self) -> PersistedLanguage {
        match self {
            Self::Ru => PersistedLanguage::Ru,
            Self::En => PersistedLanguage::En,
        }
    }

    /// Translates a [`Key`] into a static string for the current language.
    pub(crate) fn t(self, key: Key) -> &'static str {
        match self {
            Self::Ru => ru::translate(key),
            Self::En => en::translate(key),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Key, Lang};
    use k580_persistence::Language as PersistedLanguage;

    #[test]
    fn russian_and_english_resolve_distinct_strings() {
        assert_eq!(Lang::Ru.t(Key::MenuSettings), "Настройки");
        assert_eq!(Lang::En.t(Key::MenuSettings), "Settings");
    }

    #[test]
    fn lang_round_trips_through_persistence() {
        for lang in [Lang::Ru, Lang::En] {
            assert_eq!(Lang::from_persistence(lang.to_persistence()), lang);
        }
        assert_eq!(Lang::from_persistence(PersistedLanguage::Ru), Lang::Ru);
        assert_eq!(Lang::from_persistence(PersistedLanguage::En), Lang::En);
    }
}
