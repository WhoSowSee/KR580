use crate::Cpu8080State;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RegPair {
    BC,
    DE,
    HL,
    SP,
}

impl RegPair {
    pub(crate) fn from_code(code: u8) -> Self {
        match code & 0x03 {
            0 => Self::BC,
            1 => Self::DE,
            2 => Self::HL,
            _ => Self::SP,
        }
    }
}

impl Cpu8080State {
    pub(crate) fn read_reg_code(&mut self, code: u8) -> u8 {
        match code & 0x07 {
            0 => self.registers.b,
            1 => self.registers.c,
            2 => self.registers.d,
            3 => self.registers.e,
            4 => self.registers.h,
            5 => self.registers.l,
            // M-код 110 = `(HL)` — единственная ветка, где регистр-
            // код реально дёргает шину. Раньше `read_reg_code` был
            // `&self`, потому что `Memory64K::read` чист от побочек,
            // но теперь чтение через шину обновляет латчи адреса/
            // данных (`last_address_bus`, `last_data_bus_byte`),
            // и без `&mut self` это не выразить. Все остальные семь
            // регистров — внутренние, шины не касаются, и латчи не
            // должны меняться от MOV r1, r2.
            6 => self.bus_read(self.registers.hl()),
            _ => self.registers.a,
        }
    }

    pub(crate) fn write_reg_code(&mut self, code: u8, value: u8) {
        match code & 0x07 {
            0 => self.registers.b = value,
            1 => self.registers.c = value,
            2 => self.registers.d = value,
            3 => self.registers.e = value,
            4 => self.registers.h = value,
            5 => self.registers.l = value,
            // Симметрично `read_reg_code`: M=`(HL)` → запись через
            // шину с обновлением латчей.
            6 => self.bus_write(self.registers.hl(), value),
            _ => self.registers.a = value,
        }
    }

    pub(crate) fn read_pair(&self, pair: RegPair) -> u16 {
        match pair {
            RegPair::BC => self.registers.bc(),
            RegPair::DE => self.registers.de(),
            RegPair::HL => self.registers.hl(),
            RegPair::SP => self.sp,
        }
    }

    pub(crate) fn write_pair(&mut self, pair: RegPair, value: u16) {
        match pair {
            RegPair::BC => self.registers.set_bc(value),
            RegPair::DE => self.registers.set_de(value),
            RegPair::HL => self.registers.set_hl(value),
            RegPair::SP => self.sp = value,
        }
    }

    pub(crate) fn fetch_byte(&mut self, offset: u16) -> u8 {
        // Operand fetch: PC+offset выставляется на адресную шину,
        // байт идёт через буфер данных. `&mut self`, потому что
        // bus_read обновляет латчи (см. `bus_read` doc-comment).
        self.bus_read(self.pc.wrapping_add(offset))
    }

    pub(crate) fn fetch_word(&mut self, offset: u16) -> u16 {
        self.bus_read_word(self.pc.wrapping_add(offset))
    }

    pub(crate) fn push_word(&mut self, value: u16) {
        let [hi, lo] = value.to_be_bytes();
        self.sp = self.sp.wrapping_sub(1);
        self.bus_write(self.sp, hi);
        self.sp = self.sp.wrapping_sub(1);
        self.bus_write(self.sp, lo);
    }

    pub(crate) fn pop_word(&mut self) -> u16 {
        let lo = self.bus_read(self.sp);
        self.sp = self.sp.wrapping_add(1);
        let hi = self.bus_read(self.sp);
        self.sp = self.sp.wrapping_add(1);
        u16::from_be_bytes([hi, lo])
    }
}
