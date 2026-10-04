use super::network::{NetworkDefaults, apply_network_defaults, is_directory_writable};
use crate::app::{DesktopApp, SettingsNotice};
use crate::backend::AppError;
use crate::i18n::Key;
use crate::persistence::Settings;
use crate::runtime::file_work::{FileRequest, FileResult, SettingsAction};
use crate::settings_storage::{
    language_from_lang, load_settings, preset_from_speed_tier, save_settings,
};
use std::sync::Arc;
use std::time::Instant;

impl DesktopApp {
    pub(super) fn settings_save_pending(&self) -> bool {
        self.preferences
            .settings_dialog
            .as_ref()
            .is_some_and(|dialog| dialog.saving)
    }
    pub(crate) fn queue_settings_change(
        &mut self,
        action: SettingsAction,
        change: impl FnOnce(&mut Settings) + Send + 'static,
    ) {
        self.queue_file_work(FileRequest::Settings(action), move || {
            let mut settings = load_settings();
            change(&mut settings);
            save_settings(&settings)?;
            Ok(FileResult::Settings(Box::new(settings)))
        });
    }

    pub(super) fn save_settings_dialog(&mut self, network: NetworkDefaults, notice: Key) {
        if self.settings_save_pending() {
            return;
        }
        let Some(dialog) = self.preferences.settings_dialog.as_ref() else {
            return;
        };
        let baseline = Arc::new(dialog.clone());
        let draft = Arc::clone(&baseline);
        self.preferences.settings_dialog.as_mut().unwrap().saving = true;
        self.queue_settings_change(
            SettingsAction::Dialog {
                generation: self.preferences.dialog_generation,
                baseline,
                active_speed: self.execution.speed_tier,
                notice,
            },
            move |settings| {
                settings.general.language = language_from_lang(draft.draft_lang);
                settings.general.default_speed = preset_from_speed_tier(draft.draft_speed);
                settings.general.follow_pc = draft.draft_follow_pc;
                settings.general.memory_operand_highlighting =
                    draft.draft_memory_operand_highlighting;
                settings.general.show_file_name = draft.draft_show_file_name;
                settings.general.monitor_split = draft.draft_monitor_split;
                settings.general.floppy_image_path = draft.draft_floppy_image_path.clone();
                settings.general.hdd_directory = draft.draft_hdd_directory.clone();
                settings
                    .general
                    .set_printer_settings(draft.draft_printer_settings.clone());
                settings.general.printer_dialog_mode = draft.draft_printer_dialog_mode;
                settings.ui.theme = draft.draft_color_scheme;
                apply_network_defaults(&mut settings.network, network);
                settings.shortcuts = draft.draft_shortcuts.clone();
            },
        );
    }

    pub(crate) fn finish_settings_write(
        &mut self,
        action: SettingsAction,
        result: Result<Box<Settings>, AppError>,
    ) {
        let result = result.map(|settings| {
            self.preferences.stored = *settings;
        });
        match action {
            SettingsAction::Dialog {
                generation,
                baseline,
                active_speed,
                notice,
            } => {
                if generation != self.preferences.dialog_generation {
                    return;
                }
                if let Some(dialog) = self.preferences.settings_dialog.as_mut() {
                    dialog.saving = false;
                }
                match result {
                    Ok(()) => {
                        self.commit_settings_dialog_state(&baseline, active_speed);
                        self.show_settings_notice(notice);
                    }
                    Err(error) => {
                        self.preferences.settings_notice = None;
                        self.show_error_notice(crate::runtime::humanize_error::humanize(
                            &error,
                            self.preferences.lang,
                        ));
                    }
                }
            }
            SettingsAction::Quiet => {
                if let Err(error) = result {
                    self.show_error_notice(crate::runtime::humanize_error::humanize(
                        &error,
                        self.preferences.lang,
                    ));
                }
            }
            SettingsAction::PrinterPresets {
                printer_name,
                selected,
            } => {
                self.finish_printer_preset_write(&printer_name, selected, result);
            }
        }
    }

    pub(super) fn validate_hdd_directory(&mut self, path: std::path::PathBuf) {
        if self.preferences.settings_dialog.is_none() {
            return;
        }
        self.preferences.directory_generation =
            self.preferences.directory_generation.wrapping_add(1);
        let request = FileRequest::HddDirectory {
            dialog_generation: self.preferences.dialog_generation,
            generation: self.preferences.directory_generation,
            path: path.clone(),
        };
        self.queue_file_work(request, move || {
            Ok(FileResult::DirectoryValidated(is_directory_writable(&path)))
        });
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
    #[test]
    fn failed_write_preserves_draft_and_success_commits_only_submitted_values() {
        let (mut app, _) = DesktopApp::with_initial_path(None);
        app.preferences.settings_dialog = Some(SettingsDialog::new(
            crate::app::settings_modal::SettingsInitialState {
                follow_pc: true,
                ..Default::default()
            },
        ));
        let generation = app.preferences.dialog_generation;
        let active_speed = app.execution.speed_tier;
        app.preferences
            .settings_dialog
            .as_mut()
            .unwrap()
            .draft_follow_pc = false;
        let _task = app.update(crate::app::Message::SettingsDraftPrinterDialogModeSet(
            crate::persistence::PrinterDialogMode::System,
        ));
        let baseline = Arc::new(app.preferences.settings_dialog.as_ref().unwrap().clone());
        app.finish_settings_write(
            SettingsAction::Dialog {
                generation,
                baseline: Arc::clone(&baseline),
                active_speed,
                notice: Key::SettingsSavedNotice,
            },
            Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied).into()),
        );
        let dialog = app.preferences.settings_dialog.as_ref().unwrap();
        assert!(!dialog.draft_follow_pc);
        assert!(dialog.original_follow_pc);
        assert!(app.preferences.settings_notice.is_none());
        assert!(app.shell.error_notice.is_some());
        app.preferences
            .settings_dialog
            .as_mut()
            .unwrap()
            .draft_follow_pc = true;
        let _task = app.update(crate::app::Message::SettingsDraftPrinterDialogModeSet(
            crate::persistence::PrinterDialogMode::Custom,
        ));
        app.finish_settings_write(
            SettingsAction::Dialog {
                generation,
                baseline,
                active_speed,
                notice: Key::SettingsSavedNotice,
            },
            Ok(Box::new(Settings::default())),
        );
        let dialog = app.preferences.settings_dialog.as_ref().unwrap();
        assert!(dialog.draft_follow_pc);
        assert!(!dialog.original_follow_pc);
        assert_eq!(
            dialog.original_printer_dialog_mode,
            crate::persistence::PrinterDialogMode::System
        );
        assert_eq!(
            app.printer_setup.printer_dialog_mode,
            crate::persistence::PrinterDialogMode::Custom
        );
    }
}
