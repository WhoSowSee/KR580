use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum AppError {
    #[error("core error: {0}")]
    Core(String),
    #[error("persistence error: {0}")]
    Persistence(String),
    #[error("I/O error: {0}")]
    Io(String),
    #[error("application worker stopped")]
    WorkerStopped,
}

impl From<k580_core::CoreError> for AppError {
    fn from(value: k580_core::CoreError) -> Self {
        Self::Core(value.to_string())
    }
}

impl From<k580_persistence::PersistenceError> for AppError {
    fn from(value: k580_persistence::PersistenceError) -> Self {
        Self::Persistence(value.to_string())
    }
}

impl From<k580_persistence::ProgramError> for AppError {
    fn from(value: k580_persistence::ProgramError) -> Self {
        Self::Persistence(value.to_string())
    }
}

impl From<k580_persistence::ExportError> for AppError {
    fn from(value: k580_persistence::ExportError) -> Self {
        Self::Persistence(value.to_string())
    }
}

impl From<k580_persistence::ImportError> for AppError {
    fn from(value: k580_persistence::ImportError) -> Self {
        Self::Persistence(value.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

impl From<k580_core::PortError> for AppError {
    fn from(value: k580_core::PortError) -> Self {
        Self::Core(value.to_string())
    }
}

impl From<k580_core::ValidationError> for AppError {
    fn from(value: k580_core::ValidationError) -> Self {
        Self::Core(value.to_string())
    }
}
