use std::time::Instant;

use super::constants::SETTINGS_SEARCH_INPUT_ID;
use super::messages::{Message, SpeedTier};
use super::settings_modal::SettingsDialog;
use super::settings_modal::{FooterFocus, ResetConfirmFocus, SettingsCategory, SettingsSection};
use super::state::DesktopApp;
use crate::i18n::Key;
use crate::settings_storage::{
    default_lang, language_from_lang, load_settings, preset_from_speed_tier, save_settings,
};
use iced::Task;

impl DesktopApp {
    pub(super) fn dispatch_settings_message(&mut self, message: Message) -> Option<Task<Message>> {
        if let Some(task) = self.dispatch_shortcut_settings_message(&message) {
            return Some(task);
        }
        match message {
            Message::OpenSettings => {
                self.settings_saved_notice = None;
                self.close_top_menu();
                self.hide_opcode_dropdown();
                self.close_open_device_panel();
                let settings = load_settings();
                self.settings_dialog = Some(SettingsDialog::new_with_shortcuts_and_printer(
                    self.lang,
                    self.default_speed,
                    self.color_scheme,
                    self.follow_pc,
                    self.memory_operand_highlighting,
                    settings.general.floppy_image_path,
                    settings.general.hdd_directory,
                    settings.general.printer_settings,
                    settings.general.printer_dialog_mode,
                    settings.network,
                    settings.shortcuts,
                ));
                Some(Task::none())
            }
            Message::CloseSettings => {
                self.settings_saved_notice = None;
                if let Some(dialog) = self.settings_dialog.take() {
                    let lang_changed = self.lang != dialog.original_lang;
                    self.lang = dialog.original_lang;
                    self.color_scheme = dialog.original_color_scheme;
                    let speed_changed = self.default_speed != dialog.original_speed
                        || self.speed_tier != dialog.original_speed;
                    self.default_speed = dialog.original_speed;
                    self.follow_pc = dialog.original_follow_pc;
                    self.memory_operand_highlighting = dialog.original_memory_operand_highlighting;
                    self.printer_dialog_mode = dialog.original_printer_dialog_mode;
                    self.shortcut_settings = dialog.original_shortcuts;
                    if speed_changed {
                        self.apply_speed_tier(dialog.original_speed);
                    }
                    if lang_changed {
                        self.refresh_localized_status();
                    }
                }
                Some(Task::none())
            }
            Message::SaveSettings => {
                let previous_notice = self.settings_saved_notice.take();
                let Some(dialog) = self.settings_dialog.as_ref() else {
                    return Some(Task::none());
                };
                let network = match parse_network_defaults(dialog) {
                    Ok(network) => network,
                    Err(_) => {
                        let error = self
                            .lang
                            .t(Key::Network(
                                crate::i18n::NetworkKey::GeneralSettingsInvalid,
                            ))
                            .to_owned();
                        if let Some(dialog) = self.settings_dialog.as_mut() {
                            dialog.network_error = Some(error);
                        }
                        return Some(Task::none());
                    }
                };
                self.save_settings_dialog(dialog, network);
                self.commit_settings_dialog_state();
                let started_at = Instant::now();
                self.settings_saved_notice = Some(match previous_notice {
                    Some(notice) => notice.restarted(started_at),
                    None => super::SettingsSavedNotice::new(started_at),
                });
                Some(Task::none())
            }
            Message::SettingsCategorySelected(category) => {
                if let Some(dialog) = self.settings_dialog.as_mut() {
                    dialog.category = category;
                    dialog.sidebar_focus = category;
                    dialog.content_focus = Some(dialog.first_content_focus());
                    dialog.keyboard_focus_visible = false;
                    if category != SettingsCategory::Shortcuts
                        && dialog.footer_focus == FooterFocus::ShortcutReset
                    {
                        dialog.footer_focus = FooterFocus::Cancel;
                    }
                    dialog.recording_shortcut = None;
                }
                Some(Task::none())
            }
            Message::SettingsSearchChanged(query) => {
                if let Some(dialog) = self.settings_dialog.as_mut() {
                    dialog.search = query;
                    dialog.language_dropdown_open = false;
                    dialog.dropdown_highlight = None;
                    dialog.recording_shortcut = None;
                }
                Some(Task::none())
            }
            Message::SettingsDraftLanguageChanged(lang) => {
                if let Some(dialog) = self.settings_dialog.as_mut() {
                    dialog.draft_lang = lang;
                    dialog.language_dropdown_open = false;
                    dialog.dropdown_highlight = None;
                }
                self.lang = lang;
                self.refresh_localized_status();
                Some(Task::none())
            }
            Message::SettingsDraftSpeedChanged(tier) => {
                if let Some(dialog) = self.settings_dialog.as_mut() {
                    dialog.draft_speed = tier;
                }
                self.default_speed = tier;
                self.apply_speed_tier(tier);
                Some(Task::none())
            }
            Message::SettingsDraftFollowPcSet(value) => {
                if let Some(dialog) = self.settings_dialog.as_mut() {
                    dialog.draft_follow_pc = value;
                }
                self.follow_pc = value;
                Some(Task::none())
            }
            Message::SettingsDraftMemoryOperandHighlightingSet(value) => {
                if let Some(dialog) = self.settings_dialog.as_mut() {
                    dialog.draft_memory_operand_highlighting = value;
                }
                self.memory_operand_highlighting = value;
                Some(Task::none())
            }
            Message::SettingsDraftColorSchemeChanged(scheme) => {
                if let Some(dialog) = self.settings_dialog.as_mut() {
                    dialog.draft_color_scheme = scheme;
                }
                self.color_scheme = scheme;
                Some(Task::none())
            }
            Message::SettingsDraftPrinterDialogModeSet(mode) => {
                if let Some(dialog) = self.settings_dialog.as_mut() {
                    dialog.draft_printer_dialog_mode = mode;
                }
                self.printer_dialog_mode = mode;
                Some(Task::none())
            }
            Message::SettingsFloppyImageBrowse => Some(self.browse_settings_floppy_image()),
            Message::SettingsDraftFloppyImageSet(path) => {
                if let Some(dialog) = self.settings_dialog.as_mut() {
                    dialog.draft_floppy_image_path = Some(path);
                }
                Some(Task::none())
            }
            Message::SettingsFloppyImageClear => {
                if let Some(dialog) = self.settings_dialog.as_mut() {
                    dialog.draft_floppy_image_path = None;
                }
                Some(Task::none())
            }
            Message::SettingsHddDirectoryBrowse => Some(self.browse_settings_hdd_directory()),
            Message::SettingsDraftHddDirectorySet(path) => {
                if let Some(dialog) = self.settings_dialog.as_mut() {
                    dialog.draft_hdd_directory = Some(path);
                }
                Some(Task::none())
            }
            Message::SettingsPrinterSetup => Some(self.configure_printer_settings()),
            Message::SettingsPrinterSetupFinished(result) => {
                self.finish_printer_settings_setup(result);
                Some(Task::none())
            }
            Message::SettingsPrinterClear => {
                if let Some(dialog) = self.settings_dialog.as_mut() {
                    dialog.draft_printer_settings = None;
                }
                Some(Task::none())
            }
            Message::SettingsNetworkClientHostChanged(host) => {
                if let Some(dialog) = self.settings_dialog.as_mut() {
                    dialog.draft_network_client_host = host;
                    dialog.network_error = None;
                }
                Some(Task::none())
            }
            Message::SettingsNetworkClientPortChanged(port) => {
                if let Some(dialog) = self.settings_dialog.as_mut() {
                    dialog.draft_network_client_port = port;
                    dialog.network_error = None;
                }
                Some(Task::none())
            }
            Message::SettingsNetworkServerHostChanged(host) => {
                if let Some(dialog) = self.settings_dialog.as_mut() {
                    dialog.draft_network_server_host = host;
                    dialog.network_error = None;
                }
                Some(Task::none())
            }
            Message::SettingsNetworkServerPortChanged(port) => {
                if let Some(dialog) = self.settings_dialog.as_mut() {
                    dialog.draft_network_server_port = port;
                    dialog.network_error = None;
                }
                Some(Task::none())
            }
            Message::SettingsLanguageDropdownToggled => {
                if let Some(dialog) = self.settings_dialog.as_mut() {
                    dialog.language_dropdown_open = !dialog.language_dropdown_open;
                    dialog.recording_shortcut = None;
                    dialog.dropdown_highlight = if dialog.language_dropdown_open {
                        Some(dialog.draft_lang)
                    } else {
                        None
                    };
                }
                Some(Task::none())
            }
            Message::SettingsSectionCycle { backward } => {
                let Some(dialog) = self.settings_dialog.as_mut() else {
                    return Some(Task::none());
                };
                if dialog.reset_confirm_open {
                    return Some(Task::none());
                }
                cycle_section(dialog, backward);
                let target = dialog.section;
                Some(match target {
                    SettingsSection::Search => {
                        iced::widget::operation::focus(SETTINGS_SEARCH_INPUT_ID)
                    }
                    _ => iced::widget::operation::focus("settings-blur"),
                })
            }
            Message::SettingsResetRequested => {
                if let Some(dialog) = self.settings_dialog.as_mut() {
                    dialog.reset_confirm_open = true;
                    dialog.reset_confirm_focus = ResetConfirmFocus::Cancel;
                    dialog.reset_confirm_keyboard_focus_visible = false;
                    dialog.language_dropdown_open = false;
                    dialog.dropdown_highlight = None;
                    dialog.recording_shortcut = None;
                }
                Some(Task::none())
            }
            Message::SettingsResetCancelled => {
                if let Some(dialog) = self.settings_dialog.as_mut() {
                    dialog.reset_confirm_open = false;
                    dialog.reset_confirm_keyboard_focus_visible = false;
                }
                Some(Task::none())
            }
            Message::SettingsResetConfirmed => {
                let default_lang = default_lang();
                let default_speed = SpeedTier::High;
                let default_color_scheme = crate::persistence::ColorScheme::DEFAULT;
                let default_follow_pc = false;
                let default_memory_operand_highlighting = true;
                let default_printer_dialog_mode = crate::persistence::PrinterDialogMode::default();
                let network = crate::persistence::NetworkSettings::default();
                let shortcuts = crate::persistence::ShortcutSettings::default();
                if let Some(dialog) = self.settings_dialog.as_mut() {
                    dialog.draft_lang = default_lang;
                    dialog.draft_speed = default_speed;
                    dialog.draft_color_scheme = default_color_scheme;
                    dialog.draft_floppy_image_path = None;
                    dialog.draft_hdd_directory = None;
                    dialog.draft_printer_settings = None;
                    dialog.draft_printer_dialog_mode = default_printer_dialog_mode;
                    dialog.original_lang = default_lang;
                    dialog.original_speed = default_speed;
                    dialog.original_color_scheme = default_color_scheme;
                    dialog.draft_network_client_host = network.host;
                    dialog.draft_network_client_port = network.port.to_string();
                    dialog.draft_network_server_host = network.bind_host;
                    dialog.draft_network_server_port = network.bind_port.to_string();
                    dialog.draft_shortcuts = shortcuts.clone();
                    dialog.original_shortcuts = shortcuts.clone();
                    dialog.recording_shortcut = None;
                    dialog.draft_follow_pc = default_follow_pc;
                    dialog.draft_memory_operand_highlighting = default_memory_operand_highlighting;
                    dialog.original_follow_pc = default_follow_pc;
                    dialog.original_memory_operand_highlighting =
                        default_memory_operand_highlighting;
                    dialog.original_printer_dialog_mode = default_printer_dialog_mode;
                    dialog.network_error = None;
                    dialog.reset_confirm_open = false;
                    dialog.reset_confirm_keyboard_focus_visible = false;
                }
                self.follow_pc = default_follow_pc;
                self.memory_operand_highlighting = default_memory_operand_highlighting;
                self.shortcut_settings = shortcuts;
                self.printer_default_settings = None;
                self.printer_dialog_mode = default_printer_dialog_mode;
                let lang_changed = self.lang != default_lang;
                self.lang = default_lang;
                self.default_speed = default_speed;
                self.color_scheme = default_color_scheme;
                self.apply_speed_tier(default_speed);
                if lang_changed {
                    self.refresh_localized_status();
                }
                if let Some(dialog) = self.settings_dialog.as_ref()
                    && let Ok(network) = parse_network_defaults(dialog)
                {
                    self.save_settings_dialog(dialog, network);
                }
                Some(Task::none())
            }
            Message::SettingsFileAssociationRegister => {
                if let Err(error) = k580_ui::file_assoc::register() {
                    self.error_notice =
                        Some(format!("{}: {}", self.lang.t(Key::ErrorPrefix), error));
                    self.error_notice_dismiss_at =
                        Some(std::time::Instant::now() + std::time::Duration::from_secs(8));
                }
                self.file_association_toggle_revision =
                    self.file_association_toggle_revision.wrapping_add(1);
                Some(Task::none())
            }
            Message::SettingsFileAssociationUnregister => {
                if let Err(error) = k580_ui::file_assoc::unregister() {
                    self.error_notice =
                        Some(format!("{}: {}", self.lang.t(Key::ErrorPrefix), error));
                    self.error_notice_dismiss_at =
                        Some(std::time::Instant::now() + std::time::Duration::from_secs(8));
                }
                self.file_association_toggle_revision =
                    self.file_association_toggle_revision.wrapping_add(1);
                Some(Task::none())
            }
            _ => None,
        }
    }

