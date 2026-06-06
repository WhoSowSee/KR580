use crate::ops::operand::RegPair;
use crate::{Cpu8080State, InstructionOutcome};

pub(crate) fn handles(opcode: u8) -> bool {
    (0x40..=0x7F).contains(&opcode)
        || opcode & 0xC7 == 0x06
        || opcode & 0xCF == 0x01
        || opcode & 0xCF == 0x03
        || opcode & 0xCF == 0x09
        || opcode & 0xCF == 0x0B
        || matches!(
            opcode,
            0x02 | 0x0A | 0x12 | 0x1A | 0x22 | 0x2A | 0x32 | 0x3A | 0xE3 | 0xEB | 0xF9
        )
}

impl Cpu8080State {
    pub(crate) fn execute_data_opcode(
        &mut self,
        opcode: u8,
        mnemonic: String,
        pc_before: u16,
        t_states: u8,
    ) -> InstructionOutcome {
        if (0x40..=0x7F).contains(&opcode) {
            if opcode == 0x76 {
                self.pc = self.pc.wrapping_add(1);
                self.halted = true;
                return self.outcome(Some(opcode), mnemonic, pc_before, t_states, false);
            }
            let dst = (opcode >> 3) & 7;
            let src = opcode & 7;
            let value = self.read_reg_code(src);
            self.write_reg_code(dst, value);
            self.pc = self.pc.wrapping_add(1);
            return self.outcome(Some(opcode), mnemonic, pc_before, t_states, false);
        }

        if opcode & 0xC7 == 0x06 {
            let reg = (opcode >> 3) & 7;
            let value = self.fetch_byte(1);
            self.write_reg_code(reg, value);
            self.pc = self.pc.wrapping_add(2);
            return self.outcome(Some(opcode), mnemonic, pc_before, t_states, false);
        }

        if opcode & 0xCF == 0x01 {
            // LXI rp,d16: operand passes through W/Z on the way to rp.
            let value = self.fetch_word(1);
            self.registers.set_wz(value);
            self.write_pair(RegPair::from_code((opcode >> 4) & 3), value);
            self.pc = self.pc.wrapping_add(3);
            return self.outcome(Some(opcode), mnemonic, pc_before, t_states, false);
        }

        if opcode & 0xCF == 0x03 {
            let pair = RegPair::from_code((opcode >> 4) & 3);
            self.write_pair(pair, self.read_pair(pair).wrapping_add(1));
            self.pc = self.pc.wrapping_add(1);
            return self.outcome(Some(opcode), mnemonic, pc_before, t_states, false);
        }

        if opcode & 0xCF == 0x09 {
            let rhs = self.read_pair(RegPair::from_code((opcode >> 4) & 3));
            let hl = self.registers.hl();
            let sum = hl as u32 + rhs as u32;
            self.registers.set_hl(sum as u16);
            self.flags.carry = sum > 0xFFFF;
            self.pc = self.pc.wrapping_add(1);
            return self.outcome(Some(opcode), mnemonic, pc_before, t_states, false);
        }

        if opcode & 0xCF == 0x0B {
            let pair = RegPair::from_code((opcode >> 4) & 3);
            self.write_pair(pair, self.read_pair(pair).wrapping_sub(1));
            self.pc = self.pc.wrapping_add(1);
            return self.outcome(Some(opcode), mnemonic, pc_before, t_states, false);
        }

        match opcode {
            // STAX/LDAX (BC|DE) intentionally skip WZ – the address
            // is already on the latches, no microcode parks it.
            0x02 => self.bus_write(self.registers.bc(), self.registers.a),
            0x0A => self.registers.a = self.bus_read(self.registers.bc()),
            0x12 => self.bus_write(self.registers.de(), self.registers.a),
            0x1A => self.registers.a = self.bus_read(self.registers.de()),
            0x22 => self.shld(),
            0x2A => self.lhld(),
            0x32 => {
                let address = self.fetch_word(1);
                self.registers.set_wz(address);
                self.bus_write(address, self.registers.a);
            }
            0x3A => {
                let address = self.fetch_word(1);
                self.registers.set_wz(address);
                self.registers.a = self.bus_read(address);
            }
            0xE3 => self.xthl(),
            0xEB => self.xchg(),
            0xF9 => {
                let value = self.registers.hl();
                self.registers.set_wz(value);
                self.sp = value;
            }
            _ => unreachable!("data dispatch reached non-data opcode {opcode:#04X}"),
        }
        self.advance_data_pc(opcode);
        self.outcome(Some(opcode), mnemonic, pc_before, t_states, false)
    }

    fn advance_data_pc(&mut self, opcode: u8) {
        match opcode {
            0x02 | 0x0A | 0x12 | 0x1A | 0xE3 | 0xEB | 0xF9 => self.pc = self.pc.wrapping_add(1),
            0x22 | 0x2A | 0x32 | 0x3A => self.pc = self.pc.wrapping_add(3),
            _ => {}
        }
    }

    fn shld(&mut self) {
        let address = self.fetch_word(1);
        self.registers.set_wz(address);
        self.bus_write(address, self.registers.l);
        self.bus_write(address.wrapping_add(1), self.registers.h);
    }

    fn lhld(&mut self) {
        let address = self.fetch_word(1);
        self.registers.set_wz(address);
        self.registers.l = self.bus_read(address);
        self.registers.h = self.bus_read(address.wrapping_add(1));
    }

    fn xchg(&mut self) {
        // HL → WZ; DE → HL; WZ → DE. WZ residue = previous HL.
        let prev_hl = self.registers.hl();
        let prev_de = self.registers.de();
        self.registers.set_wz(prev_hl);
        self.registers.set_hl(prev_de);
        self.registers.set_de(prev_hl);
    }

    fn xthl(&mut self) {
        let lo = self.bus_read(self.sp);
        let hi = self.bus_read(self.sp.wrapping_add(1));
        self.registers.w = hi;
        self.registers.z = lo;
        self.bus_write(self.sp, self.registers.l);
        self.bus_write(self.sp.wrapping_add(1), self.registers.h);
        self.registers.h = hi;
        self.registers.l = lo;
    }
}
