use crate::app::{DesktopApp, StatusKind};
use crate::backend::AppError;
use crate::i18n::Key;
use crate::runtime::storage_files::ImageRead;
use k580_ui::devices::StorageKind;
use std::path::PathBuf;

pub(crate) enum FileRequest {
    ImportTargets {
        generation: u64,
        path: PathBuf,
    },
    Image {
        kind: StorageKind,
        generation: u64,
        path: PathBuf,
    },
    MonitorSaved(PathBuf),
    FloppySaved,
    HddDeleted {
        path: PathBuf,
        generation: u64,
    },
    Settings(SettingsAction),
    HddDirectory {
        dialog_generation: u64,
        generation: u64,
        path: PathBuf,
    },
}

pub(crate) enum SettingsAction {
    Dialog {
        generation: u64,
        baseline: std::sync::Arc<crate::app::SettingsDialog>,
        active_speed: crate::app::SpeedTier,
        notice: Key,
    },
    Quiet,
    PrinterPresets {
        printer_name: String,
        selected: Option<String>,
    },
}

pub(crate) enum FileResult {
    ImportTargets(Vec<String>),
    Image(ImageRead),
    Saved(PathBuf),
    Deleted,
    Settings(Box<crate::persistence::Settings>),
    DirectoryValidated(bool),
}

pub(crate) struct FileCompletion {
    pub(crate) request: FileRequest,
    pub(crate) result: Result<FileResult, AppError>,
}

impl DesktopApp {
    pub(super) fn apply_file_completion(&mut self, completion: FileCompletion) {
        match completion.request {
            FileRequest::Settings(action) => {
                self.finish_settings_write(
                    action,
                    completion.result.map(|result| {
                        let FileResult::Settings(settings) = result else {
                            unreachable!()
                        };
                        settings
                    }),
                );
            }
            FileRequest::HddDirectory {
                dialog_generation,
                generation,
                path,
            } => {
                if self.preferences.dialog_generation != dialog_generation
                    || self.preferences.directory_generation != generation
                    || self.preferences.settings_dialog.is_none()
                {
                    return;
                }
                match completion.result {
                    Ok(FileResult::DirectoryValidated(true)) => {
                        self.preferences
                            .settings_dialog
                            .as_mut()
                            .unwrap()
                            .draft_hdd_directory = Some(path);
                    }
                    Ok(FileResult::DirectoryValidated(false)) => self.show_error_notice(
                        self.preferences.lang.t(Key::ErrHddDirectoryNotWritable),
                    ),
                    Ok(_) => unreachable!(),
                    Err(error) => self.show_file_error(error),
                }
            }
            FileRequest::ImportTargets { generation, path } => {
                self.finish_import_targets(
                    generation,
                    path,
                    completion.result.map(|result| {
                        let FileResult::ImportTargets(targets) = result else {
                            unreachable!()
                        };
                        targets
                    }),
                );
            }
            FileRequest::Image {
                kind,
                generation,
                path,
            } => {
                self.apply_image_contents(
                    kind,
                    generation,
                    path,
                    completion.result.map(|result| {
                        let FileResult::Image(image) = result else {
                            unreachable!()
                        };
                        image
                    }),
                );
            }
            FileRequest::MonitorSaved(path) => match completion.result {
                Ok(_) => self.set_status(StatusKind::MonitorImageSaved {
                    display: path.display().to_string(),
                }),
                Err(error) => self.show_file_error(error),
            },
            FileRequest::FloppySaved => match completion.result {
                Ok(FileResult::Saved(path)) => self.set_status_custom(format!(
                    "{}: {}",
                    self.preferences.lang.t(Key::FloppyBufferSaved),
                    path.display()
                )),
                Ok(_) => unreachable!(),
                Err(error) => self.show_file_error(error),
            },
            FileRequest::HddDeleted { path, generation } => match completion.result {
                _ if self.panels.hdd_generation != generation => {}
                Ok(_) if self.snapshot.devices.hdd.path.as_ref() == Some(&path) => {
                    self.dispatch_action(
                        crate::backend::AppCommand::DetachHddFile,
                        crate::app::BackendAction::HddDeleted(path),
                    );
                }
                Ok(_) => {}
                Err(error) => self.show_file_error(error),
            },
        }
    }

    fn show_file_error(&mut self, error: AppError) {
        self.show_error_notice(crate::runtime::humanize_error::humanize(
            &error,
            self.preferences.lang,
        ));
    }
}
