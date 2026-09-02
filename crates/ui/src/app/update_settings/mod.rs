use std::time::Instant;

use super::constants::SETTINGS_SEARCH_INPUT_ID;
use super::messages::Message;
use super::settings_modal::SettingsDialog;
use super::settings_modal::{FooterFocus, ResetConfirmFocus, SettingsCategory, SettingsSection};
use super::state::DesktopApp;
use crate::i18n::Key;
use crate::settings_storage::{
    language_from_lang, load_settings, preset_from_speed_tier, save_settings,
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
                let mut dialog = SettingsDialog::new_with_shortcuts_and_printer(
                    self.lang,
                    self.default_speed,
                    self.color_scheme,
                    self.follow_pc,
                    self.memory_operand_highlighting,
                    self.show_file_name,
                    settings.general.floppy_image_path,
                    settings.general.hdd_directory,
                    settings.general.printer_settings,
                    settings.general.printer_dialog_mode,
                    settings.network,
                    settings.shortcuts,
                );
                dialog.original_active_speed = self.speed_tier;
                dialog.draft_monitor_split = settings.general.monitor_split;
                dialog.original_monitor_split = self.monitor_split;
                self.settings_dialog = Some(dialog);
                Some(Task::none())
            }
            Message::CloseSettings => {
                self.settings_saved_notice = None;
                if let Some(dialog) = self.settings_dialog.take() {
                    self.apply_language(dialog.original_lang);
                    self.color_scheme = dialog.original_color_scheme;
                    let speed_changed = self.default_speed != dialog.original_speed
                        || self.speed_tier != dialog.original_active_speed;
                    self.default_speed = dialog.original_speed;
                    self.follow_pc = dialog.original_follow_pc;
                    self.memory_operand_highlighting = dialog.original_memory_operand_highlighting;
                    self.show_file_name = dialog.original_show_file_name;
                    self.monitor_split = dialog.original_monitor_split;
                    self.printer_dialog_mode = dialog.original_printer_dialog_mode;
                    self.shortcut_settings = dialog.original_shortcuts;
                    if speed_changed {
                        self.apply_speed_tier(dialog.original_active_speed);
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
                self.apply_language(lang);
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
            Message::SettingsDraftShowFileNameSet(value) => {
                if let Some(dialog) = self.settings_dialog.as_mut() {
                    dialog.draft_show_file_name = value;
                }
                self.show_file_name = value;
                Some(Task::none())
            }
            Message::SettingsDraftMonitorSplitSet(value) => {
                if let Some(dialog) = self.settings_dialog.as_mut() {
                    dialog.draft_monitor_split = value;
                }
                self.monitor_split = value;
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
                if !network::is_directory_writable(&path) {
                    self.show_error_notice(self.lang.t(Key::ErrHddDirectoryNotWritable));
                    return Some(Task::none());
                }
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
                self.reset_settings();
                Some(Task::none())
            }
            Message::SettingsFileAssociationRegister => {
                Some(self.update_file_association(k580_ui::file_assoc::register))
            }
            Message::SettingsFileAssociationUnregister => {
                Some(self.update_file_association(k580_ui::file_assoc::unregister))
            }
            _ => None,
        }
    }

    fn update_file_association(&mut self, operation: fn() -> Result<(), String>) -> Task<Message> {
        if let Err(error) = operation() {
            self.show_error_notice(format!("{}: {}", self.lang.t(Key::ErrorPrefix), error));
        }
        self.file_association_toggle_revision =
            self.file_association_toggle_revision.wrapping_add(1);
        Task::none()
    }

    pub(super) fn commit_settings_dialog_state(&mut self) {
        let active_speed = self.speed_tier;
        let Some(dialog) = self.settings_dialog.as_mut() else {
            return;
        };
        self.shortcut_settings = dialog.draft_shortcuts.clone();
        self.printer_default_settings = dialog.draft_printer_settings.clone();
        self.printer_dialog_mode = dialog.draft_printer_dialog_mode;
        dialog.original_lang = dialog.draft_lang;
        dialog.original_speed = dialog.draft_speed;
        dialog.original_active_speed = active_speed;
        dialog.original_color_scheme = dialog.draft_color_scheme;
        dialog.original_follow_pc = dialog.draft_follow_pc;
        dialog.original_memory_operand_highlighting = dialog.draft_memory_operand_highlighting;
        dialog.original_show_file_name = dialog.draft_show_file_name;
        dialog.original_monitor_split = self.monitor_split;
        dialog.original_printer_dialog_mode = dialog.draft_printer_dialog_mode;
        dialog.original_shortcuts = dialog.draft_shortcuts.clone();
    }

    fn save_settings_dialog(&self, dialog: &SettingsDialog, network: NetworkDefaults) {
        let mut settings = load_settings();
        settings.general.language = language_from_lang(self.lang);
        settings.general.default_speed = preset_from_speed_tier(self.default_speed);
        settings.general.follow_pc = dialog.draft_follow_pc;
        settings.general.memory_operand_highlighting = dialog.draft_memory_operand_highlighting;
        settings.general.show_file_name = dialog.draft_show_file_name;
        settings.general.monitor_split = dialog.draft_monitor_split;
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
mod reset;
mod section;
mod shortcuts;
mod storage;
use network::{NetworkDefaults, apply_network_defaults, parse_network_defaults};
use section::cycle_section;
