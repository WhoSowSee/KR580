use super::{Emulator, IoJob, IoWorker};
use crate::backend::{AppCommand, AppEvent, RequestId};
use crate::persistence::ExportOptions;

impl Emulator {
    pub(in crate::backend) fn start_io_request(
        &mut self,
        id: RequestId,
        command: &AppCommand,
        worker: &IoWorker,
    ) -> Option<Vec<AppEvent>> {
        let job = match command {
            AppCommand::SaveProgram(path) => IoJob::SaveProgram {
                path: path.clone(),
                state: Box::new(self.cpu.clone()),
            },
            AppCommand::LoadProgram(path) => IoJob::LoadProgram { path: path.clone() },
            AppCommand::SaveSubprogram { path, start, end } => IoJob::SaveSubprogram {
                path: path.clone(),
                state: Box::new(self.cpu.clone()),
                start: *start,
                end: *end,
            },
            AppCommand::LoadSubprogram { path, start } => IoJob::LoadSubprogram {
                path: path.clone(),
                start: *start,
            },
            AppCommand::ExportTxt(path) => IoJob::ExportTxt {
                path: path.clone(),
                model: self.export_model(),
            },
            AppCommand::ExportXlsx(path) => IoJob::ExportXlsx {
                path: path.clone(),
                model: self.export_model(),
                options: ExportOptions::default(),
            },
            AppCommand::ExportTxtWithOptions(path, options) if options.text_sections.is_empty() => {
                IoJob::ExportTxt {
                    path: path.clone(),
                    model: self.export_model_with_options(options),
                }
            }
            AppCommand::ExportTxtWithOptions(path, options) => IoJob::ExportTxtSections {
                path: path.clone(),
                models: self.export_text_models(options),
            },
            AppCommand::ExportXlsxWithOptions(path, options) if options.xlsx_pages.is_empty() => {
                IoJob::ExportXlsx {
                    path: path.clone(),
                    model: self.export_model_with_options(options),
                    options: options.clone(),
                }
            }
            AppCommand::ExportXlsxWithOptions(path, options) => IoJob::ExportXlsxPages {
                path: path.clone(),
                models: self.export_xlsx_models(options),
            },
            AppCommand::ImportTxt(path) => IoJob::ImportTxt { path: path.clone() },
            AppCommand::ImportTxtSection(path, section) => IoJob::ImportTxtSection {
                path: path.clone(),
                section: section.clone(),
            },
            AppCommand::ImportXlsx(path) => IoJob::ImportXlsx { path: path.clone() },
            AppCommand::ImportXlsxSheet(path, sheet) => IoJob::ImportXlsxSheet {
                path: path.clone(),
                sheet: sheet.clone(),
            },
            _ => return None,
        };
        let replacement = matches!(
            command,
            AppCommand::LoadProgram(_) | AppCommand::LoadSubprogram { .. }
        );
        let generation = self
            .document_generation
            .wrapping_add(u64::from(replacement));
        if let Err(error) = worker.enqueue(id, generation, job) {
            return Some(vec![
                AppEvent::ErrorRaised(error.clone()),
                AppEvent::CommandFinished {
                    id,
                    result: Err(error),
                },
            ]);
        }
        let mut events = Vec::new();
        if replacement {
            self.document_generation = generation;
            if self.running {
                self.running = false;
                events.push(AppEvent::Stopped);
            }
        }
        Some(events)
    }
}
