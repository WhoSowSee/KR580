use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Flags {
    pub sign: bool,
    pub zero: bool,
    pub auxiliary_carry: bool,
    pub parity: bool,
    pub carry: bool,
}

impl Flags {
    pub const PSW_ALWAYS_SET: u8 = 0b0000_0010;

    pub fn from_psw(byte: u8) -> Self {
        Self {
            sign: byte & 0x80 != 0,
            zero: byte & 0x40 != 0,
            auxiliary_carry: byte & 0x10 != 0,
            parity: byte & 0x04 != 0,
            carry: byte & 0x01 != 0,
        }
    }

    pub fn to_psw(self) -> u8 {
        (u8::from(self.sign) << 7)
            | (u8::from(self.zero) << 6)
            | (u8::from(self.auxiliary_carry) << 4)
            | (u8::from(self.parity) << 2)
            | Self::PSW_ALWAYS_SET
            | u8::from(self.carry)
    }

    pub fn set_sign_zero_parity(&mut self, value: u8) {
        self.sign = value & 0x80 != 0;
        self.zero = value == 0;
        self.parity = value.count_ones().is_multiple_of(2);
    }
}
