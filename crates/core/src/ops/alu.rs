use crate::Cpu8080State;

impl Cpu8080State {
    pub(crate) fn inr_value(&mut self, value: u8) -> u8 {
        let result = value.wrapping_add(1);
        self.flags.auxiliary_carry = (value & 0x0F) == 0x0F;
        self.flags.set_sign_zero_parity(result);
        result
    }

    pub(crate) fn dcr_value(&mut self, value: u8) -> u8 {
        let result = value.wrapping_sub(1);
        self.flags.auxiliary_carry = (result & 0x0F) != 0x0F;
        self.flags.set_sign_zero_parity(result);
        result
    }

    pub(crate) fn add(&mut self, value: u8, carry: bool) {
        let carry_in = u8::from(carry && self.flags.carry);
        let a = self.registers.a;
        let sum = a as u16 + value as u16 + carry_in as u16;
        let result = sum as u8;
        self.flags.carry = sum > 0xFF;
        self.flags.auxiliary_carry = ((a & 0x0F) + (value & 0x0F) + carry_in) > 0x0F;
        self.flags.set_sign_zero_parity(result);
        self.registers.a = result;
    }

    pub(crate) fn sub(&mut self, value: u8, borrow: bool, write_result: bool) {
        let borrow_in = u8::from(borrow && self.flags.carry);
        let a = self.registers.a;
        let result = a.wrapping_sub(value).wrapping_sub(borrow_in);
        self.flags.carry = (a as u16) < (value as u16 + borrow_in as u16);
        self.flags.auxiliary_carry = (a & 0x0F) >= ((value & 0x0F).wrapping_add(borrow_in));
        self.flags.set_sign_zero_parity(result);
        if write_result {
            self.registers.a = result;
        }
    }

    pub(crate) fn ana(&mut self, value: u8) {
        let a = self.registers.a;
        let result = a & value;
        self.flags.carry = false;
        self.flags.auxiliary_carry = ((a | value) & 0x08) != 0;
        self.flags.set_sign_zero_parity(result);
        self.registers.a = result;
    }

    pub(crate) fn xra(&mut self, value: u8) {
        let result = self.registers.a ^ value;
        self.flags.carry = false;
        self.flags.auxiliary_carry = false;
        self.flags.set_sign_zero_parity(result);
        self.registers.a = result;
    }

    pub(crate) fn ora(&mut self, value: u8) {
        let result = self.registers.a | value;
        self.flags.carry = false;
        self.flags.auxiliary_carry = false;
        self.flags.set_sign_zero_parity(result);
        self.registers.a = result;
    }

    pub(crate) fn daa(&mut self) {
        let a = self.registers.a;
        let mut correction = 0u8;
        let mut carry = self.flags.carry;

        if (a & 0x0F) > 9 || self.flags.auxiliary_carry {
            correction |= 0x06;
        }
        if a > 0x99 || self.flags.carry {
            correction |= 0x60;
            carry = true;
        }

        let result = a.wrapping_add(correction);
        self.flags.auxiliary_carry = ((a & 0x0F) + (correction & 0x0F)) > 0x0F;
        self.flags.carry = carry;
        self.flags.set_sign_zero_parity(result);
        self.registers.a = result;
    }
}
