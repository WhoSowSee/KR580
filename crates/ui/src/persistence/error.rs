use crate::persistence::program::ProgramError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("settings I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("settings JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unsupported settings version {0}")]
    UnsupportedVersion(u32),
}

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("export I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("spreadsheet export error: {0}")]
    Spreadsheet(String),
}

#[derive(Debug, Error)]
pub enum ImportError {
    #[error("import I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("spreadsheet import error: {0}")]
    Spreadsheet(String),
    #[error("malformed import file: {0}")]
    Malformed(String),
}

#[derive(Debug, Error)]
pub enum PersistenceError {
    #[error(transparent)]
    Program(#[from] ProgramError),
    #[error(transparent)]
    Settings(#[from] SettingsError),
    #[error(transparent)]
    Export(#[from] ExportError),
    #[error(transparent)]
    Import(#[from] ImportError),
}
