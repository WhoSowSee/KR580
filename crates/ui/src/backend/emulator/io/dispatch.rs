use super::{Emulator, IoCompletion, IoJob, spawn};
use crate::backend::{AppCommand, AppEvent, RequestId};
use crate::persistence::ExportOptions;
use crossbeam_channel::Sender;

impl Emulator {
    pub(in crate::backend) fn start_io_request(
        &mut self,
        id: RequestId,
        command: &AppCommand,
        completion_tx: Sender<IoCompletion>,
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
        let mut events = Vec::new();
        if matches!(
            command,
            AppCommand::LoadProgram(_) | AppCommand::LoadSubprogram { .. }
        ) {
            self.document_generation = self.document_generation.wrapping_add(1);
            if self.running {
                self.running = false;
                events.push(AppEvent::Stopped);
            }
        }
        spawn(id, self.document_generation, job, completion_tx);
        Some(events)
    }
}
