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
    /// Reference ("legacy") `.580` files produced by the original
    /// emulator the project was based on are exactly 65 549 bytes:
    /// 65 536 bytes of raw RAM followed by a 13-byte trailer ending
    /// in `FF FF`. Anything else cannot be the legacy format, and
    /// the loader bails before touching memory so a stray import of
    /// a wrong file does not silently overwrite the user's RAM.
    #[error("legacy .580 file must be exactly 65549 bytes, got {0}")]
    InvalidLegacyLength(usize),
    /// Last two bytes of the legacy trailer were not the expected
    /// `FF FF` end-of-record marker. Distinct from a length mismatch
    /// so the user gets a clear hint that the file *looks*
    /// legacy-shaped but the trailer is wrong — most likely a
    /// different binary format that just happens to be 65 549 bytes,
    /// or a corrupted save.
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
