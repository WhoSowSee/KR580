use super::IoUpdate;
use crate::backend::{AppError, CommandResult};
use crate::persistence::import::CpuPatch;
use crate::persistence::{
    ExportModel, ExportOptions, Exporters, Importers, ProgramSerializer, SubprogramSerializer,
};
use k580_core::Cpu8080State;
use std::path::PathBuf;

pub(super) enum IoJob {
    AttachStorage {
        kind: crate::devices::StorageKind,
        path: PathBuf,
    },
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
        start: u16,
    },
    ExportTxt {
        path: PathBuf,
        model: ExportModel,
    },
    ExportXlsx {
        path: PathBuf,
        model: ExportModel,
        options: ExportOptions,
    },
    ExportTxtSections {
        path: PathBuf,
        models: Vec<(String, ExportModel)>,
    },
    ExportXlsxPages {
        path: PathBuf,
        models: Vec<(String, ExportModel, ExportOptions)>,
    },
    ImportTxt {
        path: PathBuf,
    },
    ImportTxtSection {
        path: PathBuf,
        section: String,
    },
    ImportXlsx {
        path: PathBuf,
    },
    ImportXlsxSheet {
        path: PathBuf,
        sheet: String,
    },
}

pub(super) fn run(job: IoJob) -> Result<(CommandResult, Option<IoUpdate>), AppError> {
    match job {
        IoJob::AttachStorage { kind, path } => {
            let file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .map_err(crate::devices::DeviceError::from);
            Ok((
                CommandResult::Completed,
                Some(IoUpdate::Storage { kind, path, file }),
            ))
        }
        IoJob::SaveProgram { path, state } => {
            ProgramSerializer::save_file(path, &state)?;
            Ok((CommandResult::SavedProgram { state }, None))
        }
        IoJob::LoadProgram { path } => {
            let state = ProgramSerializer::load_file(path)?;
            Ok((
                CommandResult::LoadedProgram,
                Some(IoUpdate::Program(Box::new(state))),
            ))
        }
        IoJob::SaveSubprogram {
            path,
            state,
            start,
            end,
        } => {
            SubprogramSerializer::save_file(path, &state, start, end)?;
            Ok((CommandResult::SavedSubprogram { state }, None))
        }
        IoJob::LoadSubprogram { path, start } => {
            let (values, end) = SubprogramSerializer::read_block(&path, start)?;
            Ok((
                CommandResult::LoadedSubprogram { end },
                Some(IoUpdate::Subprogram { start, values }),
            ))
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
        IoJob::ImportTxt { path } => imported(Importers::read_txt(path)?),
        IoJob::ImportTxtSection { path, section } => {
            imported(Importers::read_txt_section(path, &section)?)
        }
        IoJob::ImportXlsx { path } => imported(Importers::read_xlsx(path)?),
        IoJob::ImportXlsxSheet { path, sheet } => {
            imported(Importers::read_xlsx_sheet(path, &sheet)?)
        }
    }
}

fn imported(model: ExportModel) -> Result<(CommandResult, Option<IoUpdate>), AppError> {
    Ok((
        CommandResult::Imported,
        Some(IoUpdate::Import(CpuPatch::owned(model)?)),
    ))
}
