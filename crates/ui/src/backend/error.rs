mod data;
mod kind;
mod persistence;

pub use data::ErrorData;
pub use kind::AppErrorKind;
use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum AppError {
    #[error("core error: {0}")]
    Core(#[source] ErrorData),
    #[error("persistence error: {0}")]
    Persistence(#[source] ErrorData),
    #[error("I/O error: {0}")]
    Io(#[source] ErrorData),
    #[error("application worker stopped")]
    WorkerStopped,
}

impl AppError {
    pub fn kind(&self) -> AppErrorKind {
        match self {
            Self::Core(data) | Self::Persistence(data) | Self::Io(data) => data.kind,
            Self::WorkerStopped => AppErrorKind::Internal,
        }
    }
}

impl From<k580_core::CoreError> for AppError {
    fn from(error: k580_core::CoreError) -> Self {
        let kind = match &error {
            k580_core::CoreError::Decode(k580_core::DecodeError::UndocumentedOpcode(_)) => {
                AppErrorKind::UndocumentedOpcode
            }
            k580_core::CoreError::Decode(_) => AppErrorKind::Generic,
            k580_core::CoreError::Port(error) => AppErrorKind::port(error),
        };
        Self::Core(ErrorData::capture(kind, error))
    }
}

impl From<k580_core::PortError> for AppError {
    fn from(error: k580_core::PortError) -> Self {
        Self::Core(ErrorData::capture(AppErrorKind::port(&error), error))
    }
}

impl From<k580_core::ValidationError> for AppError {
    fn from(error: k580_core::ValidationError) -> Self {
        Self::Core(ErrorData::capture(AppErrorKind::AddressRange, error))
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(ErrorData::capture(
            AppErrorKind::io(&error, AppErrorKind::Io),
            error,
        ))
    }
}

impl From<image::ImageError> for AppError {
    fn from(error: image::ImageError) -> Self {
        let kind = match &error {
            image::ImageError::IoError(error) => AppErrorKind::io(error, AppErrorKind::Io),
            _ => AppErrorKind::Generic,
        };
        Self::Io(ErrorData::capture(kind, error))
    }
}

impl From<crate::devices::DeviceError> for AppError {
    fn from(error: crate::devices::DeviceError) -> Self {
        use crate::devices::DeviceError;
        let kind = match &error {
            DeviceError::NotReady => AppErrorKind::DeviceNotReady,
            DeviceError::Busy => AppErrorKind::DeviceBusy,
            DeviceError::PathNotFound(_) => AppErrorKind::NotFound,
            DeviceError::PermissionDenied(_) => AppErrorKind::PermissionDenied,
            DeviceError::Io(_) => AppErrorKind::Io,
            _ => AppErrorKind::Generic,
        };
        Self::Io(ErrorData::capture(kind, error))
    }
}
