use thiserror::Error;

#[derive(Debug, Error)]
pub enum SnapshotError {
    #[error("snapshot data is truncated")]
    Truncated,
    #[error("invalid .580 magic header")]
    InvalidMagic,
    #[error("unsupported .580 version {0}")]
    UnsupportedVersion(u16),
    #[error("payload length does not match the header")]
    PayloadLengthMismatch,
    #[error("unsupported snapshot TLV tag {0:#04X}")]
    UnsupportedTag(u8),
    #[error("invalid length {length} for tag {tag:#04X}")]
    InvalidLength { tag: u8, length: usize },
    #[error("required snapshot tag {0:#04X} is missing")]
    MissingTag(u8),
    #[error("legacy .580 file must be exactly 65549 bytes, got {0}")]
    InvalidLegacyLength(usize),
    #[error("legacy .580 trailer is missing the FF FF end marker")]
    InvalidLegacyTrailer,
}

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
    Snapshot(#[from] SnapshotError),
    #[error(transparent)]
    Settings(#[from] SettingsError),
    #[error(transparent)]
    Export(#[from] ExportError),
    #[error(transparent)]
    Import(#[from] ImportError),
    #[error("subprogram I/O error: {0}")]
    SubprogramIo(#[from] std::io::Error),
}
