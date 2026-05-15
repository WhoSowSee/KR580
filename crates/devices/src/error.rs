use k580_core::PortError;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DeviceError {
    #[error("device is not ready")]
    NotReady,
    #[error("device is busy")]
    Busy,
    #[error("device operation timed out")]
    Timeout,
    #[error("device is disconnected")]
    Disconnected,
    #[error("path not found: {0}")]
    PathNotFound(String),
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    #[error("I/O error: {0}")]
    Io(String),
    #[error("protocol error: {0}")]
    Protocol(String),
}

impl From<DeviceError> for PortError {
    fn from(value: DeviceError) -> Self {
        match value {
            DeviceError::NotReady => Self::NotReady,
            DeviceError::Busy => Self::Busy,
            DeviceError::Timeout => Self::Timeout,
            DeviceError::Disconnected => Self::Disconnected,
            DeviceError::PathNotFound(path) => Self::PathNotFound(path),
            DeviceError::PermissionDenied(path) => Self::PermissionDenied(path),
            DeviceError::Io(err) => Self::Io(err),
            DeviceError::Protocol(err) => Self::Protocol(err),
        }
    }
}

impl From<std::io::Error> for DeviceError {
    fn from(value: std::io::Error) -> Self {
        use std::io::ErrorKind;
        match value.kind() {
            ErrorKind::NotFound => Self::PathNotFound(value.to_string()),
            ErrorKind::PermissionDenied => Self::PermissionDenied(value.to_string()),
            ErrorKind::TimedOut => Self::Timeout,
            ErrorKind::NotConnected => Self::Disconnected,
            _ => Self::Io(value.to_string()),
        }
    }
}
