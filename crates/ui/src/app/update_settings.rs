use iced::Task;

use super::constants::SETTINGS_SEARCH_INPUT_ID;
use super::messages::{Message, SpeedTier};
use super::network::{NetworkEndpointError, parse_network_endpoint};
use super::settings_modal::{FooterFocus, ResetConfirmFocus, SettingsDialog, SettingsSection};
use super::state::DesktopApp;
use crate::i18n::{Key, Lang};
use crate::settings_storage::{
    language_from_lang, load_settings, preset_from_speed_tier, save_settings,
};

impl DesktopApp {
    /// Dispatches every `Message::Settings*` and the dialog lifecycle
    /// messages (`OpenSettings` / `CloseSettings` / `SaveSettings` /
    /// `PersistSettings`). Returns `Some(task)` on a recognised
    /// settings message, `None` otherwise so the main `update` loop
    /// can fall through to the rest of the match arms.
    pub(super) fn dispatch_settings_message(&mut self, message: Message) -> Option<Task<Message>> {
        match message {
            Message::OpenSettings => {
                self.open_menu = None;
                self.hide_opcode_dropdown();
                let settings = load_settings();
                self.settings_dialog = Some(SettingsDialog::new(
                    self.lang,
                    self.default_speed,
                    self.follow_pc,
                    settings.general.floppy_image_path,
                    settings.general.hdd_directory,
                    settings.network,
                ));
                Some(Task::none())
            }
            Message::CloseSettings => {
                if let Some(dialog) = self.settings_dialog.take() {
                    let lang_changed = self.lang != dialog.original_lang;
                    self.lang = dialog.original_lang;
                    let speed_changed = self.default_speed != dialog.original_speed
                        || self.speed_tier != dialog.original_speed;
                    self.default_speed = dialog.original_speed;
                    self.follow_pc = dialog.original_follow_pc;
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
                let mut settings = load_settings();
                settings.general.floppy_image_path = dialog.draft_floppy_image_path.clone();
                settings.general.hdd_directory = dialog.draft_hdd_directory.clone();
                apply_network_defaults(&mut settings.network, network);
                save_settings(&settings);
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
                if let Some(dialog) = self.settings_dialog.as_ref() {
                    settings.general.follow_pc = dialog.draft_follow_pc;
                    settings.general.floppy_image_path = dialog.draft_floppy_image_path.clone();
                    settings.general.hdd_directory = dialog.draft_hdd_directory.clone();
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
                }
                Some(Task::none())
            }
            Message::SettingsSearchChanged(query) => {
                if let Some(dialog) = self.settings_dialog.as_mut() {
                    dialog.search = query;
                    dialog.language_dropdown_open = false;
                    dialog.dropdown_highlight = None;
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
                // Direct apply: the modal router whitelists only its
                // own message variants, so a `Task::done(SpeedTierChanged)`
                // round-trip would be swallowed mid-flight.
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
                // iced has no global "blur" operation, so when we
                // leave Search we focus a dummy id no widget owns –
                // the focused text_input clears its caret on the
                // next pass and Tab/Enter no longer route to it.
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
                let default_lang = Lang::Ru;
                let default_speed = SpeedTier::Medium;
                let network = k580_persistence::NetworkSettings::default();
                if let Some(dialog) = self.settings_dialog.as_mut() {
                    dialog.draft_lang = default_lang;
                    dialog.draft_speed = default_speed;
                    dialog.draft_floppy_image_path = None;
                    dialog.draft_hdd_directory = None;
                    dialog.original_lang = default_lang;
                    dialog.original_speed = default_speed;
                    dialog.draft_network_client_host = network.host;
                    dialog.draft_network_client_port = network.port.to_string();
                    dialog.draft_network_server_host = network.bind_host;
                    dialog.draft_network_server_port = network.bind_port.to_string();
                    dialog.network_error = None;
                    dialog.reset_confirm_open = false;
                }
                let lang_changed = self.lang != default_lang;
                self.lang = default_lang;
                self.default_speed = default_speed;
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
                if let Some(dialog) = self.settings_dialog.as_mut() {
                    dialog.file_association_registered = k580_ui::file_assoc::is_registered();
                }
                Some(Task::none())
            }
            Message::SettingsFileAssociationUnregister => {
                if let Err(error) = k580_ui::file_assoc::unregister() {
                    self.error_notice =
                        Some(format!("{}: {}", self.lang.t(Key::ErrorPrefix), error));
                    self.error_notice_dismiss_at =
                        Some(std::time::Instant::now() + std::time::Duration::from_secs(8));
                }
                if let Some(dialog) = self.settings_dialog.as_mut() {
                    dialog.file_association_registered = k580_ui::file_assoc::is_registered();
                }
                Some(Task::none())
            }
            _ => None,
        }
    }
}

type NetworkDefaults = ((String, u16), (String, u16));

fn parse_network_defaults(
    dialog: &SettingsDialog,
) -> Result<NetworkDefaults, NetworkEndpointError> {
    let client = parse_network_endpoint(
        &dialog.draft_network_client_host,
        &dialog.draft_network_client_port,
    )?;
    let server = parse_network_endpoint(
        &dialog.draft_network_server_host,
        &dialog.draft_network_server_port,
    )?;
    Ok((client, server))
}

fn apply_network_defaults(
    settings: &mut k580_persistence::NetworkSettings,
    ((client_host, client_port), (server_host, server_port)): NetworkDefaults,
) {
    settings.host = client_host;
    settings.port = client_port;
    settings.bind_host = server_host;
    settings.bind_port = server_port;
}

#[cfg(unix)]
fn is_directory_writable(path: &std::path::Path) -> bool {
    use std::os::unix::ffi::OsStrExt;
    let mut buf = path.as_os_str().as_bytes().to_vec();
    buf.push(0);
    unsafe { libc::access(buf.as_ptr() as *const libc::c_char, libc::W_OK) == 0 }
}

#[cfg(windows)]
fn is_directory_writable(path: &std::path::Path) -> bool {
    use std::time::{SystemTime, UNIX_EPOCH};
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let test_file = path.join(format!(".kr580_{stamp:x}"));
    let ok = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&test_file)
        .is_ok();
    let _ = std::fs::remove_file(&test_file);
    ok
}

fn cycle_section(dialog: &mut SettingsDialog, backward: bool) {
    dialog.language_dropdown_open = false;
    dialog.dropdown_highlight = None;
    let next = if backward {
        dialog.section.previous()
    } else {
        dialog.section.next()
    };
    dialog.section = next;
    match next {
        SettingsSection::Search | SettingsSection::Sidebar => {}
        SettingsSection::Content => {
            dialog.content_focus = Some(if backward {
                dialog.last_content_focus()
            } else {
                dialog.first_content_focus()
            });
        }
        SettingsSection::Footer => {
            dialog.footer_focus = if backward {
                FooterFocus::Save
            } else {
                FooterFocus::Cancel
            };
        }
    }
}
