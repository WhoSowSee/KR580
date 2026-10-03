use super::super::AppError;
use crate::persistence::ExportOptions;
use crate::persistence::{Exporters, Importers, ProgramSerializer, SubprogramSerializer};
use crossbeam_channel::Sender;
use k580_core::Cpu8080State;
use std::path::PathBuf;

use super::super::command::{CommandResult, RequestId};
use super::super::{AppCommand, AppEvent};
use super::Emulator;

pub(crate) enum IoJob {
    SaveProgram {
        path: PathBuf,
        state: Box<Cpu8080State>,
    },
    LoadProgram {
        path: PathBuf,
    },
    SaveSubprogram {
        path: PathBuf,
        state: Box<Cpu8080State>,
        start: u16,
        end: u16,
    },
    LoadSubprogram {
        path: PathBuf,
        state: Box<Cpu8080State>,
        start: u16,
    },
    ExportTxt {
        path: PathBuf,
        model: crate::persistence::ExportModel,
    },
    ExportXlsx {
        path: PathBuf,
        model: crate::persistence::ExportModel,
        options: ExportOptions,
    },
    ExportTxtSections {
        path: PathBuf,
        models: Vec<(String, crate::persistence::ExportModel)>,
    },
    ExportXlsxPages {
        path: PathBuf,
        models: Vec<(String, crate::persistence::ExportModel, ExportOptions)>,
    },
    ImportTxt {
        path: PathBuf,
        state: Box<Cpu8080State>,
    },
    ImportTxtSection {
        path: PathBuf,
        section: String,
        state: Box<Cpu8080State>,
    },
    ImportXlsx {
        path: PathBuf,
        state: Box<Cpu8080State>,
    },
    ImportXlsxSheet {
        path: PathBuf,
        sheet: String,
        state: Box<Cpu8080State>,
    },
}

pub(crate) struct IoCompletion {
    pub id: RequestId,
    pub result: Result<CommandResult, AppError>,
    pub state_update: Option<Box<Cpu8080State>>,
}

pub(crate) fn spawn(id: RequestId, job: IoJob, completion_tx: Sender<IoCompletion>) {
    std::thread::spawn(move || {
        let result = run(job);
        let (result, state_update) = match result {
            Ok((result, state_update)) => (Ok(result), state_update),
            Err(error) => (Err(error), None),
        };
        let _ = completion_tx.send(IoCompletion {
            id,
            result,
            state_update,
        });
    });
}

impl Emulator {
    pub(in crate::backend) fn start_io_request(
        &self,
        id: RequestId,
        command: &AppCommand,
        completion_tx: Sender<IoCompletion>,
    ) -> bool {
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
                state: Box::new(self.cpu.clone()),
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
            AppCommand::ImportTxt(path) => IoJob::ImportTxt {
                path: path.clone(),
                state: Box::new(self.cpu.clone()),
            },
            AppCommand::ImportTxtSection(path, section) => IoJob::ImportTxtSection {
                path: path.clone(),
                section: section.clone(),
                state: Box::new(self.cpu.clone()),
            },
            AppCommand::ImportXlsx(path) => IoJob::ImportXlsx {
                path: path.clone(),
                state: Box::new(self.cpu.clone()),
            },
            AppCommand::ImportXlsxSheet(path, sheet) => IoJob::ImportXlsxSheet {
                path: path.clone(),
                sheet: sheet.clone(),
                state: Box::new(self.cpu.clone()),
            },
            _ => return false,
        };
        spawn(id, job, completion_tx);
        true
    }

    pub(in crate::backend) fn finish_io(&mut self, completion: IoCompletion) -> Vec<AppEvent> {
        if let Some(state) = completion.state_update {
            self.cpu = *state;
            self.running = false;
            self.instructions_since_run = 0;
        }
        let mut events = Vec::new();
        let result = match completion.result {
            Ok(result) => Ok(result),
            Err(error) => {
                events.push(AppEvent::ErrorRaised(error.clone()));
                Err(error)
            }
        };
        events.push(AppEvent::StateChanged(Box::new(self.snapshot())));
        events.push(AppEvent::CommandFinished {
            id: completion.id,
            result,
        });
        events
    }
}

fn run(job: IoJob) -> Result<(CommandResult, Option<Box<Cpu8080State>>), AppError> {
    match job {
        IoJob::SaveProgram { path, state } => {
            ProgramSerializer::save_file(path, &state)?;
            Ok((CommandResult::SavedProgram, None))
        }
        IoJob::LoadProgram { path } => {
            let state = ProgramSerializer::load_file(path)?;
            Ok((CommandResult::LoadedProgram, Some(Box::new(state))))
        }
        IoJob::SaveSubprogram {
            path,
            state,
            start,
            end,
        } => {
            SubprogramSerializer::save_file(path, &state, start, end)?;
            Ok((CommandResult::SavedSubprogram, None))
        }
        IoJob::LoadSubprogram {
            path,
            mut state,
            start,
        } => {
            let end = SubprogramSerializer::load_into_state(&path, start, &mut state)?;
            Ok((CommandResult::LoadedSubprogram { end }, Some(state)))
        }
        IoJob::ExportTxt { path, model } => {
            Exporters::write_txt(path, &model)?;
            Ok((CommandResult::Exported, None))
        }
        IoJob::ExportXlsx {
            path,
            model,
            options,
        } => {
            Exporters::write_xlsx_with_options(path, &model, &options)?;
            Ok((CommandResult::Exported, None))
        }
        IoJob::ExportTxtSections { path, models } => {
            Exporters::write_txt_sections(path, &models)?;
            Ok((CommandResult::Exported, None))
        }
        IoJob::ExportXlsxPages { path, models } => {
            Exporters::write_xlsx_pages(path, &models)?;
            Ok((CommandResult::Exported, None))
        }
        IoJob::ImportTxt { path, mut state } => {
            Importers::read_txt(path)?.apply_to(&mut state)?;
            Ok((CommandResult::Imported, Some(state)))
        }
        IoJob::ImportTxtSection {
            path,
            section,
            mut state,
        } => {
            Importers::read_txt_section(path, &section)?.apply_to(&mut state)?;
            Ok((CommandResult::Imported, Some(state)))
        }
        IoJob::ImportXlsx { path, mut state } => {
            Importers::read_xlsx(path)?.apply_to(&mut state)?;
            Ok((CommandResult::Imported, Some(state)))
        }
        IoJob::ImportXlsxSheet {
            path,
            sheet,
            mut state,
        } => {
            Importers::read_xlsx_sheet(path, &sheet)?.apply_to(&mut state)?;
            Ok((CommandResult::Imported, Some(state)))
        }
    }
}
