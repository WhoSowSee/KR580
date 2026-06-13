use iced::Task;

use super::focus::{
    ContentFocus, FooterFocus, ResetConfirmFocus, SettingsCategory, SettingsSection,
};
use crate::app::messages::{Message, SpeedTier};
use crate::app::state::DesktopApp;
use crate::i18n::Lang;

impl DesktopApp {
    /// Mirrors `route_discard_modal_message`: while the settings modal
    /// is open, only messages that drive its own state pass through –
    /// everything else (CtrlS, Tick, ArrowKey, ...) is swallowed so the
    /// rest of the app stays inert.
    pub(crate) fn route_settings_modal_message(
        &mut self,
        message: &Message,
    ) -> Option<Task<Message>> {
        let dialog = self.settings_dialog.as_ref()?;
        let reset_open = dialog.reset_confirm_open;

        match message {
            Message::Tick
            | Message::CursorMoved(_)
            | Message::ModifiersChanged(_)
            | Message::FocusReconciled(_)
            | Message::ResolveFocusedTracker(_) => None,
            Message::CloseSettings
            | Message::SaveSettings
            | Message::PersistSettings
            | Message::SettingsCategorySelected(_)
            | Message::SettingsSearchChanged(_)
            | Message::SettingsDraftLanguageChanged(_)
            | Message::SettingsDraftSpeedChanged(_)
            | Message::SettingsDraftFollowPcSet(_)
            | Message::SettingsHddDirectoryBrowse
            | Message::SettingsDraftHddDirectorySet(_)
            | Message::SettingsNetworkClientHostChanged(_)
            | Message::SettingsNetworkClientPortChanged(_)
            | Message::SettingsNetworkServerHostChanged(_)
            | Message::SettingsNetworkServerPortChanged(_)
            | Message::SettingsLanguageDropdownToggled
            | Message::SettingsResetRequested
            | Message::SettingsResetConfirmed
            | Message::SettingsResetCancelled
            | Message::SettingsSectionCycle { .. } => None,
            Message::EscPressed => {
                if reset_open {
                    Some(Task::done(Message::SettingsResetCancelled))
                } else if dialog.language_dropdown_open {
                    Some(Task::done(Message::SettingsLanguageDropdownToggled))
                } else {
                    Some(Task::done(Message::CloseSettings))
                }
            }
            Message::EnterPressed => {
                if reset_open {
                    let action = match dialog.reset_confirm_focus {
                        ResetConfirmFocus::Cancel => Message::SettingsResetCancelled,
                        ResetConfirmFocus::Confirm => Message::SettingsResetConfirmed,
                    };
                    return Some(Task::done(action));
                }
                if dialog.language_dropdown_open {
                    let target = dialog.dropdown_highlight.unwrap_or(dialog.draft_lang);
                    return Some(Task::done(Message::SettingsDraftLanguageChanged(target)));
                }
                Some(self.handle_settings_enter())
            }
            Message::FocusCycle { backward } => {
                Some(self.handle_settings_tab(*backward, reset_open))
            }
            Message::ArrowKey(direction) => Some(self.handle_settings_vertical_arrow(*direction)),
            Message::HorizontalArrowKey(direction) => {
                Some(self.handle_settings_horizontal_arrow(*direction))
            }
            Message::MousePressed | Message::MousePressedIgnored => None,
            _ => Some(Task::none()),
        }
    }

    fn handle_settings_enter(&mut self) -> Task<Message> {
        let Some(dialog) = self.settings_dialog.as_ref() else {
            return Task::none();
        };
        match dialog.section {
            SettingsSection::Footer => {
                let action = match dialog.footer_focus {
                    FooterFocus::Reset => Message::SettingsResetRequested,
                    FooterFocus::Cancel => Message::CloseSettings,
                    FooterFocus::Save => Message::SaveSettings,
                };
                Task::done(action)
            }
            SettingsSection::Sidebar | SettingsSection::Search => Task::none(),
            SettingsSection::Content => self.activate_focused_content(),
        }
    }

