//! Typed persistence errors.

use thiserror::Error;

/// Top-level persistence error.
#[derive(Debug, Error)]
pub enum PersistenceError {
    /// Snapshot read / write failure.
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),
    /// Settings JSON parse / write failure.
    #[error("settings error: {0}")]
    Settings(String),
    /// Underlying I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// Export error.
    #[error(transparent)]
    Export(#[from] ExportError),
}

/// Errors raised when reading or writing `.580` snapshots.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SnapshotError {
    /// Magic bytes do not match `K580`.
    #[error("bad magic: expected K580")]
    BadMagic,
    /// Version is not supported by this build.
    #[error("unsupported snapshot version {0}")]
    UnsupportedVersion(u16),
    /// Payload truncated.
    #[error("payload truncated: expected {expected} bytes, got {actual}")]
    Truncated {
        /// Expected.
        expected: usize,
        /// Actual.
        actual: usize,
    },
    /// Required TLV tag missing from the payload.
    #[error("missing required TLV tag {0:#04X}")]
    MissingTag(u8),
    /// Unknown low-bit TLV tag (must fail).
    #[error("unsupported TLV tag {0:#04X}")]
    UnsupportedTag(u8),
    /// TLV value of unexpected length.
    #[error("invalid TLV value length for tag {tag:#04X}: {len}")]
    InvalidLength {
        /// Tag.
        tag: u8,
        /// Length seen.
        len: u32,
    },
    /// Wrap underlying memory layout error.
    #[error("memory size mismatch")]
    MemorySize,
}

/// Export errors.
#[derive(Debug, Error)]
pub enum ExportError {
    /// I/O error during export.
    #[error("export I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// Encoding error (e.g. UTF-8 issue).
    #[error("encoding error: {0}")]
    Encoding(String),
}
