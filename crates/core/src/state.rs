use crate::{CoreError, Flags, Memory64K, PortBus, RegisterName, Registers};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Cpu8080State {
    pub registers: Registers,
    pub pc: u16,
    pub sp: u16,
    pub flags: Flags,
    pub memory: Memory64K,
    pub interrupt_request_pending: bool,
    pub interrupt_enable: bool,
    pub interrupt_enable_pending: bool,
    pub halted: bool,
    pub cycle_count: u64,
    pub interrupt_vector_byte: Option<u8>,
    pub tact_phase: Option<u8>,
    pub(crate) active_tacts_remaining: u8,
    pub(crate) active_tacts_total: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstructionOutcome {
    pub opcode: Option<u8>,
    pub mnemonic: String,
    pub pc_before: u16,
    pub pc_after: u16,
    pub t_states: u8,
    pub halted: bool,
    pub interrupt_accepted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TactOutcome {
    pub tact_phase: u8,
    pub instruction_boundary: bool,
    pub cycle_count: u64,
}

impl Cpu8080State {
    pub fn reset_cpu(&mut self) {
        let memory = core::mem::take(&mut self.memory);
        *self = Self {
            memory,
            ..Self::default()
        };
    }

    pub fn reset_ram(&mut self) {
        self.memory.clear();
    }

    pub fn request_interrupt(&mut self, vector_byte: u8) {
        self.interrupt_request_pending = true;
        self.interrupt_vector_byte = Some(vector_byte);
    }

    pub fn set_register(&mut self, register: RegisterName, value: u8) {
        self.registers.set(register, value);
    }

    pub fn get_register(&self, register: RegisterName) -> u8 {
        self.registers.get(register)
    }

    pub fn set_memory(&mut self, address: u16, value: u8) {
        self.memory.write(address, value);
    }

    pub fn step_instruction<B: PortBus>(
        &mut self,
        bus: &mut B,
    ) -> Result<InstructionOutcome, CoreError> {
        if self.active_tacts_remaining > 0 {
            self.cycle_count += u64::from(self.active_tacts_remaining);
            self.active_tacts_remaining = 0;
            self.active_tacts_total = 0;
            self.tact_phase = None;
            return Ok(InstructionOutcome {
                opcode: None,
                mnemonic: "TACT-COMPLETE".to_owned(),
                pc_before: self.pc,
                pc_after: self.pc,
                t_states: 0,
                halted: self.halted,
                interrupt_accepted: false,
            });
        }

        let outcome = self.execute_instruction_boundary(bus)?;
        self.cycle_count += u64::from(outcome.t_states);
        Ok(outcome)
    }

    pub fn step_tact<B: PortBus>(&mut self, bus: &mut B) -> Result<TactOutcome, CoreError> {
        if self.active_tacts_remaining == 0 {
            let t_states = if self.halted && !self.can_accept_interrupt() {
                1
            } else {
                self.execute_instruction_boundary(bus)?.t_states.max(1)
            };
            self.active_tacts_total = t_states;
            self.active_tacts_remaining = t_states;
            self.tact_phase = Some(0);
        }

        let phase = self.active_tacts_total - self.active_tacts_remaining;
        self.active_tacts_remaining -= 1;
        self.cycle_count += 1;
        let boundary = self.active_tacts_remaining == 0;
        self.tact_phase = if boundary { None } else { Some(phase + 1) };

        Ok(TactOutcome {
            tact_phase: phase,
            instruction_boundary: boundary,
            cycle_count: self.cycle_count,
        })
    }

    pub fn run_for_t_states<B: PortBus>(
        &mut self,
        bus: &mut B,
        t_states: u64,
    ) -> Result<(), CoreError> {
        for _ in 0..t_states {
            self.step_tact(bus)?;
        }
        Ok(())
    }

    pub fn run_until_halt<B: PortBus>(
        &mut self,
        bus: &mut B,
        max_instructions: u64,
    ) -> Result<u64, CoreError> {
        let mut executed = 0;
        while !self.halted && executed < max_instructions {
            self.step_instruction(bus)?;
            executed += 1;
        }
        Ok(executed)
    }
}
