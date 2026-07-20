use super::focus::{
    ContentFocus, FooterFocus, ResetConfirmFocus, SettingsCategory, SettingsSection,
};
use crate::app::messages::SpeedTier;
use crate::i18n::Lang;
use crate::persistence::{
    ColorScheme, NetworkSettings, PrinterDialogMode, ShortcutAction, ShortcutSettings,
};
use k580_ui::devices::printer::PrinterSettings;

/// Draft state edited by the dialog. Live language, speed, and shortcut
/// fields on `DesktopApp` are kept in sync for immediate preview;
/// `original_*` snapshots let Cancel / backdrop click roll that back.
#[derive(Clone, Debug)]
pub(crate) struct SettingsDialog {
    pub(crate) category: SettingsCategory,
    pub(crate) sidebar_focus: SettingsCategory,
    pub(crate) search: String,
    pub(crate) draft_lang: Lang,
    pub(crate) draft_speed: SpeedTier,
    pub(crate) draft_color_scheme: ColorScheme,
    pub(crate) draft_follow_pc: bool,
    pub(crate) draft_memory_operand_highlighting: bool,
    pub(crate) draft_floppy_image_path: Option<std::path::PathBuf>,
    pub(crate) draft_hdd_directory: Option<std::path::PathBuf>,
    pub(crate) draft_printer_settings: Option<PrinterSettings>,
    pub(crate) draft_printer_dialog_mode: PrinterDialogMode,
    pub(crate) draft_network_client_host: String,
    pub(crate) draft_network_client_port: String,
    pub(crate) draft_network_server_host: String,
    pub(crate) draft_network_server_port: String,
    pub(crate) draft_shortcuts: ShortcutSettings,
    pub(crate) original_shortcuts: ShortcutSettings,
    pub(crate) recording_shortcut: Option<ShortcutAction>,
    pub(crate) network_error: Option<String>,
    pub(crate) language_dropdown_open: bool,
    /// Keyboard highlight inside the open language dropdown. `None`
    /// when the dropdown is closed; while open, ArrowUp / ArrowDown
    /// move the highlight here without committing – the draft only
    /// changes once the user presses Enter or clicks an option.
    pub(crate) dropdown_highlight: Option<Lang>,
    pub(crate) original_lang: Lang,
    pub(crate) original_speed: SpeedTier,
    pub(crate) original_color_scheme: ColorScheme,
    pub(crate) original_follow_pc: bool,
    pub(crate) original_memory_operand_highlighting: bool,
    pub(crate) original_printer_dialog_mode: PrinterDialogMode,
    pub(crate) footer_focus: FooterFocus,
    pub(crate) reset_confirm_open: bool,
    pub(crate) reset_confirm_focus: ResetConfirmFocus,
    pub(crate) reset_confirm_keyboard_focus_visible: bool,
    pub(crate) section: SettingsSection,
    pub(crate) content_focus: Option<ContentFocus>,
    pub(crate) keyboard_focus_visible: bool,
}

