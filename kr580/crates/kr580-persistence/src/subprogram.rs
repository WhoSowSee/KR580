//! `.krs` subprogram format.
//!
//! Per `prompt/04_file_formats.md`: a raw memory slice plus an explicit
//! caller-provided base address. No GUI state, no hidden metadata. The base
//! address is supplied by the caller / dialog at load time.
//!
//! On disk the format is a tiny header so we can recover the canonical length
//! and base address that the user chose at save time:
//!
//! ```text
//! magic    : "KRS1" (4 bytes)
//! base     : u16 little-endian
//! length   : u32 little-endian
//! payload  : `length` raw bytes
//! ```
//!
//! `SubprogramSerializer` is the *only* `.krs` reader / writer.

use crate::error::PersistenceError;

/// Magic header for `.krs` files.
pub const MAGIC: &[u8; 4] = b"KRS1";

/// In-memory representation of a `.krs` subprogram.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubprogramFile {
    /// Base address at which to install the bytes.
    pub base: u16,
    /// Raw bytes (length implicit).
    pub bytes: Vec<u8>,
}

/// Reader / writer for `.krs` files.
pub struct SubprogramSerializer;

impl SubprogramSerializer {
    /// Build a versioned `.krs` byte stream.
    pub fn save(file: &SubprogramFile) -> Vec<u8> {
        let mut out = Vec::with_capacity(file.bytes.len() + 10);
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&file.base.to_le_bytes());
        out.extend_from_slice(&(file.bytes.len() as u32).to_le_bytes());
        out.extend_from_slice(&file.bytes);
        out
    }

    /// Parse a `.krs` byte stream.
    pub fn load(bytes: &[u8]) -> Result<SubprogramFile, PersistenceError> {
        if bytes.len() < 10 {
            return Err(PersistenceError::Settings(
                "krs: header truncated".to_string(),
            ));
        }
        if &bytes[0..4] != MAGIC {
            return Err(PersistenceError::Settings("krs: bad magic".to_string()));
        }
        let base = u16::from_le_bytes([bytes[4], bytes[5]]);
        let len = u32::from_le_bytes([bytes[6], bytes[7], bytes[8], bytes[9]]) as usize;
        let payload = &bytes[10..];
        if payload.len() < len {
            return Err(PersistenceError::Settings(
                "krs: payload truncated".to_string(),
            ));
        }
        Ok(SubprogramFile {
            base,
            bytes: payload[..len].to_vec(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let file = SubprogramFile {
            base: 0x1000,
            bytes: vec![0x3E, 0x42, 0x76], // MVI A,0x42; HLT
        };
        let bytes = SubprogramSerializer::save(&file);
        let back = SubprogramSerializer::load(&bytes).unwrap();
        assert_eq!(back, file);
    }

    #[test]
    fn empty_payload_ok() {
        let file = SubprogramFile {
            base: 0,
            bytes: vec![],
        };
        let bytes = SubprogramSerializer::save(&file);
        let back = SubprogramSerializer::load(&bytes).unwrap();
        assert_eq!(back, file);
    }
}
