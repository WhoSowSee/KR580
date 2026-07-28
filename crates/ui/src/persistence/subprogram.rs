use k580_core::Cpu8080State;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SubprogramError {
    #[error("not a .krs file")]
    NotAKrsFile,
    #[error("subprogram file is empty")]
    EmptyFile,
    #[error("subprogram range is invalid: {start:#06X}..{end:#06X}")]
    InvalidRange { start: u16, end: u16 },
    #[error("subprogram of {length} bytes does not fit at {start:#06X}")]
    MemoryOverflow { start: u16, length: u64 },
    #[error("subprogram I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct SubprogramSerializer;

impl SubprogramSerializer {
    pub fn supports_path(path: impl AsRef<Path>) -> bool {
        path.as_ref()
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("krs"))
    }

    pub fn load_into_state(
        path: impl AsRef<Path>,
        start: u16,
        state: &mut Cpu8080State,
    ) -> Result<(), SubprogramError> {
        validate_path(path.as_ref())?;
        let bytes = std::fs::read(path)?;
        checked_load_end(start, bytes.len() as u64)?;
        state
            .set_memory_block(start, &bytes)
            .map_err(|_| SubprogramError::MemoryOverflow {
                start,
                length: bytes.len() as u64,
            })?;
        Ok(())
    }

    pub fn file_end(path: impl AsRef<Path>, start: u16) -> Result<u16, SubprogramError> {
        validate_path(path.as_ref())?;
        checked_load_end(start, std::fs::metadata(path)?.len())
    }

    pub fn save_file(
        path: impl AsRef<Path>,
        state: &Cpu8080State,
        start: u16,
        end: u16,
    ) -> Result<(), SubprogramError> {
        validate_path(path.as_ref())?;
        if start > end {
            return Err(SubprogramError::InvalidRange { start, end });
        }
        let bytes = &state.memory.as_slice()[usize::from(start)..=usize::from(end)];
        std::fs::write(path, bytes)?;
        Ok(())
    }
}

fn checked_load_end(start: u16, length: u64) -> Result<u16, SubprogramError> {
    if length == 0 {
        return Err(SubprogramError::EmptyFile);
    }
    let end = u64::from(start)
        .checked_add(length - 1)
        .ok_or(SubprogramError::MemoryOverflow { start, length })?;
    u16::try_from(end).map_err(|_| SubprogramError::MemoryOverflow { start, length })
}

fn validate_path(path: &Path) -> Result<(), SubprogramError> {
    if SubprogramSerializer::supports_path(path) {
        Ok(())
    } else {
        Err(SubprogramError::NotAKrsFile)
    }
}

#[cfg(test)]
mod tests {
    use super::{SubprogramError, SubprogramSerializer};
    use k580_core::Cpu8080State;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(name: &str, extension: &str) -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("kr580-{name}-{stamp}.{extension}"))
    }

    #[test]
    fn loads_raw_bytes_at_requested_address() {
        let path = temp_path("subprogram-load", "krs");
        fs::write(&path, [0x3E, 0x42, 0x76]).unwrap();
        let mut state = Cpu8080State::default();

        SubprogramSerializer::load_into_state(&path, 0x0100, &mut state).unwrap();

        assert_eq!(
            &state.memory.as_slice()[0x0100..=0x0102],
            &[0x3E, 0x42, 0x76]
        );
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn rejects_subprogram_that_overflows_memory() {
        let path = temp_path("subprogram-overflow", "krs");
        fs::write(&path, [0x00, 0x01]).unwrap();
        let mut state = Cpu8080State::default();

        let error = SubprogramSerializer::load_into_state(&path, 0xFFFF, &mut state).unwrap_err();

        assert!(matches!(
            error,
            SubprogramError::MemoryOverflow {
                start: 0xFFFF,
                length: 2
            }
        ));
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn reports_inclusive_file_end() {
        let path = temp_path("subprogram-end", "krs");
        fs::write(&path, [0x00, 0x01, 0x02]).unwrap();

        assert_eq!(
            SubprogramSerializer::file_end(&path, 0x0100).unwrap(),
            0x0102
        );
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn saves_inclusive_memory_range_as_raw_bytes() {
        let path = temp_path("subprogram-save", "krs");
        let mut state = Cpu8080State::default();
        state.memory.as_mut_slice()[0x0200..=0x0202].copy_from_slice(&[0xC3, 0x00, 0x02]);

        SubprogramSerializer::save_file(&path, &state, 0x0200, 0x0202).unwrap();

        assert_eq!(fs::read(&path).unwrap(), [0xC3, 0x00, 0x02]);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn rejects_non_krs_path() {
        let path = temp_path("subprogram-extension", "bin");
        fs::write(&path, [0x00]).unwrap();

        let error = SubprogramSerializer::load_into_state(&path, 0, &mut Cpu8080State::default())
            .unwrap_err();

        assert!(matches!(error, SubprogramError::NotAKrsFile));
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn recognizes_krs_extension_case_insensitively() {
        assert!(SubprogramSerializer::supports_path("program.krs"));
        assert!(SubprogramSerializer::supports_path("program.KRS"));
        assert!(!SubprogramSerializer::supports_path("program.580"));
    }
}
