#[derive(Clone, PartialEq, Eq)]
pub struct Memory64K {
    bytes: Box<[u8; Self::SIZE]>,
}

impl Default for Memory64K {
    fn default() -> Self {
        Self {
            bytes: Box::new([0; Self::SIZE]),
        }
    }
}

impl core::fmt::Debug for Memory64K {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Memory64K")
            .field("size", &Self::SIZE)
            .finish()
    }
}

impl Memory64K {
    pub const SIZE: usize = 65_536;

    pub fn read(&self, address: u16) -> u8 {
        self.bytes[address as usize]
    }

    pub fn write(&mut self, address: u16, value: u8) {
        self.bytes[address as usize] = value;
    }

    pub fn read_word(&self, address: u16) -> u16 {
        let lo = self.read(address) as u16;
        let hi = self.read(address.wrapping_add(1)) as u16;
        lo | (hi << 8)
    }

    pub fn write_word(&mut self, address: u16, value: u16) {
        self.write(address, value as u8);
        self.write(address.wrapping_add(1), (value >> 8) as u8);
    }

    pub fn as_slice(&self) -> &[u8] {
        self.bytes.as_slice()
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        self.bytes.as_mut_slice()
    }

    pub fn clear(&mut self) {
        self.bytes.fill(0);
    }
}
