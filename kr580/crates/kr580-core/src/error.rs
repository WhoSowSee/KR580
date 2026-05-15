//! Typed errors raised by the core.

use thiserror::Error;

/// Top-level error for the CPU core.
#[derive(Debug, Error)]
pub enum CoreError {
    /// Failed to decode the byte at `pc`.
    #[error("decode error at PC={pc:#06X}: {source}")]
    Decode {
        /// Program counter at which decode failed.
        pc: u16,
        /// Underlying decode error.
        #[source]
        source: DecodeError,
    },

    /// Validation error from a command (out-of-range value, bad register, …).
    #[error("validation error: {0}")]
    Validation(String),

    /// Halt was hit while a forward-progress operation expected work to be done.
    #[error("CPU halted")]
    Halted,
}

/// Error raised by the opcode decoder.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DecodeError {
    /// Undocumented 8080 opcode slot. Per `prompt/09_quality_gates.md`, these
    /// must raise an error and stop execution; they are NOT NOP / JMP / CALL /
    /// RET aliases.
    #[error("undocumented 8080 opcode {0:#04X}")]
    UndocumentedOpcode(u8),
}
