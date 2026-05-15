//! Flat 64 KiB RAM owned by the core.

use serde::{Deserialize, Serialize};

/// Total RAM size: 64 KiB, addressed by `u16`.
pub const MEMORY_SIZE: usize = 0x10000;

/// Flat 64 KiB RAM owned by the core. No bank switching, no ROM regions.
#[derive(Clone, Serialize, Deserialize)]
pub struct Memory64K {
    #[serde(with = "serde_bytes_array")]
    bytes: Box<[u8; MEMORY_SIZE]>,
}

impl Memory64K {
    /// Build a zeroed 64 KiB RAM.
    #[must_use]
    pub fn new() -> Self {
        Self {
            bytes: Box::new([0u8; MEMORY_SIZE]),
        }
    }

    /// Read a single byte.
    #[inline]
    #[must_use]
    pub fn read(&self, addr: u16) -> u8 {
        self.bytes[addr as usize]
    }

    /// Write a single byte.
    #[inline]
    pub fn write(&mut self, addr: u16, value: u8) {
        self.bytes[addr as usize] = value;
    }

    /// Read a little-endian word.
    #[inline]
    #[must_use]
    pub fn read_word_le(&self, addr: u16) -> u16 {
        let lo = self.read(addr) as u16;
        let hi = self.read(addr.wrapping_add(1)) as u16;
        (hi << 8) | lo
    }

    /// Write a little-endian word.
    #[inline]
    pub fn write_word_le(&mut self, addr: u16, value: u16) {
        self.write(addr, value as u8);
        self.write(addr.wrapping_add(1), (value >> 8) as u8);
    }

    /// Borrow the entire RAM as a slice.
    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        &self.bytes[..]
    }

    /// Mutable view of the entire RAM.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.bytes[..]
    }

    /// Replace RAM contents from a slice. Returns `Err` if length differs.
    pub fn replace_from(&mut self, src: &[u8]) -> Result<(), MemoryError> {
        if src.len() != MEMORY_SIZE {
            return Err(MemoryError::SizeMismatch {
                expected: MEMORY_SIZE,
                actual: src.len(),
            });
        }
        self.bytes.copy_from_slice(src);
        Ok(())
    }

    /// Zero the entire RAM.
    pub fn clear(&mut self) {
        self.bytes.fill(0);
    }

    /// Load `data` at `start` (wrapping is rejected; out-of-range is rejected).
    pub fn load_at(&mut self, start: u16, data: &[u8]) -> Result<(), MemoryError> {
        let start = start as usize;
        let end = start
            .checked_add(data.len())
            .ok_or(MemoryError::OutOfRange {
                start,
                len: data.len(),
            })?;
        if end > MEMORY_SIZE {
            return Err(MemoryError::OutOfRange {
                start,
                len: data.len(),
            });
        }
        self.bytes[start..end].copy_from_slice(data);
        Ok(())
    }
}

impl Default for Memory64K {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Memory64K {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Memory64K")
            .field("size", &MEMORY_SIZE)
            .finish()
    }
}

/// Memory operation errors.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum MemoryError {
    /// Slice size does not match RAM size.
    #[error("memory size mismatch: expected {expected}, got {actual}")]
    SizeMismatch {
        /// Expected size.
        expected: usize,
        /// Actual size.
        actual: usize,
    },
    /// Range out of bounds.
    #[error("memory range out of bounds: start={start} len={len}")]
    OutOfRange {
        /// Start address.
        start: usize,
        /// Length.
        len: usize,
    },
}

mod serde_bytes_array {
    use super::MEMORY_SIZE;
    use serde::{Deserialize, Deserializer, Serializer};

    #[allow(clippy::borrowed_box)]
    pub fn serialize<S: Serializer>(b: &Box<[u8; MEMORY_SIZE]>, s: S) -> Result<S::Ok, S::Error> {
        serde::Serialize::serialize(&b[..], s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        d: D,
    ) -> Result<Box<[u8; MEMORY_SIZE]>, D::Error> {
        let v: Vec<u8> = Vec::deserialize(d)?;
        if v.len() != MEMORY_SIZE {
            return Err(serde::de::Error::custom(format!(
                "expected {MEMORY_SIZE} bytes, got {}",
                v.len()
            )));
        }
        let mut arr = Box::new([0u8; MEMORY_SIZE]);
        arr.copy_from_slice(&v);
        Ok(arr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_write_byte() {
        let mut m = Memory64K::new();
        m.write(0x1234, 0xAB);
        assert_eq!(m.read(0x1234), 0xAB);
    }

    #[test]
    fn read_write_word_le() {
        let mut m = Memory64K::new();
        m.write_word_le(0x0100, 0xBEEF);
        assert_eq!(m.read(0x0100), 0xEF);
        assert_eq!(m.read(0x0101), 0xBE);
        assert_eq!(m.read_word_le(0x0100), 0xBEEF);
    }

    #[test]
    fn word_at_top_wraps() {
        let mut m = Memory64K::new();
        m.write_word_le(0xFFFF, 0xABCD);
        assert_eq!(m.read(0xFFFF), 0xCD);
        assert_eq!(m.read(0x0000), 0xAB);
    }

    #[test]
    fn replace_from_size_check() {
        let mut m = Memory64K::new();
        let bad = vec![0u8; 10];
        let err = m.replace_from(&bad).unwrap_err();
        assert!(matches!(err, MemoryError::SizeMismatch { .. }));
    }

    #[test]
    fn load_at_rejects_overflow() {
        let mut m = Memory64K::new();
        let err = m.load_at(0xFFFE, &[1, 2, 3, 4]).unwrap_err();
        assert!(matches!(err, MemoryError::OutOfRange { .. }));
    }
}
