//! Typed device errors.

use thiserror::Error;

/// Device-side error. The same variant set is used by every device, surfaced
/// through `DeviceStatus` rather than via panics.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DeviceError {
    /// Device has not been configured yet.
    #[error("device not ready")]
    NotReady,
    /// Device is busy with another command.
    #[error("device busy")]
    Busy,
    /// Operation timed out.
    #[error("device timeout")]
    Timeout,
    /// Network or storage handle was disconnected.
    #[error("device disconnected")]
    Disconnected,
    /// Configured path could not be opened.
    #[error("path not found: {0}")]
    PathNotFound(String),
    /// Permission denied on the underlying handle.
    #[error("permission denied")]
    PermissionDenied,
    /// Generic I/O error from the host.
    #[error("I/O error: {0}")]
    Io(String),
    /// Network protocol error.
    #[error("protocol error: {0}")]
    Protocol(String),
}
