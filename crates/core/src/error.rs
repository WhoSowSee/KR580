use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ValidationError {
    #[error("address range {start:#06X}..{end:#06X} is outside 64 KiB memory")]
    MemoryRange { start: u16, end: u32 },
    #[error("invalid register name: {0}")]
    InvalidRegister(String),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DecodeError {
    #[error("undocumented opcode {0:#04X}")]
    UndocumentedOpcode(u8),
    #[error("invalid interrupt vector opcode {0:#04X}")]
    InvalidInterruptVector(u8),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PortError {
    #[error("invalid I/O port {0:#04X}")]
    InvalidPort(u8),
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

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CoreError {
    #[error(transparent)]
    Decode(#[from] DecodeError),
    #[error(transparent)]
    Port(#[from] PortError),
    #[error(transparent)]
    Validation(#[from] ValidationError),
}
