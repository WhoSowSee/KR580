use iced::Task;

use crate::app::messages::Message;
use crate::app::state::DesktopApp;
use crate::persistence::ShortcutSettings;

impl DesktopApp {
    pub(super) fn dispatch_shortcut_settings_message(
        &mut self,
        message: &Message,
    ) -> Option<Task<Message>> {
        match message {
            Message::SettingsShortcutCaptureStarted(action) => {
                if let Some(dialog) = self.settings_dialog.as_mut() {
                    dialog.recording_shortcut = Some(*action);
                    dialog.language_dropdown_open = false;
                    dialog.dropdown_highlight = None;
                }
                Some(Task::none())
            }
            Message::SettingsShortcutCaptured(binding) => {
                if let Some(dialog) = self.settings_dialog.as_mut()
                    && let Some(action) = dialog.recording_shortcut.take()
                {
                    dialog.draft_shortcuts.assign(action, *binding);
                    self.shortcut_settings = dialog.draft_shortcuts.clone();
                }
                Some(Task::none())
            }
            Message::SettingsShortcutCaptureCancelled => {
                if let Some(dialog) = self.settings_dialog.as_mut() {
                    dialog.recording_shortcut = None;
                }
                Some(Task::none())
            }
            Message::SettingsShortcutsReset => {
                if let Some(dialog) = self.settings_dialog.as_mut() {
                    dialog.draft_shortcuts = ShortcutSettings::default();
                    self.shortcut_settings = dialog.draft_shortcuts.clone();
                    dialog.recording_shortcut = None;
                }
                Some(Task::none())
            }
            _ => None,
        }
    }
}
