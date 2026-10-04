use super::network::{NetworkDefaults, apply_network_defaults};
use crate::app::{DesktopApp, SettingsDialog, SettingsNotice};
use crate::i18n::Key;
use crate::settings_storage::{
    language_from_lang, load_settings, preset_from_speed_tier, save_settings,
};
use std::time::Instant;

impl DesktopApp {
    pub(super) fn save_settings_dialog(
        &self,
        dialog: &SettingsDialog,
        network: NetworkDefaults,
    ) -> Result<(), crate::persistence::SettingsError> {
        let mut settings = load_settings();
        settings.general.language = language_from_lang(self.preferences.lang);
        settings.general.default_speed = preset_from_speed_tier(self.preferences.default_speed);
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
        save_settings(&settings)
    }

    pub(super) fn finish_settings_save(
        &mut self,
        result: Result<(), crate::persistence::SettingsError>,
        notice: Key,
    ) {
        match result {
            Ok(()) => {
                self.commit_settings_dialog_state();
                self.show_settings_notice(notice);
            }
            Err(error) => {
                self.preferences.settings_notice = None;
                let notice =
                    crate::runtime::humanize_error::humanize(&error.into(), self.preferences.lang);
                self.show_error_notice(notice);
            }
        }
    }

    pub(super) fn show_settings_notice(&mut self, message_key: Key) {
        let started_at = Instant::now();
        self.preferences.settings_notice = Some(match self.preferences.settings_notice.take() {
            Some(notice) => notice.restarted(message_key, started_at),
            None => SettingsNotice::new(message_key, started_at),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::SettingsDialog;
    use crate::persistence::NetworkSettings;
    #[test]
    fn failed_storage_write_keeps_draft_and_cancel_baseline_without_success() {
        let (mut app, _) = DesktopApp::with_initial_path(None);
        app.preferences.settings_dialog = Some(SettingsDialog::new(
            crate::app::settings_modal::SettingsInitialState {
                lang: app.preferences.lang,
                speed: app.preferences.default_speed,
                color_scheme: app.preferences.color_scheme,
                follow_pc: true,
                memory_operand_highlighting: true,
                floppy_image_path: None,
                hdd_directory: None,
                network: NetworkSettings::default(),
                shortcuts: app.preferences.shortcut_settings.clone(),
                ..Default::default()
            },
        ));
        app.preferences
            .settings_dialog
            .as_mut()
            .unwrap()
            .draft_follow_pc = false;
        app.finish_settings_save(
            Err(crate::persistence::SettingsError::Io(std::io::Error::from(
                std::io::ErrorKind::PermissionDenied,
            ))),
            Key::SettingsSavedNotice,
        );
        let dialog = app.preferences.settings_dialog.as_ref().unwrap();
        assert!(!dialog.draft_follow_pc);
        assert!(dialog.original_follow_pc);
        assert!(app.preferences.settings_notice.is_none());
        assert!(app.shell.error_notice.is_some());
    }
}
