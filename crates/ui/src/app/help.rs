use crate::i18n::{Key, Lang};
use iced::widget::text_editor;
use std::collections::BTreeSet;
use std::sync::Arc;

mod dialog;
pub(crate) mod markdown;
mod search;
#[cfg(test)]
mod tests;

pub(crate) use markdown::{
    HelpMarkdownHighlight, HelpMarkdownHighlighter, HelpMarkdownLine, parse_help_markdown_line,
};
pub(crate) use search::{HelpSearchResponse, HelpSearchResult, run_help_search};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum HelpNode {
    CatIntroduction,
    TopicAbout,
    TopicFeatures,
    TopicGeneralPrinciples,
    CatProgramInterface,
    TopicMainWindow,
    TopicRamEditing,
    TopicRegisterEditing,
    TopicRunButtons,
    TopicMemorySearch,
    CatMainMenu,
    TopicMenuFile,
    TopicMenuMpSystem,
    TopicMenuView,
    TopicMenuHelp,
    CatFilesExport,
    TopicSaveLoad,
    TopicImport,
    TopicExport,
    CatExternalDevices,
    TopicMonitor,
    TopicFloppy,
    TopicHdd,
    TopicNetwork,
    TopicPrinter,
    CatSettings,
    TopicGeneralSettings,
    TopicAppearance,
    TopicShortcuts,
    CatCpuArchitecture,
    TopicRegisters,
    TopicFlagsRegister,
    TopicMemoryIoSpaces,
    CatInstructionSet,
    TopicCommandSummary,
    TopicDataTransferCommands,
    TopicLogicalCommands,
    TopicArithmeticCommands,
    TopicControlTransferCommands,
    TopicProcessorControlCommands,
    TopicIoCommands,
    TopicStackCommands,
}

impl HelpNode {
    pub(crate) const ROOTS: [Self; 8] = [
        Self::CatIntroduction,
        Self::CatProgramInterface,
        Self::CatMainMenu,
        Self::CatFilesExport,
        Self::CatExternalDevices,
        Self::CatSettings,
        Self::CatCpuArchitecture,
        Self::CatInstructionSet,
    ];

