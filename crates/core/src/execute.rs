use crate::ops::operand::RegPair;
use crate::{CoreError, Cpu8080State, DecodeError, InstructionOutcome, PortBus, decode_opcode};

impl Cpu8080State {
    pub(crate) fn can_accept_interrupt(&self) -> bool {
        self.interrupt_request_pending
            && self.interrupt_enable
            && self.interrupt_vector_byte.is_some()
    }

    pub(crate) fn execute_instruction_boundary<B: PortBus>(
        &mut self,
        bus: &mut B,
    ) -> Result<InstructionOutcome, CoreError> {
        if self.can_accept_interrupt() {
            return self.accept_interrupt();
        }
        if self.halted {
            return Ok(self.outcome(None, "HALTED", self.pc, 0, false));
        }

        let pending_before = self.interrupt_enable_pending;
        let opcode = self.memory.read(self.pc);
        let outcome = self.execute_opcode(opcode, bus)?;
        if pending_before && self.interrupt_enable_pending {
            self.interrupt_enable = true;
            self.interrupt_enable_pending = false;
        }
        Ok(outcome)
    }

    fn accept_interrupt(&mut self) -> Result<InstructionOutcome, CoreError> {
        let vector = self
            .interrupt_vector_byte
            .take()
            .expect("checked by can_accept_interrupt");
        if vector & 0xC7 != 0xC7 {
            return Err(DecodeError::InvalidInterruptVector(vector).into());
        }
        self.interrupt_enable = false;
        self.interrupt_enable_pending = false;
        self.interrupt_request_pending = false;
        self.halted = false;

        let pc_before = self.pc;
        let rst = (vector >> 3) & 7;
        self.push_word(self.pc);
        self.pc = u16::from(rst) * 8;
        Ok(InstructionOutcome {
            opcode: Some(vector),
            mnemonic: format!("RST {}", rst),
            pc_before,
            pc_after: self.pc,
            t_states: 11,
            halted: false,
            interrupt_accepted: true,
        })
    }

    fn execute_opcode<B: PortBus>(
        &mut self,
        opcode: u8,
        bus: &mut B,
    ) -> Result<InstructionOutcome, CoreError> {
        let pc_before = self.pc;
        let info = decode_opcode(opcode)?;

        if (0x40..=0x7F).contains(&opcode) {
            if opcode == 0x76 {
                self.pc = self.pc.wrapping_add(1);
                self.halted = true;
                return Ok(self.outcome(Some(opcode), info.mnemonic, pc_before, 7, false));
            }
            let dst = (opcode >> 3) & 7;
            let src = opcode & 7;
            let value = self.read_reg_code(src);
            self.write_reg_code(dst, value);
            self.pc = self.pc.wrapping_add(1);
            return Ok(self.outcome(
                Some(opcode),
                info.mnemonic,
                pc_before,
                info.timing.t_states_taken,
                false,
            ));
        }

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
            return Ok(self.outcome(
                Some(opcode),
                info.mnemonic,
                pc_before,
                info.timing.t_states_taken,
                false,
            ));
        }

