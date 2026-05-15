use k580_core::{Cpu8080State, ValidationError};
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Subprogram {
    pub base_address: u16,
    pub bytes: Vec<u8>,
}

pub struct SubprogramSerializer;

impl SubprogramSerializer {
    pub fn to_bytes(subprogram: &Subprogram) -> Vec<u8> {
        subprogram.bytes.clone()
    }

    pub fn from_bytes(base_address: u16, bytes: Vec<u8>) -> Subprogram {
        Subprogram {
            base_address,
            bytes,
        }
    }

    pub fn save_file(path: impl AsRef<Path>, subprogram: &Subprogram) -> std::io::Result<()> {
        std::fs::write(path, Self::to_bytes(subprogram))
    }

    pub fn load_file(path: impl AsRef<Path>, base_address: u16) -> std::io::Result<Subprogram> {
        Ok(Self::from_bytes(base_address, std::fs::read(path)?))
    }

    pub fn load_into_state(
        state: &mut Cpu8080State,
        subprogram: &Subprogram,
    ) -> Result<(), ValidationError> {
        let start = subprogram.base_address as usize;
        let end = start + subprogram.bytes.len();
        if end > k580_core::Memory64K::SIZE {
            return Err(ValidationError::MemoryRange {
                start: subprogram.base_address,
                end: end as u32,
            });
        }
        state.memory.as_mut_slice()[start..end].copy_from_slice(&subprogram.bytes);
        Ok(())
    }
}
