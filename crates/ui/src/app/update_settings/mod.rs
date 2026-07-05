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
                self.open_menu = None;
                self.hide_opcode_dropdown();
                self.close_open_device_panel();
                let settings = load_settings();
                self.settings_dialog = Some(SettingsDialog::new_with_shortcuts(
                    self.lang,
                    self.default_speed,
                    self.color_scheme,
                    self.follow_pc,
                    self.memory_operand_highlighting,
                    settings.general.floppy_image_path,
                    settings.general.hdd_directory,
                    settings.network,
                    settings.shortcuts,
                ));
                Some(Task::none())
            }
            Message::CloseSettings => {
                if let Some(dialog) = self.settings_dialog.take() {
                    let lang_changed = self.lang != dialog.original_lang;
                    self.lang = dialog.original_lang;
                    self.color_scheme = dialog.original_color_scheme;
                    let speed_changed = self.default_speed != dialog.original_speed
                        || self.speed_tier != dialog.original_speed;
                    self.default_speed = dialog.original_speed;
                    self.follow_pc = dialog.original_follow_pc;
                    self.memory_operand_highlighting = dialog.original_memory_operand_highlighting;
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
                let shortcuts = dialog.draft_shortcuts.clone();
                let mut settings = load_settings();
                settings.general.follow_pc = dialog.draft_follow_pc;
                settings.general.memory_operand_highlighting =
                    dialog.draft_memory_operand_highlighting;
                settings.general.floppy_image_path = dialog.draft_floppy_image_path.clone();
                settings.general.hdd_directory = dialog.draft_hdd_directory.clone();
                settings.ui.theme = dialog.draft_color_scheme;
                apply_network_defaults(&mut settings.network, network);
                settings.shortcuts = shortcuts.clone();
                save_settings(&settings);
                self.shortcut_settings = shortcuts;
                if self.settings_dialog.take().is_some() {
                    Some(Task::done(Message::PersistSettings))
                } else {
                    Some(Task::none())
                }
            }
            Message::PersistSettings => {
                let mut settings = load_settings();
                settings.general.language = language_from_lang(self.lang);
                settings.general.default_speed = preset_from_speed_tier(self.default_speed);
                settings.ui.theme = self.color_scheme;
                if let Some(dialog) = self.settings_dialog.as_ref() {
                    settings.general.follow_pc = dialog.draft_follow_pc;
                    settings.general.floppy_image_path = dialog.draft_floppy_image_path.clone();
                    settings.general.hdd_directory = dialog.draft_hdd_directory.clone();
                    settings.shortcuts = dialog.draft_shortcuts.clone();
                    settings.ui.theme = dialog.draft_color_scheme;
                    if let Ok(network) = parse_network_defaults(dialog) {
                        apply_network_defaults(&mut settings.network, network);
                    }
                }
                save_settings(&settings);
                Some(Task::none())
            }
            Message::SettingsCategorySelected(category) => {
                if let Some(dialog) = self.settings_dialog.as_mut() {
                    dialog.category = category;
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
            Message::SettingsFloppyImageBrowse => {
                if self.settings_dialog.is_none() {
                    return Some(Task::none());
                }
                let preferred = self
                    .settings_dialog
                    .as_ref()
                    .and_then(|d| d.draft_floppy_image_path.clone())
                    .unwrap_or_else(|| {
                        std::env::var("HOME")
                            .or_else(|_| std::env::var("USERPROFILE"))
                            .map(std::path::PathBuf::from)
                            .unwrap_or_else(|_| std::path::PathBuf::from("."))
                    });
                let mut dialog =
                    rfd::FileDialog::new().add_filter("KR580 floppy image", &["kpd", "img", "bin"]);
                if preferred.exists() && preferred.is_file() {
                    if let Some(parent) = preferred.parent() {
                        dialog = dialog.set_directory(parent);
                    }
                    if let Some(name) = preferred.file_name() {
                        dialog = dialog.set_file_name(name.to_string_lossy().as_ref());
                    }
                } else if preferred.exists() && preferred.is_dir() {
                    dialog = dialog.set_directory(&preferred);
                } else if let Some(parent) = preferred.parent() {
                    dialog = dialog.set_directory(parent);
                }
                if let Some(path) = dialog.pick_file() {
                    return Some(Task::done(Message::SettingsDraftFloppyImageSet(path)));
                }
                Some(Task::none())
            }
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
            Message::SettingsHddDirectoryBrowse => {
                if self.settings_dialog.is_none() {
                    return Some(Task::none());
                }
                let preferred = self
                    .settings_dialog
                    .as_ref()
                    .and_then(|d| d.draft_hdd_directory.clone())
                    .unwrap_or_else(|| {
                        std::env::var("HOME")
                            .or_else(|_| std::env::var("USERPROFILE"))
                            .map(std::path::PathBuf::from)
                            .unwrap_or_else(|_| std::path::PathBuf::from("."))
                    });
                let mut dialog = rfd::FileDialog::new();
                if preferred.exists() && preferred.is_dir() {
                    dialog = dialog.set_directory(&preferred);
                } else if let Some(parent) = preferred.parent() {
                    dialog = dialog.set_directory(parent);
                }
                if let Some(folder) = dialog.pick_folder() {
                    if !is_directory_writable(&folder) {
                        self.error_notice =
                            Some(self.lang.t(Key::ErrHddDirectoryNotWritable).to_owned());
                        self.error_notice_dismiss_at =
                            Some(std::time::Instant::now() + std::time::Duration::from_secs(8));
                        return Some(Task::none());
                    }
                    return Some(Task::done(Message::SettingsDraftHddDirectorySet(folder)));
                }
                Some(Task::none())
            }
            Message::SettingsDraftHddDirectorySet(path) => {
                if let Some(dialog) = self.settings_dialog.as_mut() {
                    dialog.draft_hdd_directory = Some(path);
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
                    dialog.language_dropdown_open = false;
                    dialog.dropdown_highlight = None;
                    dialog.recording_shortcut = None;
                }
                Some(Task::none())
            }
            Message::SettingsResetCancelled => {
                if let Some(dialog) = self.settings_dialog.as_mut() {
                    dialog.reset_confirm_open = false;
                }
                Some(Task::none())
            }
            Message::SettingsResetConfirmed => {
                let default_lang = default_lang();
                let default_speed = SpeedTier::High;
                let default_color_scheme = crate::persistence::ColorScheme::DEFAULT;
                let default_follow_pc = false;
                let default_memory_operand_highlighting = true;
                let network = crate::persistence::NetworkSettings::default();
                let shortcuts = crate::persistence::ShortcutSettings::default();
                if let Some(dialog) = self.settings_dialog.as_mut() {
                    dialog.draft_lang = default_lang;
                    dialog.draft_speed = default_speed;
                    dialog.draft_color_scheme = default_color_scheme;
                    dialog.draft_floppy_image_path = None;
                    dialog.draft_hdd_directory = None;
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
                    dialog.network_error = None;
                    dialog.reset_confirm_open = false;
                }
                self.follow_pc = default_follow_pc;
                self.memory_operand_highlighting = default_memory_operand_highlighting;
                self.shortcut_settings = shortcuts;
                let lang_changed = self.lang != default_lang;
                self.lang = default_lang;
                self.default_speed = default_speed;
                self.color_scheme = default_color_scheme;
                self.apply_speed_tier(default_speed);
                if lang_changed {
                    self.refresh_localized_status();
                }
                Some(Task::done(Message::PersistSettings))
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
}

mod network;
mod section;
mod shortcuts;
use network::{apply_network_defaults, is_directory_writable, parse_network_defaults};
use section::cycle_section;
