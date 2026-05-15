use crate::{Cpu8080State, InstructionOutcome};

pub(crate) fn handles(opcode: u8) -> bool {
    (0x80..=0xBF).contains(&opcode)
        || opcode & 0xC7 == 0x04
        || opcode & 0xC7 == 0x05
        || matches!(
            opcode,
            0xC6 | 0xCE | 0xD6 | 0xDE | 0xE6 | 0xEE | 0xF6 | 0xFE
        )
}

impl Cpu8080State {
    pub(crate) fn execute_alu_opcode(
        &mut self,
        opcode: u8,
        mnemonic: String,
        pc_before: u16,
        t_states: u8,
    ) -> InstructionOutcome {
        if (0x80..=0xBF).contains(&opcode) {
            let value = self.read_reg_code(opcode & 7);
            match (opcode >> 3) & 7 {
                0 => self.add(value, false),
                1 => self.add(value, true),
                2 => self.sub(value, false, true),
                3 => self.sub(value, true, true),
                4 => self.ana(value),
                5 => self.xra(value),
                6 => self.ora(value),
                _ => self.sub(value, false, false),
            }
            self.pc = self.pc.wrapping_add(1);
            return self.outcome(Some(opcode), mnemonic, pc_before, t_states, false);
        }

        if opcode & 0xC7 == 0x04 {
            let reg = (opcode >> 3) & 7;
            let value = self.read_reg_code(reg);
            let result = self.inr_value(value);
            self.write_reg_code(reg, result);
            self.pc = self.pc.wrapping_add(1);
            return self.outcome(Some(opcode), mnemonic, pc_before, t_states, false);
        }

        if opcode & 0xC7 == 0x05 {
            let reg = (opcode >> 3) & 7;
            let value = self.read_reg_code(reg);
            let result = self.dcr_value(value);
            self.write_reg_code(reg, result);
            self.pc = self.pc.wrapping_add(1);
            return self.outcome(Some(opcode), mnemonic, pc_before, t_states, false);
        }

        match opcode {
            0xC6 => self.add(self.fetch_byte(1), false),
            0xCE => self.add(self.fetch_byte(1), true),
            0xD6 => self.sub(self.fetch_byte(1), false, true),
            0xDE => self.sub(self.fetch_byte(1), true, true),
            0xE6 => self.ana(self.fetch_byte(1)),
            0xEE => self.xra(self.fetch_byte(1)),
            0xF6 => self.ora(self.fetch_byte(1)),
            0xFE => self.sub(self.fetch_byte(1), false, false),
            _ => unreachable!("ALU dispatch reached non-ALU opcode {opcode:#04X}"),
        }
        self.pc = self.pc.wrapping_add(2);
        self.outcome(Some(opcode), mnemonic, pc_before, t_states, false)
    }

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
