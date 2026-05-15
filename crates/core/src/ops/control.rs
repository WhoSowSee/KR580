use crate::{Cpu8080State, InstructionOutcome};

pub(crate) fn handles(opcode: u8) -> bool {
    opcode & 0xC7 == 0xC0
        || opcode & 0xC7 == 0xC2
        || opcode & 0xC7 == 0xC4
        || opcode & 0xC7 == 0xC7
        || matches!(opcode, 0xC3 | 0xC9 | 0xCD | 0xE9)
}

impl Cpu8080State {
    pub(crate) fn execute_control_opcode(
        &mut self,
        opcode: u8,
        mnemonic: String,
        pc_before: u16,
        t_states: u8,
    ) -> InstructionOutcome {
        if opcode & 0xC7 == 0xC0 {
            let taken = self.condition((opcode >> 3) & 7);
            if taken {
                self.pc = self.pop_word();
            } else {
                self.pc = self.pc.wrapping_add(1);
            }
            return self.outcome(
                Some(opcode),
                mnemonic,
                pc_before,
                if taken { 11 } else { 5 },
                false,
            );
        }

        if opcode & 0xC7 == 0xC2 {
            let target = self.fetch_word(1);
            let taken = self.condition((opcode >> 3) & 7);
            self.pc = if taken {
                target
            } else {
                self.pc.wrapping_add(3)
            };
            return self.outcome(Some(opcode), mnemonic, pc_before, t_states, false);
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
            return self.outcome(
                Some(opcode),
                mnemonic,
                pc_before,
                if taken { 17 } else { 11 },
                false,
            );
        }

        if opcode & 0xC7 == 0xC7 {
            let rst = (opcode >> 3) & 7;
            self.push_word(self.pc.wrapping_add(1));
            self.pc = u16::from(rst) * 8;
            return self.outcome(Some(opcode), mnemonic, pc_before, t_states, false);
        }

        match opcode {
            0xC3 => self.pc = self.fetch_word(1),
            0xC9 => self.pc = self.pop_word(),
            0xCD => {
                let target = self.fetch_word(1);
                self.push_word(self.pc.wrapping_add(3));
                self.pc = target;
            }
            0xE9 => self.pc = self.registers.hl(),
            _ => unreachable!("control dispatch reached non-control opcode {opcode:#04X}"),
        }
        self.outcome(Some(opcode), mnemonic, pc_before, t_states, false)
    }

    pub(crate) fn condition(&self, code: u8) -> bool {
        match code & 7 {
            0 => !self.flags.zero,
            1 => self.flags.zero,
            2 => !self.flags.carry,
            3 => self.flags.carry,
            4 => !self.flags.parity,
            5 => self.flags.parity,
            6 => !self.flags.sign,
            _ => self.flags.sign,
        }
    }
}
