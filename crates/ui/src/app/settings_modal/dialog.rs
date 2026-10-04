use super::focus::{
    ContentFocus, FooterFocus, ResetConfirmFocus, SettingsCategory, SettingsSection,
};
use crate::app::messages::SpeedTier;
use crate::app::state::DropdownState;
use crate::i18n::Lang;
use crate::persistence::{
    ColorScheme, NetworkSettings, PrinterDialogMode, ShortcutAction, ShortcutSettings,
};
use k580_ui::devices::printer::PrinterSettings;

/// Original values restore immediate previews on Cancel; Save commits only after persistence succeeds.
#[derive(Clone, Debug)]
pub(crate) struct SettingsDialog {
    pub(crate) category: SettingsCategory,
    pub(crate) sidebar_focus: SettingsCategory,
    pub(crate) search: String,
    pub(crate) content_can_scroll_up: bool,
    pub(crate) content_can_scroll_down: bool,
    pub(crate) draft_lang: Lang,
    pub(crate) draft_speed: SpeedTier,
    pub(crate) draft_color_scheme: ColorScheme,
    pub(crate) draft_follow_pc: bool,
    pub(crate) draft_memory_operand_highlighting: bool,
    pub(crate) draft_show_file_name: bool,
    pub(crate) draft_monitor_split: bool,
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
    #[cfg(target_os = "windows")]
    pub(crate) file_association_registered: bool,
    pub(crate) language_dropdown: DropdownState<Lang>,
    pub(crate) original_lang: Lang,
    pub(crate) original_speed: SpeedTier,
    pub(crate) original_active_speed: SpeedTier,
    pub(crate) original_color_scheme: ColorScheme,
    pub(crate) original_follow_pc: bool,
    pub(crate) original_memory_operand_highlighting: bool,
    pub(crate) original_show_file_name: bool,
    pub(crate) original_monitor_split: bool,
    pub(crate) original_printer_dialog_mode: PrinterDialogMode,
    pub(crate) footer_focus: FooterFocus,
    pub(crate) reset_confirm_open: bool,
    pub(crate) reset_confirm_focus: ResetConfirmFocus,
    pub(crate) reset_confirm_keyboard_focus_visible: bool,
    pub(crate) section: SettingsSection,
    pub(crate) content_focus: Option<ContentFocus>,
    pub(crate) keyboard_focus_visible: bool,
}

pub(crate) struct SettingsInitialState {
    pub(crate) lang: Lang,
    pub(crate) speed: SpeedTier,
    pub(crate) active_speed: Option<SpeedTier>,
    pub(crate) color_scheme: ColorScheme,
    pub(crate) follow_pc: bool,
    pub(crate) memory_operand_highlighting: bool,
    pub(crate) show_file_name: bool,
    pub(crate) monitor_split: bool,
    pub(crate) original_monitor_split: Option<bool>,
    pub(crate) floppy_image_path: Option<std::path::PathBuf>,
    pub(crate) hdd_directory: Option<std::path::PathBuf>,
    pub(crate) printer_settings: Option<PrinterSettings>,
    pub(crate) printer_dialog_mode: PrinterDialogMode,
    pub(crate) network: NetworkSettings,
    pub(crate) shortcuts: ShortcutSettings,
}

#[cfg(test)]
impl Default for SettingsInitialState {
    fn default() -> Self {
        Self {
            lang: Lang::Ru,
            speed: SpeedTier::Medium,
            active_speed: None,
            color_scheme: ColorScheme::DEFAULT,
            follow_pc: true,
            memory_operand_highlighting: true,
            show_file_name: false,
            monitor_split: false,
            original_monitor_split: None,
            floppy_image_path: None,
            hdd_directory: None,
            printer_settings: None,
            printer_dialog_mode: PrinterDialogMode::default(),
            network: NetworkSettings::default(),
            shortcuts: ShortcutSettings::default(),
        }
    }
}

impl SettingsDialog {
    pub(crate) fn new(initial: SettingsInitialState) -> Self {
        let SettingsInitialState {
            lang,
            speed,
            color_scheme,
            follow_pc,
            memory_operand_highlighting,
            show_file_name,
            floppy_image_path,
            hdd_directory,
            printer_settings,
            printer_dialog_mode,
            network,
            shortcuts,
            monitor_split,
            original_monitor_split,
            active_speed,
        } = initial;
        Self {
            category: SettingsCategory::General,
            sidebar_focus: SettingsCategory::General,
            search: String::new(),
            content_can_scroll_up: false,
            content_can_scroll_down: true,
            draft_lang: lang,
            draft_speed: speed,
            draft_color_scheme: color_scheme,
            draft_follow_pc: follow_pc,
            draft_memory_operand_highlighting: memory_operand_highlighting,
            draft_show_file_name: show_file_name,
            draft_monitor_split: monitor_split,
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
            #[cfg(target_os = "windows")]
            file_association_registered: k580_ui::file_assoc::is_registered(),
            language_dropdown: DropdownState::Closed,
            original_lang: lang,
            original_speed: speed,
            original_active_speed: active_speed.unwrap_or(speed),
            original_color_scheme: color_scheme,
            original_follow_pc: follow_pc,
            original_memory_operand_highlighting: memory_operand_highlighting,
            original_show_file_name: show_file_name,
            original_monitor_split: original_monitor_split.unwrap_or(monitor_split),
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
            SettingsCategory::General if k580_ui::file_assoc::is_user_configurable() => {
                ContentFocus::FileAssociation
            }
            SettingsCategory::General => ContentFocus::ShowFileNameOff,
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
                ContentFocus::MemoryOperandHighlightingOff => Some(ContentFocus::ShowFileNameOn),
                ContentFocus::ShowFileNameOn => Some(ContentFocus::ShowFileNameOff),
                ContentFocus::ShowFileNameOff if k580_ui::file_assoc::is_user_configurable() => {
                    Some(ContentFocus::FileAssociation)
                }
                ContentFocus::ShowFileNameOff => None,
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
                ContentFocus::PrinterDialogModeSystem => Some(ContentFocus::MonitorLayoutUnified),
                ContentFocus::MonitorLayoutUnified => Some(ContentFocus::MonitorLayoutSplit),
                ContentFocus::MonitorLayoutSplit => Some(ContentFocus::NetworkDefaults),
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
                ContentFocus::ShowFileNameOn => Some(ContentFocus::MemoryOperandHighlightingOff),
                ContentFocus::ShowFileNameOff => Some(ContentFocus::ShowFileNameOn),
                ContentFocus::FileAssociation => Some(ContentFocus::ShowFileNameOff),
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
                ContentFocus::MonitorLayoutUnified => Some(ContentFocus::PrinterDialogModeSystem),
                ContentFocus::MonitorLayoutSplit => Some(ContentFocus::MonitorLayoutUnified),
                ContentFocus::NetworkDefaults => Some(ContentFocus::MonitorLayoutSplit),
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
