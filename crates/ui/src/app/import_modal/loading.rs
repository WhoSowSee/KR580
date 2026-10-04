use super::{DesktopApp, ImportFileFormat, ImportModalFocus};
use crate::i18n::Key;
use crate::persistence::Importers;
use std::path::PathBuf;

impl DesktopApp {
    pub(crate) fn load_import_file(&mut self, path: PathBuf) {
        let Some(format) = ImportFileFormat::from_path(&path) else {
            self.clear_import_file_selection();
            self.import.import_modal_focus = ImportModalFocus::Browse;
            self.import.import_error = Some(
                self.preferences
                    .lang
                    .t(Key::ErrUnsupportedImportFile)
                    .to_owned(),
            );
            return;
        };
        self.clear_import_file_selection();
        let generation = self.import.generation;
        self.import.loading = true;
        self.import.import_file_display = path.display().to_string();
        self.import.import_file_format = Some(format);
        self.import.import_file_path = Some(path.clone());
        self.import.import_error = None;
        self.queue_file_work(
            crate::runtime::file_work::FileRequest::ImportTargets {
                generation,
                path: path.clone(),
            },
            move || {
                let targets = match format {
                    ImportFileFormat::Xlsx => Importers::xlsx_sheet_names(&path),
                    ImportFileFormat::Text => Importers::txt_section_names(&path),
                }?;
                Ok(crate::runtime::file_work::FileResult::ImportTargets(
                    targets,
                ))
            },
        );
    }

    pub(crate) fn finish_import_targets(
        &mut self,
        generation: u64,
        path: PathBuf,
        targets: Result<Vec<String>, crate::backend::AppError>,
    ) {
        if !self.import.import_modal_open
            || self.import.generation != generation
            || self.import.import_file_path.as_ref() != Some(&path)
        {
            return;
        }
        self.import.loading = false;
        match targets {
            Ok(targets) => {
                self.import.import_target_options = targets;
                self.import.import_target_input = self
                    .import
                    .import_target_options
                    .first()
                    .cloned()
                    .unwrap_or_default();
                self.import.import_modal_focus = if self.import.import_target_options.is_empty() {
                    ImportModalFocus::Confirm
                } else {
                    ImportModalFocus::Target
                };
            }
            Err(err) => {
                self.import.import_target_options.clear();
                self.import.import_target_input.clear();
                self.import.import_modal_focus = ImportModalFocus::Browse;
                self.import.import_error = Some(crate::runtime::humanize_error::humanize(
                    &err,
                    self.preferences.lang,
                ));
            }
        }
    }
}
