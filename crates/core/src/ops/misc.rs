use crate::{CoreError, Cpu8080State, InstructionOutcome, PortBus};

pub(crate) fn handles(opcode: u8) -> bool {
    matches!(
        opcode,
        0x00 | 0x07 | 0x0F | 0x17 | 0x1F | 0x27 | 0x2F | 0x37 | 0x3F | 0xD3 | 0xDB | 0xF3 | 0xFB
    )
}

impl Cpu8080State {
    pub(crate) fn execute_misc<B: PortBus>(
        &mut self,
        opcode: u8,
        bus: &mut B,
        pc_before: u16,
        mnemonic: String,
    ) -> Result<InstructionOutcome, CoreError> {
        match opcode {
            0x00 => self.pc = self.pc.wrapping_add(1),
            0x07 => self.rlc(),
            0x0F => self.rrc(),
            0x17 => self.ral(),
            0x1F => self.rar(),
            0x27 => self.daa(),
            0x2F => self.registers.a = !self.registers.a,
            0x37 => self.flags.carry = true,
            0x3F => self.flags.carry = !self.flags.carry,
            0xD3 => {
                let port = self.fetch_byte(1);
                bus.output(port, self.registers.a)?;
            }
            0xDB => {
                let port = self.fetch_byte(1);
                self.registers.a = bus.input(port)?;
            }
            0xF3 => {
                self.interrupt_enable = false;
                self.interrupt_enable_pending = false;
            }
            0xFB => self.interrupt_enable_pending = true,
            _ => unreachable!("misc dispatch reached non-misc opcode {opcode:#04X}"),
        }
        self.advance_misc_pc(opcode);
        Ok(self.outcome(
            Some(opcode),
            mnemonic,
            pc_before,
            self.timing_for_misc(opcode),
            false,
        ))
    }

    pub(crate) fn outcome(
        &self,
        opcode: Option<u8>,
        mnemonic: impl Into<String>,
        pc_before: u16,
        t_states: u8,
        interrupt_accepted: bool,
    ) -> InstructionOutcome {
        InstructionOutcome {
            opcode,
            mnemonic: mnemonic.into(),
            pc_before,
            pc_after: self.pc,
            t_states,
            halted: self.halted,
            interrupt_accepted,
        }
    }

    fn advance_misc_pc(&mut self, opcode: u8) {
        match opcode {
            0x07 | 0x0F | 0x17 | 0x1F | 0x27 | 0x2F | 0x37 | 0x3F => {
                self.pc = self.pc.wrapping_add(1)
            }
            0xD3 | 0xDB => self.pc = self.pc.wrapping_add(2),
            0xF3 | 0xFB => self.pc = self.pc.wrapping_add(1),
            0x00 => {}
            _ => {}
        }
    }

    fn timing_for_misc(&self, opcode: u8) -> u8 {
        match opcode {
            0xD3 | 0xDB => 10,
            _ => 4,
        }
    }

    fn rlc(&mut self) {
        let a = self.registers.a;
        self.registers.a = a.rotate_left(1);
        self.flags.carry = a & 0x80 != 0;
    }

    fn rrc(&mut self) {
        let a = self.registers.a;
        self.registers.a = a.rotate_right(1);
        self.flags.carry = a & 0x01 != 0;
    }

    fn ral(&mut self) {
        let a = self.registers.a;
        let carry = self.flags.carry;
        self.flags.carry = a & 0x80 != 0;
        self.registers.a = (a << 1) | u8::from(carry);
    }

    fn rar(&mut self) {
        let a = self.registers.a;
        let carry = self.flags.carry;
        self.flags.carry = a & 0x01 != 0;
        self.registers.a = (a >> 1) | (u8::from(carry) << 7);
    }
}
