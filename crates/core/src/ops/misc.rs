use crate::{CoreError, Cpu8080State, Flags, InstructionOutcome, PortBus, decode_opcode};

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
            0x02 => self.memory.write(self.registers.bc(), self.registers.a),
            0x07 => self.rlc(),
            0x0A => self.registers.a = self.memory.read(self.registers.bc()),
            0x0F => self.rrc(),
            0x12 => self.memory.write(self.registers.de(), self.registers.a),
            0x17 => self.ral(),
            0x1A => self.registers.a = self.memory.read(self.registers.de()),
            0x1F => self.rar(),
            0x22 => self.shld(),
            0x27 => self.daa(),
            0x2A => self.lhld(),
            0x2F => self.registers.a = !self.registers.a,
            0x32 => self.memory.write(self.fetch_word(1), self.registers.a),
            0x37 => self.flags.carry = true,
            0x3A => self.registers.a = self.memory.read(self.fetch_word(1)),
            0x3F => self.flags.carry = !self.flags.carry,
            0xC3 => self.pc = self.fetch_word(1),
            0xC6 => self.add(self.fetch_byte(1), false),
            0xC9 => self.pc = self.pop_word(),
            0xCD => {
                let target = self.fetch_word(1);
                self.push_word(self.pc.wrapping_add(3));
                self.pc = target;
            }
            0xCE => self.add(self.fetch_byte(1), true),
            0xD3 => bus.output(self.fetch_byte(1), self.registers.a)?,
            0xD6 => self.sub(self.fetch_byte(1), false, true),
            0xDB => self.registers.a = bus.input(self.fetch_byte(1))?,
            0xDE => self.sub(self.fetch_byte(1), true, true),
            0xE3 => self.xthl(),
            0xE6 => self.ana(self.fetch_byte(1)),
            0xE9 => self.pc = self.registers.hl(),
            0xEB => self.xchg(),
            0xEE => self.xra(self.fetch_byte(1)),
            0xF3 => {
                self.interrupt_enable = false;
                self.interrupt_enable_pending = false;
            }
            0xF6 => self.ora(self.fetch_byte(1)),
            0xF9 => self.sp = self.registers.hl(),
            0xFB => self.interrupt_enable_pending = true,
            0xFE => self.sub(self.fetch_byte(1), false, false),
            _ => unreachable!("misc dispatch reached non-misc opcode {opcode:#04X}"),
        }
        self.advance_misc_pc(opcode);
        let cycles = decode_opcode(opcode)?.timing.t_states_taken;
        Ok(self.outcome(Some(opcode), mnemonic, pc_before, cycles, false))
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

    pub(crate) fn push_stack_pair(&mut self, code: u8) {
        let value = match code & 3 {
            0 => self.registers.bc(),
            1 => self.registers.de(),
            2 => self.registers.hl(),
            _ => u16::from_be_bytes([self.registers.a, self.flags.to_psw()]),
        };
        self.push_word(value);
    }

    pub(crate) fn pop_stack_pair(&mut self, code: u8) {
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
            0x02 | 0x07 | 0x0A | 0x0F | 0x12 | 0x17 | 0x1A | 0x1F | 0x27 | 0x2F | 0x37 | 0x3F
            | 0xE3 | 0xEB | 0xF9 => self.pc = self.pc.wrapping_add(1),
            0x22 | 0x2A | 0x32 | 0x3A => self.pc = self.pc.wrapping_add(3),
            0xC3 | 0xC9 | 0xCD | 0xE9 => {}
            0xC6 | 0xCE | 0xD3 | 0xD6 | 0xDB | 0xDE | 0xE6 | 0xEE | 0xF6 | 0xFE => {
                self.pc = self.pc.wrapping_add(2)
            }
            0xF3 | 0xFB => self.pc = self.pc.wrapping_add(1),
            0x00 => {}
            _ => {}
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

    fn shld(&mut self) {
        let address = self.fetch_word(1);
        self.memory.write(address, self.registers.l);
        self.memory.write(address.wrapping_add(1), self.registers.h);
    }

    fn lhld(&mut self) {
        let address = self.fetch_word(1);
        self.registers.l = self.memory.read(address);
        self.registers.h = self.memory.read(address.wrapping_add(1));
    }

    fn xchg(&mut self) {
        core::mem::swap(&mut self.registers.d, &mut self.registers.h);
        core::mem::swap(&mut self.registers.e, &mut self.registers.l);
    }

    fn xthl(&mut self) {
        let lo = self.memory.read(self.sp);
        let hi = self.memory.read(self.sp.wrapping_add(1));
        self.memory.write(self.sp, self.registers.l);
        self.memory.write(self.sp.wrapping_add(1), self.registers.h);
        self.registers.h = hi;
        self.registers.l = lo;
    }
}
