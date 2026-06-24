use k580_core::{Cpu8080State, Memory64K};

pub const LEGACY_LENGTH: usize = Memory64K::SIZE + 13;

#[derive(Debug)]
pub enum ProgramError {
    NotA580File,
    EmptyFile,
    WrongSize { size: usize },
    InvalidLegacyTrailer,
    Io(std::io::Error),
}

impl std::fmt::Display for ProgramError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProgramError::NotA580File => write!(f, "not a .580 file"),
            ProgramError::EmptyFile => write!(f, "file is empty"),
            ProgramError::WrongSize { size } => {
                write!(f, "expected {LEGACY_LENGTH} bytes, got {size}")
            }
            ProgramError::InvalidLegacyTrailer => {
                write!(f, "legacy .580 trailer is missing the FF FF end marker")
            }
            ProgramError::Io(err) => write!(f, "I/O error: {err}"),
        }
    }
}

impl std::error::Error for ProgramError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ProgramError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ProgramError {
    fn from(err: std::io::Error) -> Self {
        ProgramError::Io(err)
    }
}

pub struct ProgramSerializer;

impl ProgramSerializer {
    pub fn save_file(
        path: impl AsRef<std::path::Path>,
        state: &Cpu8080State,
    ) -> Result<(), ProgramError> {
        let mut out = Vec::with_capacity(LEGACY_LENGTH);
        out.extend_from_slice(state.memory.as_slice());
        out.resize(out.len() + 9, 0);
        out.extend_from_slice(&state.pc.to_le_bytes());
        out.push(0xFF);
        out.push(0xFF);
        std::fs::write(path, out)?;
        Ok(())
    }

    pub fn load_file(path: impl AsRef<std::path::Path>) -> Result<Cpu8080State, ProgramError> {
        validate_path(path.as_ref())?;
        let bytes = std::fs::read(path)?;
        if bytes.is_empty() {
            return Err(ProgramError::EmptyFile);
        }
        if bytes.len() != LEGACY_LENGTH {
            return Err(ProgramError::WrongSize { size: bytes.len() });
        }
        Self::from_legacy_bytes(&bytes)
    }

    fn from_legacy_bytes(bytes: &[u8]) -> Result<Cpu8080State, ProgramError> {
        let trailer_start = Memory64K::SIZE;
        let trailer = &bytes[trailer_start..];
        if trailer[11] != 0xFF || trailer[12] != 0xFF {
            return Err(ProgramError::InvalidLegacyTrailer);
        }
        let mut state = Cpu8080State::default();
        state
            .memory
            .as_mut_slice()
            .copy_from_slice(&bytes[..trailer_start]);
        state.pc = u16::from_le_bytes([trailer[9], trailer[10]]);
        Ok(state)
    }
}

fn validate_path(path: &std::path::Path) -> Result<(), ProgramError> {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("580") => Ok(()),
        _ => Err(ProgramError::NotA580File),
    }
}
