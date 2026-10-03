use super::{AppError, AppErrorKind as Kind, ErrorData};
use crate::persistence::{
    ExportError, ImportError, PersistenceError, ProgramError, SettingsError, SubprogramError,
};

impl From<PersistenceError> for AppError {
    fn from(error: PersistenceError) -> Self {
        match error {
            PersistenceError::Program(error) => error.into(),
            PersistenceError::Subprogram(error) => error.into(),
            PersistenceError::Settings(error) => error.into(),
            PersistenceError::Export(error) => error.into(),
            PersistenceError::Import(error) => error.into(),
        }
    }
}

impl From<ProgramError> for AppError {
    fn from(error: ProgramError) -> Self {
        let kind = match &error {
            ProgramError::NotA580File => Kind::ProgramExtension,
            ProgramError::EmptyFile => Kind::EmptyFile,
            ProgramError::WrongSize { .. } => Kind::ProgramSize,
            ProgramError::Io(error) => Kind::io(error, Kind::Io),
        };
        Self::Persistence(ErrorData::capture(kind, error))
    }
}

impl From<SubprogramError> for AppError {
    fn from(error: SubprogramError) -> Self {
        let kind = match &error {
            SubprogramError::NotAKrsFile => Kind::SubprogramExtension,
            SubprogramError::EmptyFile => Kind::EmptyFile,
            SubprogramError::InvalidRange { .. } | SubprogramError::MemoryOverflow { .. } => {
                Kind::AddressRange
            }
            SubprogramError::Io(error) => Kind::io(error, Kind::Io),
        };
        Self::Persistence(ErrorData::capture(kind, error))
    }
}

impl From<ExportError> for AppError {
    fn from(error: ExportError) -> Self {
        let kind = match &error {
            ExportError::Io(error) => Kind::io(error, Kind::WriteFile),
            ExportError::Spreadsheet(_) => Kind::ExportFormat,
        };
        Self::Persistence(ErrorData::capture(kind, error))
    }
}

impl From<ImportError> for AppError {
    fn from(error: ImportError) -> Self {
        let kind = match &error {
            ImportError::Io(error) => Kind::io(error, Kind::ReadFile),
            ImportError::Malformed(_) | ImportError::Spreadsheet(_) => Kind::ImportFormat,
        };
        Self::Persistence(ErrorData::capture(kind, error))
    }
}

impl From<SettingsError> for AppError {
    fn from(error: SettingsError) -> Self {
        let kind = match &error {
            SettingsError::Io(error) => Kind::io(error, Kind::Io),
            SettingsError::Json(_) => Kind::SettingsJson,
            SettingsError::UnsupportedVersion(_) => Kind::SettingsVersion,
        };
        Self::Persistence(ErrorData::capture(kind, error))
    }
}
