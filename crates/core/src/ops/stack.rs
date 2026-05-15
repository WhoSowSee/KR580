use crate::{Cpu8080State, Flags, InstructionOutcome};

pub(crate) fn handles(opcode: u8) -> bool {
    opcode & 0xCF == 0xC1 || opcode & 0xCF == 0xC5
}

impl Cpu8080State {
    pub(crate) fn execute_stack_opcode(
        &mut self,
        opcode: u8,
        mnemonic: String,
        pc_before: u16,
        t_states: u8,
    ) -> InstructionOutcome {
        if opcode & 0xCF == 0xC1 {
            self.pop_stack_pair((opcode >> 4) & 3);
        } else {
            self.push_stack_pair((opcode >> 4) & 3);
        }
        self.pc = self.pc.wrapping_add(1);
        self.outcome(Some(opcode), mnemonic, pc_before, t_states, false)
    }

    fn push_stack_pair(&mut self, code: u8) {
        let value = match code & 3 {
            0 => self.registers.bc(),
            1 => self.registers.de(),
            2 => self.registers.hl(),
            _ => u16::from_be_bytes([self.registers.a, self.flags.to_psw()]),
        };
        self.push_word(value);
    }

    fn pop_stack_pair(&mut self, code: u8) {
        let value = self.pop_word();
        match code & 3 {
            0 => self.registers.set_bc(value),
            1 => self.registers.set_de(value),
            2 => self.registers.set_hl(value),
            _ => {
                let [a, psw] = value.to_be_bytes();
                self.registers.a = a;
                self.flags = Flags::from_psw(psw);
            }
        }
    }
}
