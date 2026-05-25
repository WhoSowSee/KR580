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

/// Programmer-visible register file plus the internal `W`/`Z` scratch
/// pair the 8080 microcode parks intermediate addresses in.
///
/// `W` and `Z` are deliberately NOT exposed through `RegisterName`: no
/// 8080 instruction lets a program read or write them directly, so
/// adding them to the enum would mis-advertise the architecture and
/// open a door (`set_register(RegisterName::W, …)`) that the real CPU
/// does not have. They live on `Registers` purely so the UI can show
/// what the microsequencer last loaded into them — `STA`/`LDA`/`JMP`/
/// `CALL`/`RET`/`LHLD`/`SHLD`/`XCHG`/`XTHL`/`PCHL`/`SPHL`/`LXI` all
/// route their address operand through this pair on the way to its
/// final destination, and the school-grade reference emulator we
/// match against displays that "address residue" alongside the РОН
/// register block. Without modelling W/Z we showed two static `00`
/// chips while the reference showed the live values, which the user
/// flagged as a real divergence.
///
/// The values are write-only from the core's point of view: nothing
/// inside `ops/*` reads `regs.w/z` back, they are pure observation
/// points for the schematic. Keeping it that way means we can never
/// silently start depending on the residue from a previous
/// instruction — every command must (re)compute its operand from
/// memory or registers, exactly as the real chip's microcode does.
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

    /// Records an address operand the microsequencer just loaded into
    /// the internal scratch pair. Stores `hi` in `W`, `lo` in `Z` —
    /// matches the 8080's microcode order: when the CPU fetches a
    /// `STA a16` operand, the low byte arrives first and goes into
    /// `Z`, the high byte arrives next and goes into `W`. The value
    /// then ends up wherever the instruction routes it (here: a
    /// memory write at `WZ`); the `W`/`Z` cells just keep the residue
    /// for the schematic to display.
    pub fn set_wz(&mut self, value: u16) {
        let [hi, lo] = value.to_be_bytes();
        self.w = hi;
        self.z = lo;
    }
}
