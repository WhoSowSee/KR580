//! 8080 PSW flag model.
//!
//! Layout when packed into a PSW byte (per `prompt/04_file_formats.md`):
//!
//! ```text
//! bit:  7  6  5  4  3  2  1  0
//!       S  Z  0  AC 0  P  1  CY
//! ```
//!
//! Bits 3 and 5 are always zero on store. Bit 1 is always one on store.
//! On `POP PSW`, bits 3 and 5 are ignored and bit 1 is forced to 1.

use serde::{Deserialize, Serialize};

/// Plain typed flag set.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Flags {
    /// Sign flag (bit 7 of the result).
    pub s: bool,
    /// Zero flag (result == 0).
    pub z: bool,
    /// Auxiliary carry (carry out of bit 3).
    pub ac: bool,
    /// Parity flag (even parity of low 8 bits).
    pub p: bool,
    /// Carry flag.
    pub cy: bool,
}

impl Flags {
    /// Pack this flag set into a PSW byte using the canonical 8080 layout.
    #[inline]
    pub const fn to_psw_byte(self) -> u8 {
        let mut b: u8 = 0b0000_0010; // bit 1 is always 1
        if self.s {
            b |= 0b1000_0000;
        }
        if self.z {
            b |= 0b0100_0000;
        }
        if self.ac {
            b |= 0b0001_0000;
        }
        if self.p {
            b |= 0b0000_0100;
        }
        if self.cy {
            b |= 0b0000_0001;
        }
        b
    }

    /// Restore flags from a PSW byte. Bits 3 and 5 are ignored. Bit 1 is
    /// forced to 1 on output via [`Self::to_psw_byte`].
    #[inline]
    pub const fn from_psw_byte(b: u8) -> Self {
        Flags {
            s: b & 0b1000_0000 != 0,
            z: b & 0b0100_0000 != 0,
            ac: b & 0b0001_0000 != 0,
            p: b & 0b0000_0100 != 0,
            cy: b & 0b0000_0001 != 0,
        }
    }

    /// Set Sign / Zero / Parity from a result byte. Does NOT touch CY or AC.
    #[inline]
    pub fn set_szp(&mut self, result: u8) {
        self.s = (result & 0x80) != 0;
        self.z = result == 0;
        self.p = parity_even(result);
    }
}

/// Even parity of the low 8 bits of `b`.
#[inline]
pub const fn parity_even(b: u8) -> bool {
    (b.count_ones() & 1) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn psw_byte_layout_bit1_always_one() {
        let f = Flags::default();
        assert_eq!(f.to_psw_byte() & 0b0000_0010, 0b0000_0010);
        assert_eq!(f.to_psw_byte() & 0b0010_1000, 0); // bits 3 and 5 zero
    }

    #[test]
    fn psw_roundtrip_ignores_bits_3_and_5() {
        // Set every bit, including the "always 0" bits 3 and 5.
        let raw = 0b1111_1111u8;
        let f = Flags::from_psw_byte(raw);
        let again = f.to_psw_byte();
        // bits 3 and 5 are masked off, bit 1 forced to 1.
        assert_eq!(again, 0b1101_0111);
    }

    #[test]
    fn parity_even_known_values() {
        assert!(parity_even(0x00));
        assert!(parity_even(0x03));
        assert!(!parity_even(0x01));
        assert!(parity_even(0xFF));
    }

    #[test]
    fn set_szp_does_not_touch_cy_ac() {
        let mut f = Flags {
            s: false,
            z: false,
            ac: true,
            p: false,
            cy: true,
        };
        f.set_szp(0x00);
        assert!(f.z);
        assert!(!f.s);
        assert!(f.p);
        assert!(f.ac, "AC must not be touched by set_szp");
        assert!(f.cy, "CY must not be touched by set_szp");

        f.set_szp(0x80);
        assert!(f.s);
        assert!(!f.z);
        assert!(!f.p);
    }
}
