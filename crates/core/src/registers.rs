use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegisterName {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

/// Programmer-visible registers plus the internal `W`/`Z` scratch pair.
/// `W`/`Z` are not in `RegisterName` — no 8080 instruction addresses
/// them directly. They sit on `Registers` only so the UI can show the
/// address residue the microcode parks there. Treated as write-only
/// from the core's perspective.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub w: u8,
    pub z: u8,
}

impl Registers {
    pub fn get(&self, name: RegisterName) -> u8 {
        match name {
            RegisterName::A => self.a,
            RegisterName::B => self.b,
            RegisterName::C => self.c,
            RegisterName::D => self.d,
            RegisterName::E => self.e,
            RegisterName::H => self.h,
            RegisterName::L => self.l,
        }
    }

    pub fn set(&mut self, name: RegisterName, value: u8) {
        match name {
            RegisterName::A => self.a = value,
            RegisterName::B => self.b = value,
            RegisterName::C => self.c = value,
            RegisterName::D => self.d = value,
            RegisterName::E => self.e = value,
            RegisterName::H => self.h = value,
            RegisterName::L => self.l = value,
        }
    }

    pub fn bc(&self) -> u16 {
        u16::from_be_bytes([self.b, self.c])
    }

    pub fn de(&self) -> u16 {
        u16::from_be_bytes([self.d, self.e])
    }

    pub fn hl(&self) -> u16 {
        u16::from_be_bytes([self.h, self.l])
    }

    pub fn set_bc(&mut self, value: u16) {
        let [hi, lo] = value.to_be_bytes();
        self.b = hi;
        self.c = lo;
    }

    pub fn set_de(&mut self, value: u16) {
        let [hi, lo] = value.to_be_bytes();
        self.d = hi;
        self.e = lo;
    }

    pub fn set_hl(&mut self, value: u16) {
        let [hi, lo] = value.to_be_bytes();
        self.h = hi;
        self.l = lo;
    }

    /// 8080 microcode order: high byte → `W`, low byte → `Z` (low
    /// arrives first into Z, high into W).
    pub fn set_wz(&mut self, value: u16) {
        let [hi, lo] = value.to_be_bytes();
        self.w = hi;
        self.z = lo;
    }
}
