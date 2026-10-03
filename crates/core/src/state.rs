use crate::{
    CoreError, Flags, MachineCycleLayout, Memory64K, PortBus, RegisterName, Registers,
    ValidationError,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cpu8080State<M = Memory64K> {
    pub registers: Registers,
    pub pc: u16,
    pub sp: u16,
    pub flags: Flags,
    pub memory: M,
    pub interrupt_request_pending: bool,
    pub interrupt_enable: bool,
    pub interrupt_enable_pending: bool,
    pub halted: bool,
    pub cycle_count: u64,
    pub interrupt_vector_byte: Option<u8>,
    pub tact_phase: Option<u8>,
    /// Last completed T-phase persists at instruction boundaries; None after reset.
    pub last_completed_tact_phase: Option<u8>,
    pub(crate) active_tacts_remaining: u8,
    pub(crate) active_tacts_total: u8,
    pub(crate) active_opcode: Option<u8>,
    pub(crate) active_branch_taken: bool,
    /// Last opcode latched on M1, retained until the next M1 including after HLT.
    pub last_fetched_opcode: u8,
    /// Last latched data bus byte, including the HLT opcode after PC advances.
    pub last_data_bus_byte: u8,
    /// Last latched address, including the HLT address after PC advances.
    pub last_address_bus: u16,
}

pub type CpuMetadata = Cpu8080State<()>;

impl Default for Cpu8080State {
    fn default() -> Self {
        Self {
            registers: Registers::default(),
            pc: 0,
            sp: Self::RESET_SP,
            flags: Flags::default(),
            memory: Memory64K::default(),
            interrupt_request_pending: false,
            interrupt_enable: false,
            interrupt_enable_pending: false,
            halted: false,
            cycle_count: 0,
            interrupt_vector_byte: None,
            tact_phase: None,
            last_completed_tact_phase: None,
            active_tacts_remaining: 0,
            active_tacts_total: 0,
            active_opcode: None,
            active_branch_taken: true,
            last_fetched_opcode: 0,
            last_data_bus_byte: 0,
            last_address_bus: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstructionOutcome<M = String> {
    pub opcode: Option<u8>,
    pub mnemonic: M,
    pub pc_before: u16,
    pub pc_after: u16,
    pub t_states: u8,
    pub halted: bool,
    pub interrupt_accepted: bool,
}

pub type InstructionStep = InstructionOutcome<&'static str>;

impl From<InstructionStep> for InstructionOutcome {
    fn from(step: InstructionStep) -> Self {
        Self {
            opcode: step.opcode,
            mnemonic: step.mnemonic.to_owned(),
            pc_before: step.pc_before,
            pc_after: step.pc_after,
            t_states: step.t_states,
            halted: step.halted,
            interrupt_accepted: step.interrupt_accepted,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TactOutcome {
    pub tact_phase: u8,
    pub instruction_boundary: bool,
    pub cycle_count: u64,
}

impl Cpu8080State {
    pub fn metadata(&self) -> CpuMetadata {
        Cpu8080State {
            memory: (),
            registers: self.registers,
            pc: self.pc,
            sp: self.sp,
            flags: self.flags,
            interrupt_request_pending: self.interrupt_request_pending,
            interrupt_enable: self.interrupt_enable,
            interrupt_enable_pending: self.interrupt_enable_pending,
            halted: self.halted,
            cycle_count: self.cycle_count,
            interrupt_vector_byte: self.interrupt_vector_byte,
            tact_phase: self.tact_phase,
            last_completed_tact_phase: self.last_completed_tact_phase,
            active_tacts_remaining: self.active_tacts_remaining,
            active_tacts_total: self.active_tacts_total,
            active_opcode: self.active_opcode,
            active_branch_taken: self.active_branch_taken,
            last_fetched_opcode: self.last_fetched_opcode,
            last_data_bus_byte: self.last_data_bus_byte,
            last_address_bus: self.last_address_bus,
        }
    }

    pub fn apply_metadata(&mut self, metadata: CpuMetadata) {
        self.registers = metadata.registers;
        self.pc = metadata.pc;
        self.sp = metadata.sp;
        self.flags = metadata.flags;
        self.interrupt_request_pending = metadata.interrupt_request_pending;
        self.interrupt_enable = metadata.interrupt_enable;
        self.interrupt_enable_pending = metadata.interrupt_enable_pending;
        self.halted = metadata.halted;
        self.cycle_count = metadata.cycle_count;
        self.interrupt_vector_byte = metadata.interrupt_vector_byte;
        self.tact_phase = metadata.tact_phase;
        self.last_completed_tact_phase = metadata.last_completed_tact_phase;
        self.active_tacts_remaining = metadata.active_tacts_remaining;
        self.active_tacts_total = metadata.active_tacts_total;
        self.active_opcode = metadata.active_opcode;
        self.active_branch_taken = metadata.active_branch_taken;
        self.last_fetched_opcode = metadata.last_fetched_opcode;
        self.last_data_bus_byte = metadata.last_data_bus_byte;
        self.last_address_bus = metadata.last_address_bus;
    }

    /// Reset chooses SP=0xFFFF; physical 8080 hardware leaves SP unspecified.
    pub const RESET_SP: u16 = 0xFFFF;

    pub fn reset_cpu(&mut self) {
        let memory = core::mem::take(&mut self.memory);
        *self = Self {
            memory,
            sp: Self::RESET_SP,
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

    pub fn set_memory_block(&mut self, start: u16, values: &[u8]) -> Result<(), ValidationError> {
        let end = u32::from(start) + values.len() as u32;
        if end > Memory64K::SIZE as u32 {
            return Err(ValidationError::MemoryRange { start, end });
        }
        self.memory.as_mut_slice()[start as usize..end as usize].copy_from_slice(values);
        Ok(())
    }

    /// CPU memory traffic must update both address and data bus latches.
    pub(crate) fn bus_read(&mut self, address: u16) -> u8 {
        let value = self.memory.read(address);
        self.last_address_bus = address;
        self.last_data_bus_byte = value;
        value
    }

    pub(crate) fn bus_write(&mut self, address: u16, value: u8) {
        self.memory.write(address, value);
        self.last_address_bus = address;
        self.last_data_bus_byte = value;
    }

    /// Two machine cycles low → high; latches end up holding the high byte.
    pub(crate) fn bus_read_word(&mut self, address: u16) -> u16 {
        let lo = self.bus_read(address);
        let hi = self.bus_read(address.wrapping_add(1));
        u16::from(lo) | (u16::from(hi) << 8)
    }

    pub(crate) fn fetch_opcode(&mut self) -> u8 {
        let opcode = self.bus_read(self.pc);
        self.last_fetched_opcode = opcode;
        opcode
    }

    /// Reads memory without changing bus latches.
    pub fn peek(&self, address: u16) -> u8 {
        self.memory.read(address)
    }

    pub fn step_instruction<B: PortBus>(
        &mut self,
        bus: &mut B,
    ) -> Result<InstructionOutcome, CoreError> {
        self.step_instruction_metadata(bus).map(Into::into)
    }

    pub fn step_instruction_metadata<B: PortBus>(
        &mut self,
        bus: &mut B,
    ) -> Result<InstructionStep, CoreError> {
        if self.active_tacts_remaining > 0 {
            let remaining = self.active_tacts_remaining;
            let total = self.active_tacts_total;
            let outcome = self.execute_instruction_boundary(bus)?;
            self.cycle_count += u64::from(remaining);
            if total > 0 {
                self.last_completed_tact_phase = Some(total - 1);
            }
            self.clear_active_tact();
            return Ok(outcome);
        }

        let outcome = self.execute_instruction_boundary(bus)?;
        self.cycle_count += u64::from(outcome.t_states);
        if outcome.t_states > 0 {
            self.last_completed_tact_phase = Some(outcome.t_states - 1);
        }
        Ok(outcome)
    }

    pub(crate) fn clear_active_tact(&mut self) {
        self.active_tacts_remaining = 0;
        self.active_tacts_total = 0;
        self.active_opcode = None;
        self.active_branch_taken = true;
        self.tact_phase = None;
    }

    pub fn timing_opcode(&self) -> u8 {
        self.active_opcode.unwrap_or(self.last_fetched_opcode)
    }

    pub fn timing_branch_taken(&self, layout: MachineCycleLayout, phase: u8) -> bool {
        if self.active_tacts_remaining > 0 {
            return self.active_branch_taken;
        }
        if let Some(not_taken) = layout.not_taken {
            let not_taken_total: u8 = not_taken.iter().sum();
            if phase < not_taken_total {
                return false;
            }
        }
        true
    }

    pub fn tact_walk_active(&self) -> bool {
        self.active_tacts_remaining > 0
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
            self.step_instruction_metadata(bus)?;
            executed += 1;
        }
        Ok(executed)
    }
}
