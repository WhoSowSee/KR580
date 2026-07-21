//! UI localization registry.
//!
//! All user-facing strings live in per-language tables (`ru.rs` /
//! `en.rs`). Views look up text by calling `lang.t(Key::...)`; the
//! `Lang` enum is owned by `DesktopApp` so the whole UI redraws once
//! the user picks a different language in the Settings dialog.
//!
//! Persistence stores the user's choice as `crate::persistence::Language`
//! and the two enums map to each other via [`Lang::from_persistence`] and
//! [`Lang::to_persistence`]. Keeping the on-disk type separate from the
//! UI type means the persistence crate never has to know about strings.

mod en;
mod help_en;
mod help_ru;
mod keys;
mod network;
mod printer;
mod ru;
mod view;

pub(crate) use keys::Key;
pub(crate) use network::NetworkKey;
pub(crate) use printer::PrinterKey;

use crate::persistence::Language as PersistedLanguage;

pub(crate) fn lowercase_initial(value: &str) -> String {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return String::new();
    };
    first.to_lowercase().chain(characters).collect()
}

/// Active UI language. Mirrors `crate::persistence::Language` so the two
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

    pub(crate) fn stack_view_area_label(self, active: bool) -> &'static str {
        view::stack_view_area_label(self, active)
    }
}

#[cfg(test)]
mod tests {
    use super::{Key, Lang, lowercase_initial};
    use crate::persistence::Language as PersistedLanguage;

    #[test]
    fn russian_and_english_resolve_distinct_strings() {
        assert_eq!(Lang::Ru.t(Key::MenuSettings), "Настройки");
        assert_eq!(Lang::En.t(Key::MenuSettings), "Settings");
        assert_eq!(
            Lang::Ru.stack_view_area_label(false),
            "Показать стековую область памяти"
        );
        assert_eq!(
            Lang::Ru.stack_view_area_label(true),
            "Скрыть стековую область памяти"
        );
        assert_eq!(
            Lang::En.stack_view_area_label(false),
            "Show stack memory area"
        );
        assert_eq!(
            Lang::En.stack_view_area_label(true),
            "Hide stack memory area"
        );
    }

    #[test]
    fn lang_round_trips_through_persistence() {
        for lang in [Lang::Ru, Lang::En] {
            assert_eq!(Lang::from_persistence(lang.to_persistence()), lang);
        }
        assert_eq!(Lang::from_persistence(PersistedLanguage::Ru), Lang::Ru);
        assert_eq!(Lang::from_persistence(PersistedLanguage::En), Lang::En);
    }

    #[test]
    fn localized_footer_values_lowercase_unicode_initials() {
        assert_eq!(lowercase_initial("Refused"), "refused");
        assert_eq!(lowercase_initial("Отклонено"), "отклонено");
        assert_eq!(lowercase_initial("127.0.0.1"), "127.0.0.1");
        assert_eq!(lowercase_initial(""), "");
    }

    #[test]
    fn help_strings_use_en_dash_instead_of_em_dash() {
        const HELP_KEYS: &[Key] = &[
            Key::HelpDialogTitle,
            Key::HelpSearchPlaceholder,
            Key::HnIntroduction,
            Key::HnAbout,
            Key::HnFeatures,
            Key::HnGeneralPrinciples,
            Key::HnProgramInterface,
            Key::HnMainWindow,
            Key::HnRamEditing,
            Key::HnRegisterEditing,
            Key::HnRunButtons,
            Key::HnMemorySearch,
            Key::HnMainMenu,
            Key::HnMenuFile,
            Key::HnMenuMpSystem,
            Key::HnMenuView,
            Key::HnMenuHelp,
            Key::HnFilesExport,
            Key::HnSaveLoad,
            Key::HnImport,
            Key::HnExport,
            Key::HnExternalDevices,
            Key::HnMonitor,
            Key::HnFloppy,
            Key::HnHdd,
            Key::HnNetwork,
            Key::HnPrinter,
            Key::HnSettings,
            Key::HnGeneralSettings,
            Key::HnAppearance,
            Key::HnTopicShortcuts,
            Key::HnCpuArchitecture,
            Key::HnRegisters,
            Key::HnFlagsRegister,
            Key::HnMemoryIoSpaces,
            Key::HnInstructionSet,
            Key::HnCommandSummary,
            Key::HnDataTransferCommands,
            Key::HnLogicalCommands,
            Key::HnArithmeticCommands,
            Key::HnControlTransferCommands,
            Key::HnProcessorControlCommands,
            Key::HnIoCommands,
            Key::HnStackCommands,
            Key::HcAbout,
            Key::HcFeatures,
            Key::HcGeneralPrinciples,
            Key::HcMainWindow,
            Key::HcRamEditing,
            Key::HcRegisterEditing,
            Key::HcRunButtons,
            Key::HcMemorySearch,
            Key::HcMenuFile,
            Key::HcMenuMpSystem,
            Key::HcMenuView,
            Key::HcMenuHelp,
            Key::HcSaveLoad,
            Key::HcImport,
            Key::HcExport,
            Key::HcMonitor,
            Key::HcFloppy,
            Key::HcHdd,
            Key::HcNetwork,
            Key::HcPrinter,
            Key::HcGeneralSettings,
            Key::HcAppearance,
            Key::HcShortcuts,
            Key::HcRegisters,
            Key::HcFlagsRegister,
            Key::HcMemoryIoSpaces,
            Key::HcCommandSummary,
            Key::HcDataTransferCommands,
            Key::HcLogicalCommands,
            Key::HcArithmeticCommands,
            Key::HcControlTransferCommands,
            Key::HcProcessorControlCommands,
            Key::HcIoCommands,
            Key::HcStackCommands,
        ];

        for lang in [Lang::Ru, Lang::En] {
            for key in HELP_KEYS {
                assert!(!lang.t(*key).contains('—'), "{lang:?} {key:?}");
            }
        }
    }
}