        if opcode & 0xC7 == 0x04 {
            let reg = (opcode >> 3) & 7;
            let value = self.read_reg_code(reg);
            let result = self.inr_value(value);
            self.write_reg_code(reg, result);
            self.pc = self.pc.wrapping_add(1);
            return Ok(self.outcome(
                Some(opcode),
                info.mnemonic,
                pc_before,
                info.timing.t_states_taken,
                false,
            ));
        }
        if opcode & 0xC7 == 0x05 {
            let reg = (opcode >> 3) & 7;
            let value = self.read_reg_code(reg);
            let result = self.dcr_value(value);
            self.write_reg_code(reg, result);
            self.pc = self.pc.wrapping_add(1);
            return Ok(self.outcome(
                Some(opcode),
                info.mnemonic,
                pc_before,
                info.timing.t_states_taken,
                false,
            ));
        }
        if opcode & 0xC7 == 0x06 {
            let reg = (opcode >> 3) & 7;
            self.write_reg_code(reg, self.fetch_byte(1));
            self.pc = self.pc.wrapping_add(2);
            return Ok(self.outcome(
                Some(opcode),
                info.mnemonic,
                pc_before,
                info.timing.t_states_taken,
                false,
            ));
        }
        if opcode & 0xCF == 0x01 {
            self.write_pair(RegPair::from_code((opcode >> 4) & 3), self.fetch_word(1));
            self.pc = self.pc.wrapping_add(3);
            return Ok(self.outcome(Some(opcode), info.mnemonic, pc_before, 10, false));
        }
        if opcode & 0xCF == 0x03 {
            let pair = RegPair::from_code((opcode >> 4) & 3);
            self.write_pair(pair, self.read_pair(pair).wrapping_add(1));
            self.pc = self.pc.wrapping_add(1);
            return Ok(self.outcome(Some(opcode), info.mnemonic, pc_before, 5, false));
        }
        if opcode & 0xCF == 0x09 {
            let rhs = self.read_pair(RegPair::from_code((opcode >> 4) & 3));
            let hl = self.registers.hl();
            let sum = hl as u32 + rhs as u32;
            self.registers.set_hl(sum as u16);
            self.flags.carry = sum > 0xFFFF;
            self.pc = self.pc.wrapping_add(1);
            return Ok(self.outcome(Some(opcode), info.mnemonic, pc_before, 10, false));
        }
        if opcode & 0xCF == 0x0B {
            let pair = RegPair::from_code((opcode >> 4) & 3);
            self.write_pair(pair, self.read_pair(pair).wrapping_sub(1));
            self.pc = self.pc.wrapping_add(1);
            return Ok(self.outcome(Some(opcode), info.mnemonic, pc_before, 5, false));
        }
        if opcode & 0xC7 == 0xC0 {
            let taken = self.condition((opcode >> 3) & 7);
            if taken {
                self.pc = self.pop_word();
            } else {
                self.pc = self.pc.wrapping_add(1);
            }
            return Ok(self.outcome(
                Some(opcode),
                info.mnemonic,
                pc_before,
                if taken { 11 } else { 5 },
                false,
            ));
        }
        if opcode & 0xC7 == 0xC2 {
            let target = self.fetch_word(1);
            let taken = self.condition((opcode >> 3) & 7);
            self.pc = if taken {
                target
            } else {
                self.pc.wrapping_add(3)
            };
            return Ok(self.outcome(Some(opcode), info.mnemonic, pc_before, 10, false));
        }
        if opcode & 0xC7 == 0xC4 {
            let target = self.fetch_word(1);
            let taken = self.condition((opcode >> 3) & 7);
            if taken {
                self.push_word(self.pc.wrapping_add(3));
                self.pc = target;
            } else {
                self.pc = self.pc.wrapping_add(3);
            }
            return Ok(self.outcome(
                Some(opcode),
                info.mnemonic,
                pc_before,
                if taken { 17 } else { 11 },
                false,
            ));
        }
        if opcode & 0xC7 == 0xC7 {
            let rst = (opcode >> 3) & 7;
            self.push_word(self.pc.wrapping_add(1));
            self.pc = u16::from(rst) * 8;
            return Ok(self.outcome(Some(opcode), info.mnemonic, pc_before, 11, false));
        }
        if opcode & 0xCF == 0xC1 {
            self.pop_stack_pair((opcode >> 4) & 3);
            self.pc = self.pc.wrapping_add(1);
            return Ok(self.outcome(Some(opcode), info.mnemonic, pc_before, 10, false));
        }
        if opcode & 0xCF == 0xC5 {
            self.push_stack_pair((opcode >> 4) & 3);
            self.pc = self.pc.wrapping_add(1);
            return Ok(self.outcome(Some(opcode), info.mnemonic, pc_before, 11, false));
        }

        self.execute_misc(opcode, bus, pc_before, info.mnemonic)
    }
}