    pub(super) fn commit_settings_dialog_state(&mut self) {
        let Some(dialog) = self.settings_dialog.as_mut() else {
            return;
        };
        self.shortcut_settings = dialog.draft_shortcuts.clone();
        self.printer_default_settings = dialog.draft_printer_settings.clone();
        self.printer_dialog_mode = dialog.draft_printer_dialog_mode;
        dialog.original_lang = dialog.draft_lang;
        dialog.original_speed = dialog.draft_speed;
        dialog.original_color_scheme = dialog.draft_color_scheme;
        dialog.original_follow_pc = dialog.draft_follow_pc;
        dialog.original_memory_operand_highlighting = dialog.draft_memory_operand_highlighting;
        dialog.original_printer_dialog_mode = dialog.draft_printer_dialog_mode;
        dialog.original_shortcuts = dialog.draft_shortcuts.clone();
    }

    fn save_settings_dialog(&self, dialog: &SettingsDialog, network: NetworkDefaults) {
        let mut settings = load_settings();
        settings.general.language = language_from_lang(self.lang);
        settings.general.default_speed = preset_from_speed_tier(self.default_speed);
        settings.general.follow_pc = dialog.draft_follow_pc;
        settings.general.memory_operand_highlighting = dialog.draft_memory_operand_highlighting;
        settings.general.floppy_image_path = dialog.draft_floppy_image_path.clone();
        settings.general.hdd_directory = dialog.draft_hdd_directory.clone();
        settings
            .general
            .set_printer_settings(dialog.draft_printer_settings.clone());
        settings.general.printer_dialog_mode = dialog.draft_printer_dialog_mode;
        settings.ui.theme = dialog.draft_color_scheme;
        apply_network_defaults(&mut settings.network, network);
        settings.shortcuts = dialog.draft_shortcuts.clone();
        save_settings(&settings);
    }
}

mod network;
mod section;
mod shortcuts;
mod storage;
use network::{NetworkDefaults, apply_network_defaults, parse_network_defaults};
use section::cycle_section;