impl SettingsDialog {
    #[cfg(test)]
    pub(crate) fn new(
        lang: Lang,
        speed: SpeedTier,
        follow_pc: bool,
        memory_operand_highlighting: bool,
        floppy_image_path: Option<std::path::PathBuf>,
        hdd_directory: Option<std::path::PathBuf>,
        network: NetworkSettings,
    ) -> Self {
        Self::new_with_shortcuts_and_printer(
            lang,
            speed,
            ColorScheme::DEFAULT,
            follow_pc,
            memory_operand_highlighting,
            floppy_image_path,
            hdd_directory,
            None,
            PrinterDialogMode::default(),
            network,
            ShortcutSettings::default(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    #[cfg(test)]
    pub(crate) fn new_with_shortcuts(
        lang: Lang,
        speed: SpeedTier,
        color_scheme: ColorScheme,
        follow_pc: bool,
        memory_operand_highlighting: bool,
        floppy_image_path: Option<std::path::PathBuf>,
        hdd_directory: Option<std::path::PathBuf>,
        network: NetworkSettings,
        shortcuts: ShortcutSettings,
    ) -> Self {
        Self::new_with_shortcuts_and_printer(
            lang,
            speed,
            color_scheme,
            follow_pc,
            memory_operand_highlighting,
            floppy_image_path,
            hdd_directory,
            None,
            PrinterDialogMode::default(),
            network,
            shortcuts,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new_with_shortcuts_and_printer(
        lang: Lang,
        speed: SpeedTier,
        color_scheme: ColorScheme,
        follow_pc: bool,
        memory_operand_highlighting: bool,
        floppy_image_path: Option<std::path::PathBuf>,
        hdd_directory: Option<std::path::PathBuf>,
        printer_settings: Option<PrinterSettings>,
        printer_dialog_mode: PrinterDialogMode,
        network: NetworkSettings,
        shortcuts: ShortcutSettings,
    ) -> Self {
        Self {
            category: SettingsCategory::General,
            sidebar_focus: SettingsCategory::General,
            search: String::new(),
            draft_lang: lang,
            draft_speed: speed,
            draft_color_scheme: color_scheme,
            draft_follow_pc: follow_pc,
            draft_memory_operand_highlighting: memory_operand_highlighting,
            draft_floppy_image_path: floppy_image_path,
            draft_hdd_directory: hdd_directory,
            draft_printer_settings: printer_settings,
            draft_printer_dialog_mode: printer_dialog_mode,
            draft_network_client_host: network.host,
            draft_network_client_port: network.port.to_string(),
            draft_network_server_host: network.bind_host,
            draft_network_server_port: network.bind_port.to_string(),
            draft_shortcuts: shortcuts.clone(),
            original_shortcuts: shortcuts,
            recording_shortcut: None,
            network_error: None,
            language_dropdown_open: false,
            dropdown_highlight: None,
            original_lang: lang,
            original_speed: speed,
            original_color_scheme: color_scheme,
            original_follow_pc: follow_pc,
            original_memory_operand_highlighting: memory_operand_highlighting,
            original_printer_dialog_mode: printer_dialog_mode,
            footer_focus: FooterFocus::Cancel,
            reset_confirm_open: false,
            reset_confirm_focus: ResetConfirmFocus::Cancel,
            reset_confirm_keyboard_focus_visible: false,
            section: SettingsSection::Content,
            content_focus: Some(ContentFocus::LanguageAnchor),
            keyboard_focus_visible: false,
        }
    }

    pub(crate) fn search_query(&self) -> &str {
        self.search.trim()
    }

    pub(crate) fn content_focus_is_visible(&self, focus: ContentFocus) -> bool {
        self.section_focus_is_visible(SettingsSection::Content) && self.content_focus == Some(focus)
    }

    pub(crate) fn section_focus_is_visible(&self, section: SettingsSection) -> bool {
        self.keyboard_focus_visible && self.section == section
    }

    pub(crate) fn first_content_focus(&self) -> ContentFocus {
        match self.category {
            SettingsCategory::General => ContentFocus::LanguageAnchor,
            SettingsCategory::ExternalDevices => ContentFocus::FloppyImage,
            SettingsCategory::Appearance => ContentFocus::Theme,
            SettingsCategory::Shortcuts => ContentFocus::Shortcut(ShortcutAction::ALL[0]),
        }
    }

    pub(crate) fn last_content_focus(&self) -> ContentFocus {
        match self.category {
            SettingsCategory::General => ContentFocus::FileAssociation,
            SettingsCategory::ExternalDevices => ContentFocus::NetworkDefaults,
            SettingsCategory::Appearance => ContentFocus::Theme,
            SettingsCategory::Shortcuts => {
                ContentFocus::Shortcut(ShortcutAction::ALL[ShortcutAction::ALL.len() - 1])
            }
        }
    }

    pub(crate) fn next_content_focus(&self, current: ContentFocus) -> Option<ContentFocus> {
        match self.category {
            SettingsCategory::General => match current {
                ContentFocus::LanguageAnchor => Some(ContentFocus::SpeedSlow),
                ContentFocus::SpeedSlow => Some(ContentFocus::SpeedMedium),
                ContentFocus::SpeedMedium => Some(ContentFocus::SpeedFast),
                ContentFocus::SpeedFast => Some(ContentFocus::SpeedMax),
                ContentFocus::SpeedMax => Some(ContentFocus::FollowPcOn),
                ContentFocus::FollowPcOn => Some(ContentFocus::FollowPcOff),
                ContentFocus::FollowPcOff => Some(ContentFocus::MemoryOperandHighlightingOn),
                ContentFocus::MemoryOperandHighlightingOn => {
                    Some(ContentFocus::MemoryOperandHighlightingOff)
                }
                ContentFocus::MemoryOperandHighlightingOff => Some(ContentFocus::FileAssociation),
                ContentFocus::FileAssociation => None,
                _ => Some(self.first_content_focus()),
            },
            SettingsCategory::ExternalDevices => match current {
                ContentFocus::FloppyImage => Some(ContentFocus::HddDirectory),
                ContentFocus::HddDirectory => Some(ContentFocus::PrinterDefault),
                ContentFocus::PrinterDefault => Some(ContentFocus::PrinterDialogModeCustom),
                ContentFocus::PrinterDialogModeCustom => {
                    Some(ContentFocus::PrinterDialogModeSystem)
                }
                ContentFocus::PrinterDialogModeSystem => Some(ContentFocus::NetworkDefaults),
                ContentFocus::NetworkDefaults => None,
                _ => Some(self.first_content_focus()),
            },
            SettingsCategory::Appearance => match current {
                ContentFocus::Theme => None,
                _ => Some(self.first_content_focus()),
            },
            SettingsCategory::Shortcuts => next_shortcut_focus(current),
        }
    }

    pub(crate) fn previous_content_focus(&self, current: ContentFocus) -> Option<ContentFocus> {
        match self.category {
            SettingsCategory::General => match current {
                ContentFocus::LanguageAnchor => None,
                ContentFocus::SpeedSlow => Some(ContentFocus::LanguageAnchor),
                ContentFocus::SpeedMedium => Some(ContentFocus::SpeedSlow),
                ContentFocus::SpeedFast => Some(ContentFocus::SpeedMedium),
                ContentFocus::SpeedMax => Some(ContentFocus::SpeedFast),
                ContentFocus::FollowPcOn => Some(ContentFocus::SpeedMax),
                ContentFocus::FollowPcOff => Some(ContentFocus::FollowPcOn),
                ContentFocus::MemoryOperandHighlightingOn => Some(ContentFocus::FollowPcOff),
                ContentFocus::MemoryOperandHighlightingOff => {
                    Some(ContentFocus::MemoryOperandHighlightingOn)
                }
                ContentFocus::FileAssociation => Some(ContentFocus::MemoryOperandHighlightingOff),
                _ => Some(self.last_content_focus()),
            },
            SettingsCategory::ExternalDevices => match current {
                ContentFocus::FloppyImage => None,
                ContentFocus::HddDirectory => Some(ContentFocus::FloppyImage),
                ContentFocus::PrinterDefault => Some(ContentFocus::HddDirectory),
                ContentFocus::PrinterDialogModeCustom => Some(ContentFocus::PrinterDefault),
                ContentFocus::PrinterDialogModeSystem => {
                    Some(ContentFocus::PrinterDialogModeCustom)
                }
                ContentFocus::NetworkDefaults => Some(ContentFocus::PrinterDialogModeSystem),
                _ => Some(self.last_content_focus()),
            },
            SettingsCategory::Appearance => match current {
                ContentFocus::Theme => None,
                _ => Some(self.last_content_focus()),
            },
            SettingsCategory::Shortcuts => previous_shortcut_focus(current),
        }
    }
}

fn next_shortcut_focus(current: ContentFocus) -> Option<ContentFocus> {
    let ContentFocus::Shortcut(action) = current else {
        return Some(ContentFocus::Shortcut(ShortcutAction::ALL[0]));
    };
    let current = ShortcutAction::ALL
        .iter()
        .position(|candidate| *candidate == action)?;
    ShortcutAction::ALL
        .get(current + 1)
        .copied()
        .map(ContentFocus::Shortcut)
}

fn previous_shortcut_focus(current: ContentFocus) -> Option<ContentFocus> {
    let ContentFocus::Shortcut(action) = current else {
        return Some(ContentFocus::Shortcut(
            ShortcutAction::ALL[ShortcutAction::ALL.len() - 1],
        ));
    };
    let current = ShortcutAction::ALL
        .iter()
        .position(|candidate| *candidate == action)?;
    current
        .checked_sub(1)
        .and_then(|idx| ShortcutAction::ALL.get(idx).copied())
        .map(ContentFocus::Shortcut)
}