    fn activate_focused_content(&mut self) -> Task<Message> {
        let dialog = self.settings_dialog.as_ref().expect("dialog open");
        let Some(focus) = dialog.content_focus else {
            return Task::none();
        };
        match focus {
            ContentFocus::LanguageAnchor => Task::done(Message::SettingsLanguageDropdownToggled),
            ContentFocus::SpeedSlow => {
                Task::done(Message::SettingsDraftSpeedChanged(SpeedTier::Slow))
            }
            ContentFocus::SpeedMedium => {
                Task::done(Message::SettingsDraftSpeedChanged(SpeedTier::Medium))
            }
            ContentFocus::SpeedFast => {
                Task::done(Message::SettingsDraftSpeedChanged(SpeedTier::High))
            }
            ContentFocus::SpeedMax => {
                Task::done(Message::SettingsDraftSpeedChanged(SpeedTier::Max))
            }
            ContentFocus::FollowPc => {
                let current = dialog.draft_follow_pc;
                Task::done(Message::SettingsDraftFollowPcSet(!current))
            }
            ContentFocus::HddDirectory => Task::done(Message::SettingsHddDirectoryBrowse),
            ContentFocus::Theme => Task::none(),
            ContentFocus::Shortcuts => Task::none(),
        }
    }

    fn handle_settings_tab(&mut self, backward: bool, reset_open: bool) -> Task<Message> {
        let Some(dialog) = self.settings_dialog.as_mut() else {
            return Task::none();
        };
        if reset_open {
            dialog.reset_confirm_focus = dialog.reset_confirm_focus.toggled();
            return Task::none();
        }
        match dialog.section {
            SettingsSection::Search => {}
            SettingsSection::Sidebar => {
                let cur = SettingsCategory::ALL
                    .iter()
                    .position(|c| *c == dialog.category)
                    .unwrap_or(0);
                let len = SettingsCategory::ALL.len();
                let next_idx = if backward {
                    (cur + len - 1) % len
                } else {
                    (cur + 1) % len
                };
                let next = SettingsCategory::ALL[next_idx];
                return Task::done(Message::SettingsCategorySelected(next));
            }
            SettingsSection::Content => {
                let current = dialog
                    .content_focus
                    .unwrap_or_else(|| dialog.first_content_focus());
                let stepped = if backward {
                    dialog.previous_content_focus(current)
                } else {
                    dialog.next_content_focus(current)
                };
                dialog.content_focus = Some(stepped.unwrap_or_else(|| {
                    if backward {
                        dialog.last_content_focus()
                    } else {
                        dialog.first_content_focus()
                    }
                }));
            }
            SettingsSection::Footer => {
                dialog.footer_focus = if backward {
                    dialog.footer_focus.previous()
                } else {
                    dialog.footer_focus.next()
                };
            }
        }
        Task::none()
    }

    fn handle_settings_vertical_arrow(&mut self, direction: i32) -> Task<Message> {
        let Some(dialog) = self.settings_dialog.as_mut() else {
            return Task::none();
        };
        if dialog.reset_confirm_open {
            return Task::none();
        }
        if dialog.language_dropdown_open {
            // ArrowKey carries +1 for Up, -1 for Down – flip so a
            // visual Down moves to the next list item. Stop at the
            // ends instead of wrapping so the highlight doesn't
            // unexpectedly jump to the opposite edge.
            let current = dialog.dropdown_highlight.unwrap_or(dialog.draft_lang);
            let next = match (current, direction) {
                (Lang::Ru, d) if d < 0 => Lang::En,
                (Lang::En, d) if d > 0 => Lang::Ru,
                _ => return Task::none(),
            };
            dialog.dropdown_highlight = Some(next);
            return Task::none();
        }
        if dialog.section == SettingsSection::Sidebar {
            let cur = SettingsCategory::ALL
                .iter()
                .position(|c| *c == dialog.category)
                .unwrap_or(0) as i32;
            let len = SettingsCategory::ALL.len() as i32;
            let next = cur - direction;
            if next < 0 || next >= len {
                return Task::none();
            }
            let target = SettingsCategory::ALL[next as usize];
            return Task::done(Message::SettingsCategorySelected(target));
        }
        Task::none()
    }

    fn handle_settings_horizontal_arrow(&mut self, direction: i32) -> Task<Message> {
        let Some(dialog) = self.settings_dialog.as_mut() else {
            return Task::none();
        };
        if dialog.section != SettingsSection::Content {
            return Task::none();
        }
        let Some(focus) = dialog.content_focus else {
            return Task::none();
        };
        let Some(idx) = ContentFocus::SPEEDS.iter().position(|f| *f == focus) else {
            return Task::none();
        };
        let len = ContentFocus::SPEEDS.len() as i32;
        let target_idx = (idx as i32 + direction).rem_euclid(len) as usize;
        dialog.content_focus = Some(ContentFocus::SPEEDS[target_idx]);
        Task::none()
    }
}