    pub(crate) fn children(self) -> &'static [Self] {
        match self {
            Self::CatIntroduction => &[
                Self::TopicAbout,
                Self::TopicFeatures,
                Self::TopicGeneralPrinciples,
            ],
            Self::CatCpuArchitecture => &[
                Self::TopicRegisters,
                Self::TopicFlagsRegister,
                Self::TopicMemoryIoSpaces,
            ],
            Self::CatInstructionSet => &[
                Self::TopicCommandSummary,
                Self::TopicDataTransferCommands,
                Self::TopicLogicalCommands,
                Self::TopicArithmeticCommands,
                Self::TopicControlTransferCommands,
                Self::TopicProcessorControlCommands,
                Self::TopicIoCommands,
                Self::TopicStackCommands,
            ],
            Self::CatProgramInterface => &[
                Self::TopicMainWindow,
                Self::TopicRamEditing,
                Self::TopicRegisterEditing,
                Self::TopicRunButtons,
                Self::TopicMemorySearch,
            ],
            Self::CatMainMenu => &[
                Self::TopicMenuFile,
                Self::TopicMenuMpSystem,
                Self::TopicMenuView,
                Self::TopicMenuHelp,
            ],
            Self::CatExternalDevices => &[
                Self::TopicMonitor,
                Self::TopicFloppy,
                Self::TopicHdd,
                Self::TopicNetwork,
                Self::TopicPrinter,
            ],
            Self::CatFilesExport => &[Self::TopicSaveLoad, Self::TopicImport, Self::TopicExport],
            Self::CatSettings => &[
                Self::TopicGeneralSettings,
                Self::TopicAppearance,
                Self::TopicShortcuts,
            ],
            _ => &[],
        }
    }

    pub(crate) fn is_category(self) -> bool {
        !self.children().is_empty()
    }

    pub(crate) fn label_key(self) -> Key {
        match self {
            Self::CatIntroduction => Key::HnIntroduction,
            Self::TopicAbout => Key::HnAbout,
            Self::TopicFeatures => Key::HnFeatures,
            Self::TopicGeneralPrinciples => Key::HnGeneralPrinciples,
            Self::CatCpuArchitecture => Key::HnCpuArchitecture,
            Self::TopicRegisters => Key::HnRegisters,
            Self::TopicFlagsRegister => Key::HnFlagsRegister,
            Self::TopicMemoryIoSpaces => Key::HnMemoryIoSpaces,
            Self::CatInstructionSet => Key::HnInstructionSet,
            Self::TopicDataTransferCommands => Key::HnDataTransferCommands,
            Self::TopicLogicalCommands => Key::HnLogicalCommands,
            Self::TopicArithmeticCommands => Key::HnArithmeticCommands,
            Self::TopicControlTransferCommands => Key::HnControlTransferCommands,
            Self::TopicProcessorControlCommands => Key::HnProcessorControlCommands,
            Self::TopicIoCommands => Key::HnIoCommands,
            Self::TopicStackCommands => Key::HnStackCommands,
            Self::CatProgramInterface => Key::HnProgramInterface,
            Self::TopicMainWindow => Key::HnMainWindow,
            Self::TopicRamEditing => Key::HnRamEditing,
            Self::TopicRegisterEditing => Key::HnRegisterEditing,
            Self::TopicRunButtons => Key::HnRunButtons,
            Self::TopicMemorySearch => Key::HnMemorySearch,
            Self::CatMainMenu => Key::HnMainMenu,
            Self::TopicMenuFile => Key::HnMenuFile,
            Self::TopicMenuMpSystem => Key::HnMenuMpSystem,
            Self::TopicMenuView => Key::HnMenuView,
            Self::TopicMenuHelp => Key::HnMenuHelp,
            Self::CatFilesExport => Key::HnFilesExport,
            Self::TopicSaveLoad => Key::HnSaveLoad,
            Self::TopicImport => Key::HnImport,
            Self::TopicExport => Key::HnExport,
            Self::CatExternalDevices => Key::HnExternalDevices,
            Self::TopicMonitor => Key::HnMonitor,
            Self::TopicFloppy => Key::HnFloppy,
            Self::TopicHdd => Key::HnHdd,
            Self::TopicNetwork => Key::HnNetwork,
            Self::TopicPrinter => Key::HnPrinter,
            Self::CatSettings => Key::HnSettings,
            Self::TopicGeneralSettings => Key::HnGeneralSettings,
            Self::TopicAppearance => Key::HnAppearance,
            Self::TopicShortcuts => Key::HnTopicShortcuts,
            Self::TopicCommandSummary => Key::HnCommandSummary,
        }
    }

    pub(crate) fn content_key(self) -> Key {
        match self {
            Self::TopicAbout => Key::HcAbout,
            Self::TopicFeatures => Key::HcFeatures,
            Self::TopicGeneralPrinciples => Key::HcGeneralPrinciples,
            Self::TopicRegisters => Key::HcRegisters,
            Self::TopicFlagsRegister => Key::HcFlagsRegister,
            Self::TopicMemoryIoSpaces => Key::HcMemoryIoSpaces,
            Self::TopicDataTransferCommands => Key::HcDataTransferCommands,
            Self::TopicLogicalCommands => Key::HcLogicalCommands,
            Self::TopicArithmeticCommands => Key::HcArithmeticCommands,
            Self::TopicControlTransferCommands => Key::HcControlTransferCommands,
            Self::TopicProcessorControlCommands => Key::HcProcessorControlCommands,
            Self::TopicIoCommands => Key::HcIoCommands,
            Self::TopicStackCommands => Key::HcStackCommands,
            Self::TopicMainWindow => Key::HcMainWindow,
            Self::TopicRamEditing => Key::HcRamEditing,
            Self::TopicRegisterEditing => Key::HcRegisterEditing,
            Self::TopicRunButtons => Key::HcRunButtons,
            Self::TopicMemorySearch => Key::HcMemorySearch,
            Self::TopicMenuFile => Key::HcMenuFile,
            Self::TopicMenuMpSystem => Key::HcMenuMpSystem,
            Self::TopicMenuView => Key::HcMenuView,
            Self::TopicMenuHelp => Key::HcMenuHelp,
            Self::TopicSaveLoad => Key::HcSaveLoad,
            Self::TopicImport => Key::HcImport,
            Self::TopicExport => Key::HcExport,
            Self::TopicMonitor => Key::HcMonitor,
            Self::TopicFloppy => Key::HcFloppy,
            Self::TopicHdd => Key::HcHdd,
            Self::TopicNetwork => Key::HcNetwork,
            Self::TopicPrinter => Key::HcPrinter,
            Self::TopicGeneralSettings => Key::HcGeneralSettings,
            Self::TopicAppearance => Key::HcAppearance,
            Self::TopicCommandSummary => Key::HcCommandSummary,
            Self::TopicShortcuts => Key::HcShortcuts,
            _ => Key::HcAbout,
        }
    }
}

#[derive(Clone)]
pub(crate) struct HelpDialog {
    pub(crate) selected: HelpNode,
    pub(crate) expanded: BTreeSet<HelpNode>,
    pub(crate) search: String,
    pub(crate) article_content: text_editor::Content,
    pub(crate) article_highlights: markdown::HelpMarkdownHighlights,
    article_content_node: HelpNode,
    article_content_lang: Lang,
    search_index: Arc<search::HelpSearchIndex>,
    search_generation: u64,
    pending_search: Option<dialog::PendingHelpSearch>,
    search_results_query: String,
    search_matches: BTreeSet<HelpNode>,
    search_results: Vec<HelpSearchResult>,
}
