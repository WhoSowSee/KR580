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
mod help_en;
mod help_ru;
mod keys;
mod network;
mod printer;
mod ru;

pub(crate) use keys::Key;
pub(crate) use network::NetworkKey;
pub(crate) use printer::PrinterKey;

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

    #[test]
    fn help_strings_use_en_dash_instead_of_em_dash() {
        const HELP_KEYS: &[Key] = &[
            Key::HelpDialogTitle,
            Key::HelpSearchPlaceholder,
            Key::HnIntroduction,
            Key::HnAbout,
            Key::HnFeatures,
            Key::HnSystemComposition,
            Key::HnSystemComponents,
            Key::HnCpuArchitecture,
            Key::HnArchitecture,
            Key::HnRegisters,
            Key::HnFlagsRegister,
            Key::HnMemoryIoSpaces,
            Key::HnInstructionSet,
            Key::HnDataTransferCommands,
            Key::HnLogicalCommands,
            Key::HnArithmeticCommands,
            Key::HnControlTransferCommands,
            Key::HnProcessorControlCommands,
            Key::HnIoCommands,
            Key::HnStackCommands,
            Key::HnProgramInterface,
            Key::HnMainWindow,
            Key::HnMainMenu,
            Key::HnMenuFile,
            Key::HnMenuMpSystem,
            Key::HnMenuHelp,
            Key::HnSchematic,
            Key::HnRamTable,
            Key::HnExternalDevices,
            Key::HnMonitor,
            Key::HnFloppy,
            Key::HnHdd,
            Key::HnNetwork,
            Key::HnPrinter,
            Key::HnRamEditing,
            Key::HnRegisterEditing,
            Key::HnResetButtons,
            Key::HnCommandPanel,
            Key::HnRunButtons,
            Key::HnFilesExport,
            Key::HnSaveLoad,
            Key::HnImport,
            Key::HnExport,
            Key::HnFileFormats,
            Key::HnSettings,
            Key::HnGeneralSettings,
            Key::HnAppearance,
            Key::HnWorkflow,
            Key::HnGeneralPrinciples,
            Key::HnMemorySearch,
            Key::HnRegisterEdit,
            Key::HnDeviceWorkflow,
            Key::HnCommandReference,
            Key::HnCommandSummary,
            Key::HnShortcuts,
            Key::HnTopicShortcuts,
            Key::HcAbout,
            Key::HcFeatures,
            Key::HcSystemComponents,
            Key::HcArchitecture,
            Key::HcRegisters,
            Key::HcFlagsRegister,
            Key::HcMemoryIoSpaces,
            Key::HcDataTransferCommands,
            Key::HcLogicalCommands,
            Key::HcArithmeticCommands,
            Key::HcControlTransferCommands,
            Key::HcProcessorControlCommands,
            Key::HcIoCommands,
            Key::HcStackCommands,
            Key::HcMainWindow,
            Key::HcMenuFile,
            Key::HcMenuMpSystem,
            Key::HcMenuHelp,
            Key::HcSchematic,
            Key::HcRamTable,
            Key::HcMonitor,
            Key::HcFloppy,
            Key::HcHdd,
            Key::HcNetwork,
            Key::HcPrinter,
            Key::HcRamEditing,
            Key::HcRegisterEditing,
            Key::HcResetButtons,
            Key::HcCommandPanel,
            Key::HcRunButtons,
            Key::HcSaveLoad,
            Key::HcImport,
            Key::HcExport,
            Key::HcFileFormats,
            Key::HcGeneralSettings,
            Key::HcAppearance,
            Key::HcGeneralPrinciples,
            Key::HcMemorySearch,
            Key::HcRegisterEdit,
            Key::HcDeviceWorkflow,
            Key::HcCommandSummary,
            Key::HcShortcuts,
        ];

        for lang in [Lang::Ru, Lang::En] {
            for key in HELP_KEYS {
                assert!(!lang.t(*key).contains('—'), "{lang:?} {key:?}");
            }
        }
    }
}
