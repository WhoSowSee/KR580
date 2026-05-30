use iced::Task;

use super::constants::SETTINGS_SEARCH_INPUT_ID;
use super::messages::{Message, SpeedTier};
use super::settings_modal::{
    ContentFocus, FooterFocus, ResetConfirmFocus, SettingsDialog, SettingsSection,
};
use super::state::DesktopApp;
use crate::i18n::Lang;
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
                self.settings_dialog = Some(SettingsDialog::new(self.lang, self.default_speed));
                Some(Task::none())
            }
            Message::CloseSettings => {
                if let Some(dialog) = self.settings_dialog.take() {
                    let lang_changed = self.lang != dialog.original_lang;
                    self.lang = dialog.original_lang;
                    let speed_changed = self.default_speed != dialog.original_speed
                        || self.speed_tier != dialog.original_speed;
                    self.default_speed = dialog.original_speed;
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
                // leave Search we focus a dummy id no widget owns —
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
                if let Some(dialog) = self.settings_dialog.as_mut() {
                    dialog.draft_lang = default_lang;
                    dialog.draft_speed = default_speed;
                    dialog.original_lang = default_lang;
                    dialog.original_speed = default_speed;
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
            _ => None,
        }
    }
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
    // SPEEDS reference keeps the import live for tests that walk
    // the speed-segment cycle directly.
    let _ = ContentFocus::SPEEDS;
}
