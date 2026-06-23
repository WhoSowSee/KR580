use crate::{
    CoreError, Flags, MachineCycleLayout, Memory64K, PortBus, RegisterName, Registers,
    ValidationError,
};

#[derive(Clone, Debug, PartialEq, Eq)]
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
    /// Last executed T-phase of the current/just-finished instruction.
    /// `tact_phase` resets to `None` at boundaries; this field holds
    /// `total - 1` at completion so the UI freezes on the final T.
    /// `None` only on cold start / Reset.
    pub last_completed_tact_phase: Option<u8>,
    pub(crate) active_tacts_remaining: u8,
    pub(crate) active_tacts_total: u8,
    pub(crate) active_opcode: Option<u8>,
    pub(crate) active_branch_taken: bool,
    /// Mirror of the chip's IR: holds the last opcode fetched on M1
    /// until the next M1. After `HLT` a `memory.read(pc)` look-ahead
    /// would show NOP from blank RAM; the IR still reads `0x76`.
    pub last_fetched_opcode: u8,
    /// Mirror of the chip's data bus latch (D7-D0). After `HLT` it
    /// must show `0x76`, not the byte at the new PC.
    pub last_data_bus_byte: u8,
    /// Mirror of the chip's address bus latch (A0-A15). PC, HL, SP,
    /// and 16-bit immediates take turns on it. After `HLT` PC=halt+1
    /// but the latch still shows the HLT address.
    pub last_address_bus: u16,
}

/// `#[derive(Default)]` would yield `sp: 0`; reference uses `0xFFFF`.
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
    /// 8080 leaves SP indeterminate on reset; the reference uses
    /// `0xFFFF` so a stray `PUSH` lands in the high stack region.
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

    /// Mirrors both bus latches; executors must go through this so
    /// the address/data buffers don't go stale on the UI.
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

    /// Side-effect-free read for UI/disassembler; executors go through
    /// `bus_read*` / `bus_write*` / `fetch_opcode`.
    pub fn peek(&self, address: u16) -> u8 {
        self.memory.read(address)
    }

    pub fn step_instruction<B: PortBus>(
        &mut self,
        bus: &mut B,
    ) -> Result<InstructionOutcome, CoreError> {
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
            self.step_instruction(bus)?;
            executed += 1;
        }
        Ok(executed)
    }
}
