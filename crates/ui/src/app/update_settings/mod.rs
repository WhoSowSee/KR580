mod network;
mod persistence;
mod reset;
mod section;
mod shortcuts;
mod storage;

use network::parse_network_defaults;
use section::cycle_section;

use super::constants::SETTINGS_SEARCH_INPUT_ID;
use super::messages::Message;
use super::settings_modal::SettingsDialog;
use super::settings_modal::{FooterFocus, ResetConfirmFocus, SettingsCategory, SettingsSection};
use super::state::DesktopApp;
use crate::i18n::Key;
use iced::Task;

impl DesktopApp {
    pub(super) fn dispatch_settings_message(&mut self, message: Message) -> Option<Task<Message>> {
        if let Some(task) = self.dispatch_shortcut_settings_message(&message) {
            return Some(task);
        }
        match message {
            Message::OpenSettings => {
                if self.settings_save_pending() {
                    return Some(Task::none());
                }
                self.preferences.dialog_generation =
                    self.preferences.dialog_generation.wrapping_add(1);
                self.preferences.settings_notice = None;
                self.close_top_menu();
                self.hide_opcode_dropdown();
                self.close_open_device_panel();
                let settings = self.preferences.stored.clone();
                let dialog =
                    SettingsDialog::new(crate::app::settings_modal::SettingsInitialState {
                        lang: self.preferences.lang,
                        speed: self.preferences.default_speed,
                        color_scheme: self.preferences.color_scheme,
                        follow_pc: self.preferences.follow_pc,
                        memory_operand_highlighting: self.preferences.memory_operand_highlighting,
                        show_file_name: self.preferences.show_file_name,
                        floppy_image_path: settings.general.floppy_image_path,
                        hdd_directory: settings.general.hdd_directory,
                        printer_settings: settings.general.printer_settings,
                        printer_dialog_mode: settings.general.printer_dialog_mode,
                        network: settings.network,
                        shortcuts: settings.shortcuts,
                        active_speed: Some(self.execution.speed_tier),
                        monitor_split: settings.general.monitor_split,
                        original_monitor_split: Some(self.panels.monitor_split),
                    });
                self.preferences.settings_dialog = Some(dialog);
                Some(Task::none())
            }
            Message::CloseSettings => {
                if self.settings_save_pending() {
                    return Some(Task::none());
                }
                self.preferences.dialog_generation =
                    self.preferences.dialog_generation.wrapping_add(1);
                self.preferences.settings_notice = None;
                if let Some(dialog) = self.preferences.settings_dialog.take() {
                    self.apply_language(dialog.original_lang);
                    self.preferences.color_scheme = dialog.original_color_scheme;
                    let speed_changed = self.preferences.default_speed != dialog.original_speed
                        || self.execution.speed_tier != dialog.original_active_speed;
                    self.preferences.default_speed = dialog.original_speed;
                    self.preferences.follow_pc = dialog.original_follow_pc;
                    self.preferences.memory_operand_highlighting =
                        dialog.original_memory_operand_highlighting;
                    self.preferences.show_file_name = dialog.original_show_file_name;
                    self.panels.monitor_split = dialog.original_monitor_split;
                    self.printer_setup.printer_dialog_mode = dialog.original_printer_dialog_mode;
                    self.preferences.shortcut_settings = dialog.original_shortcuts;
                    if speed_changed {
                        self.apply_speed_tier(dialog.original_active_speed);
                    }
                }
                Some(Task::none())
            }
            Message::SaveSettings => {
                if self.settings_save_pending() {
                    return Some(Task::none());
                }
                let Some(dialog) = self.preferences.settings_dialog.as_ref() else {
                    return Some(Task::none());
                };
                let network = match parse_network_defaults(dialog) {
                    Ok(network) => network,
                    Err(_) => {
                        self.preferences.settings_notice = None;
                        let error = self
                            .preferences
                            .lang
                            .t(Key::Network(
                                crate::i18n::NetworkKey::GeneralSettingsInvalid,
                            ))
                            .to_owned();
                        if let Some(dialog) = self.preferences.settings_dialog.as_mut() {
                            dialog.network_error = Some(error);
                        }
                        return Some(Task::none());
                    }
                };
                self.save_settings_dialog(network, Key::SettingsSavedNotice);
                Some(Task::none())
            }
            Message::SettingsCategorySelected(category) => {
                if let Some(dialog) = self.preferences.settings_dialog.as_mut() {
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
                if let Some(dialog) = self.preferences.settings_dialog.as_mut() {
                    dialog.search = query;
                    dialog.language_dropdown.set_open(false);
                    dialog.recording_shortcut = None;
                }
                Some(Task::none())
            }
            Message::SettingsDraftLanguageChanged(lang) => {
                if let Some(dialog) = self.preferences.settings_dialog.as_mut() {
                    dialog.draft_lang = lang;
                    dialog.language_dropdown.set_open(false);
                }
                self.apply_language(lang);
                Some(Task::none())
            }
            Message::SettingsDraftSpeedChanged(tier) => {
                if let Some(dialog) = self.preferences.settings_dialog.as_mut() {
                    dialog.draft_speed = tier;
                }
                self.preferences.default_speed = tier;
                self.apply_speed_tier(tier);
                Some(Task::none())
            }
            Message::SettingsDraftFollowPcSet(value) => {
                if let Some(dialog) = self.preferences.settings_dialog.as_mut() {
                    dialog.draft_follow_pc = value;
                }
                self.preferences.follow_pc = value;
                Some(Task::none())
            }
            Message::SettingsDraftMemoryOperandHighlightingSet(value) => {
                if let Some(dialog) = self.preferences.settings_dialog.as_mut() {
                    dialog.draft_memory_operand_highlighting = value;
                }
                self.preferences.memory_operand_highlighting = value;
                Some(Task::none())
            }
            Message::SettingsDraftShowFileNameSet(value) => {
                if let Some(dialog) = self.preferences.settings_dialog.as_mut() {
                    dialog.draft_show_file_name = value;
                }
                self.preferences.show_file_name = value;
                Some(Task::none())
            }
            Message::SettingsDraftMonitorSplitSet(value) => {
                if let Some(dialog) = self.preferences.settings_dialog.as_mut() {
                    dialog.draft_monitor_split = value;
                }
                self.panels.monitor_split = value;
                Some(Task::none())
            }
            Message::SettingsDraftColorSchemeChanged(scheme) => {
                if let Some(dialog) = self.preferences.settings_dialog.as_mut() {
                    dialog.draft_color_scheme = scheme;
                }
                self.preferences.color_scheme = scheme;
                Some(Task::none())
            }
            Message::SettingsDraftPrinterDialogModeSet(mode) => {
                if let Some(dialog) = self.preferences.settings_dialog.as_mut() {
                    dialog.draft_printer_dialog_mode = mode;
                }
                self.printer_setup.printer_dialog_mode = mode;
                Some(Task::none())
            }
            Message::SettingsFloppyImageBrowse => Some(self.browse_settings_floppy_image()),
            Message::SettingsDraftFloppyImageSet(path) => {
                if let Some(dialog) = self.preferences.settings_dialog.as_mut() {
                    dialog.draft_floppy_image_path = Some(path);
                }
                Some(Task::none())
            }
            Message::SettingsFloppyImageClear => {
                if let Some(dialog) = self.preferences.settings_dialog.as_mut() {
                    dialog.draft_floppy_image_path = None;
                }
                Some(Task::none())
            }
            Message::SettingsHddDirectoryBrowse => Some(self.browse_settings_hdd_directory()),
            Message::SettingsDraftHddDirectorySet(path) => {
                self.validate_hdd_directory(path);
                Some(Task::none())
            }
            Message::SettingsPrinterSetup => Some(self.configure_printer_settings()),
            Message::SettingsPrinterSetupFinished(result) => {
                self.finish_printer_settings_setup(result);
                Some(Task::none())
            }
            Message::SettingsPrinterClear => {
                if let Some(dialog) = self.preferences.settings_dialog.as_mut() {
                    dialog.draft_printer_settings = None;
                }
                Some(Task::none())
            }
            Message::SettingsNetworkClientHostChanged(host) => {
                if let Some(dialog) = self.preferences.settings_dialog.as_mut() {
                    dialog.draft_network_client_host = host;
                    dialog.network_error = None;
                }
                Some(Task::none())
            }
            Message::SettingsNetworkClientPortChanged(port) => {
                if let Some(dialog) = self.preferences.settings_dialog.as_mut() {
                    dialog.draft_network_client_port = port;
                    dialog.network_error = None;
                }
                Some(Task::none())
            }
            Message::SettingsNetworkServerHostChanged(host) => {
                if let Some(dialog) = self.preferences.settings_dialog.as_mut() {
                    dialog.draft_network_server_host = host;
                    dialog.network_error = None;
                }
                Some(Task::none())
            }
            Message::SettingsNetworkServerPortChanged(port) => {
                if let Some(dialog) = self.preferences.settings_dialog.as_mut() {
                    dialog.draft_network_server_port = port;
                    dialog.network_error = None;
                }
                Some(Task::none())
            }
            Message::SettingsLanguageDropdownToggled => {
                if let Some(dialog) = self.preferences.settings_dialog.as_mut() {
                    dialog
                        .language_dropdown
                        .set_open(!dialog.language_dropdown.is_open());
                    dialog.recording_shortcut = None;
                    dialog
                        .language_dropdown
                        .set_highlight(if dialog.language_dropdown.is_open() {
                            Some(dialog.draft_lang)
                        } else {
                            None
                        });
                }
                Some(Task::none())
            }
            Message::SettingsSectionCycle { backward } => {
                let Some(dialog) = self.preferences.settings_dialog.as_mut() else {
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
                if self.settings_save_pending() {
                    return Some(Task::none());
                }
                if let Some(dialog) = self.preferences.settings_dialog.as_mut() {
                    dialog.reset_confirm_open = true;
                    dialog.reset_confirm_focus = ResetConfirmFocus::Cancel;
                    dialog.reset_confirm_keyboard_focus_visible = false;
                    dialog.language_dropdown.set_open(false);
                    dialog.recording_shortcut = None;
                }
                Some(Task::none())
            }
            Message::SettingsResetCancelled => {
                if let Some(dialog) = self.preferences.settings_dialog.as_mut() {
                    dialog.reset_confirm_open = false;
                    dialog.reset_confirm_keyboard_focus_visible = false;
                }
                Some(Task::none())
            }
            Message::SettingsResetConfirmed => {
                if !self.settings_save_pending() {
                    self.reset_settings();
                }
                Some(Task::none())
            }
            Message::SettingsFileAssociationRegister => {
                Some(self.update_file_association(k580_ui::file_assoc::register))
            }
            #[cfg(target_os = "windows")]
            Message::SettingsFileAssociationUnregister => {
                Some(self.update_file_association(k580_ui::file_assoc::unregister))
            }
            Message::SettingsFileAssociationFinished(result) => {
                self.preferences.file_association_pending = false;
                #[cfg(target_os = "windows")]
                if let Some(dialog) = self.preferences.settings_dialog.as_mut() {
                    dialog.file_association_registered = k580_ui::file_assoc::is_registered();
                }
                if let Err(error) = result {
                    self.show_error_notice(format!(
                        "{}: {}",
                        self.preferences.lang.t(Key::ErrorPrefix),
                        error
                    ));
                }
                Some(Task::none())
            }
            _ => None,
        }
    }

    fn update_file_association(&mut self, operation: fn() -> Result<(), String>) -> Task<Message> {
        if self.preferences.file_association_pending
            || self.preferences.settings_dialog.is_none()
            || !k580_ui::file_assoc::is_user_configurable()
        {
            return Task::none();
        }
        self.preferences.file_association_pending = true;
        Task::perform(
            async move {
                tokio::task::spawn_blocking(operation)
                    .await
                    .map_err(|error| format!("file association task failed: {error}"))?
            },
            Message::SettingsFileAssociationFinished,
        )
    }

    pub(super) fn commit_settings_dialog_state(
        &mut self,
        saved: &SettingsDialog,
        active_speed: crate::app::SpeedTier,
    ) {
        let Some(dialog) = self.preferences.settings_dialog.as_mut() else {
            return;
        };
        self.printer_setup.printer_default_settings = saved.draft_printer_settings.clone();
        dialog.original_lang = saved.draft_lang;
        dialog.original_speed = saved.draft_speed;
        dialog.original_active_speed = active_speed;
        dialog.original_color_scheme = saved.draft_color_scheme;
        dialog.original_follow_pc = saved.draft_follow_pc;
        dialog.original_memory_operand_highlighting = saved.draft_memory_operand_highlighting;
        dialog.original_show_file_name = saved.draft_show_file_name;
        dialog.original_monitor_split = saved.draft_monitor_split;
        dialog.original_printer_dialog_mode = saved.draft_printer_dialog_mode;
        dialog.original_shortcuts = saved.draft_shortcuts.clone();
    }
}
