use k580_core::{Cpu8080State, Memory64K};

pub const LEGACY_LENGTH: usize = Memory64K::SIZE + 13;

#[derive(Debug)]
pub enum ProgramError {
    NotA580File,
    EmptyFile,
    WrongSize { size: usize },
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
    pub fn supports_path(path: impl AsRef<std::path::Path>) -> bool {
        path.as_ref()
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("580"))
    }

    pub fn save_file(
        path: impl AsRef<std::path::Path>,
        state: &Cpu8080State,
    ) -> Result<(), ProgramError> {
        let mut out = Vec::with_capacity(LEGACY_LENGTH);
        out.extend_from_slice(state.memory.as_slice());
        out.resize(out.len() + 9, 0);
        out.extend_from_slice(&state.pc.to_le_bytes());
        out.extend_from_slice(&state.sp.to_le_bytes());
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
        Ok(Self::from_legacy_bytes(&bytes))
    }

    fn from_legacy_bytes(bytes: &[u8]) -> Cpu8080State {
        let trailer_start = Memory64K::SIZE;
        let trailer = &bytes[trailer_start..];
        let mut state = Cpu8080State::default();
        state
            .memory
            .as_mut_slice()
            .copy_from_slice(&bytes[..trailer_start]);
        state.pc = u16::from_le_bytes([trailer[9], trailer[10]]);
        state.sp = u16::from_le_bytes([trailer[11], trailer[12]]);
        state
    }
}

fn validate_path(path: &std::path::Path) -> Result<(), ProgramError> {
    if ProgramSerializer::supports_path(path) {
        Ok(())
    } else {
        Err(ProgramError::NotA580File)
    }
}
