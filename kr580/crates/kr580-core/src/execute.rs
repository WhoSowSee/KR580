//! Instruction executor.
//!
//! Each instruction family lives in its own submodule under `execute/`.
//! The top-level `step_instruction` function owns the fetch / decode /
//! dispatch / accounting flow.

use crate::decode::{is_undocumented, opcode_timing, Cond};
use crate::error::{CoreError, DecodeError};
use crate::flags::Flags;
use crate::interrupt::handle_pending_interrupt;
use crate::io::IoBus;
use crate::state::Cpu8080State;

mod alu;
mod control;
mod data;
mod logic;
mod misc;
mod rotates;
mod stack;

impl Cpu8080State {
    /// Execute exactly one instruction (or service a pending interrupt).
    ///
    /// Returns the number of T-states consumed by the executed instruction.
    /// If the CPU is halted and no interrupt fires, returns `0` and leaves
    /// state untouched (caller should clock devices / wait).
    pub fn step_instruction<B: IoBus>(&mut self, bus: &mut B) -> Result<u32, CoreError> {
        // Honor `EI` latch from the *previous* instruction now that we are at
        // an instruction boundary.
        if self.interrupt_enable_pending {
            self.interrupt_enable = true;
            self.interrupt_enable_pending = false;
        }

        // Service interrupts before fetch. May un-halt the CPU.
        if let Some(t) = handle_pending_interrupt(self) {
            self.cycle_count = self.cycle_count.wrapping_add(t as u64);
            return Ok(t);
        }

        if self.halted {
            return Ok(0);
        }

        let pc_before = self.pc;
        let op = self.fetch_imm8();

        if is_undocumented(op) {
            self.halted = true; // stop execution, per quality gate
            return Err(CoreError::Decode {
                pc: pc_before,
                source: DecodeError::UndocumentedOpcode(op),
            });
        }

        let t = dispatch(self, bus, op)?;
        self.cycle_count = self.cycle_count.wrapping_add(t as u64);
        Ok(t)
    }

    /// Run for at least `target` T-states. Returns the actual count consumed.
    /// May overshoot by at most one instruction.
    pub fn run_for_t_states<B: IoBus>(
        &mut self,
        bus: &mut B,
        target: u64,
    ) -> Result<u64, CoreError> {
        let mut consumed: u64 = 0;
        while consumed < target {
            let n = self.step_instruction(bus)?;
            if n == 0 {
                // Halted with no interrupt — bail out so the caller decides.
                break;
            }
            consumed = consumed.saturating_add(n as u64);
        }
        Ok(consumed)
    }

    /// Run instructions until halted, an interrupt-deferred halt persists, or
    /// a maximum step count is exhausted. The cap exists to keep tests bounded.
    pub fn run_until_halt<B: IoBus>(
        &mut self,
        bus: &mut B,
        max_steps: u64,
    ) -> Result<u64, CoreError> {
        let mut steps = 0u64;
        while steps < max_steps && !self.halted {
            let _ = self.step_instruction(bus)?;
            steps += 1;
        }
        Ok(steps)
    }

    /// Single-T-state debug step. Per `prompt/02_cpu_core.md`, the UI may
    /// surface this for debugging but the UI must not implement tact logic.
    /// Devices observe `IN`/`OUT` at instruction boundaries only, so the bus
    /// is not touched here.
    pub fn step_tact(&mut self) -> Result<(), CoreError> {
        let phase = self.tact_phase.unwrap_or(0);
        // Conservative model: mark we executed one tact; on phase rollover at
        // 4 advance the architectural state by one no-op-equivalent. Real
        // T-state decomposition is opcode-specific and outside the scope.
        let next = phase.wrapping_add(1);
        self.tact_phase = Some(next);
        self.cycle_count = self.cycle_count.wrapping_add(1);
        Ok(())
    }
}

/// Evaluate a condition against the current flags.
#[inline]
pub(crate) fn cond_holds(c: Cond, f: &Flags) -> bool {
    match c {
        Cond::NZ => !f.z,
        Cond::Z => f.z,
        Cond::NC => !f.cy,
        Cond::C => f.cy,
        Cond::PO => !f.p,
        Cond::PE => f.p,
        Cond::P => !f.s,
        Cond::M => f.s,
    }
}

/// Top-level dispatch. Returns the T-state count consumed.
fn dispatch<B: IoBus>(cpu: &mut Cpu8080State, bus: &mut B, op: u8) -> Result<u32, CoreError> {
    let t = opcode_timing(op);
    match op {
        // 0x00 NOP
        0x00 => Ok(t.t_states_taken as u32),

        // 16-bit immediate / pair / memory
        0x01 | 0x11 | 0x21 | 0x31 => {
            data::lxi_rp(cpu, op);
            Ok(t.t_states_taken as u32)
        }
        0x02 | 0x12 => {
            data::stax_rp(cpu, op);
            Ok(t.t_states_taken as u32)
        }
        0x0A | 0x1A => {
            data::ldax_rp(cpu, op);
            Ok(t.t_states_taken as u32)
        }
        0x03 | 0x13 | 0x23 | 0x33 => {
            data::inx_rp(cpu, op);
            Ok(t.t_states_taken as u32)
        }
        0x0B | 0x1B | 0x2B | 0x3B => {
            data::dcx_rp(cpu, op);
            Ok(t.t_states_taken as u32)
        }
        0x09 | 0x19 | 0x29 | 0x39 => {
            data::dad_rp(cpu, op);
            Ok(t.t_states_taken as u32)
        }
        0x22 => {
            data::shld(cpu);
            Ok(t.t_states_taken as u32)
        }
        0x2A => {
            data::lhld(cpu);
            Ok(t.t_states_taken as u32)
        }
        0x32 => {
            data::sta(cpu);
            Ok(t.t_states_taken as u32)
        }
        0x3A => {
            data::lda(cpu);
            Ok(t.t_states_taken as u32)
        }
        0xEB => {
            data::xchg(cpu);
            Ok(t.t_states_taken as u32)
        }
        0xE3 => {
            data::xthl(cpu);
            Ok(t.t_states_taken as u32)
        }
        0xF9 => {
            data::sphl(cpu);
            Ok(t.t_states_taken as u32)
        }

        // 8-bit immediate / register loads
        0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x36 | 0x3E => {
            data::mvi(cpu, op);
            Ok(t.t_states_taken as u32)
        }

        // INR/DCR
        0x04 | 0x0C | 0x14 | 0x1C | 0x24 | 0x2C | 0x34 | 0x3C => {
            alu::inr(cpu, op);
            Ok(t.t_states_taken as u32)
        }
        0x05 | 0x0D | 0x15 | 0x1D | 0x25 | 0x2D | 0x35 | 0x3D => {
            alu::dcr(cpu, op);
            Ok(t.t_states_taken as u32)
        }

        // Rotates / flag ops
        0x07 => {
            rotates::rlc(cpu);
            Ok(t.t_states_taken as u32)
        }
        0x0F => {
            rotates::rrc(cpu);
            Ok(t.t_states_taken as u32)
        }
        0x17 => {
            rotates::ral(cpu);
            Ok(t.t_states_taken as u32)
        }
        0x1F => {
            rotates::rar(cpu);
            Ok(t.t_states_taken as u32)
        }
        0x27 => {
            misc::daa(cpu);
            Ok(t.t_states_taken as u32)
        }
        0x2F => {
            misc::cma(cpu);
            Ok(t.t_states_taken as u32)
        }
        0x37 => {
            misc::stc(cpu);
            Ok(t.t_states_taken as u32)
        }
        0x3F => {
            misc::cmc(cpu);
            Ok(t.t_states_taken as u32)
        }

        // MOV r,r' (and HLT at 0x76)
        0x76 => {
            misc::hlt(cpu);
            Ok(t.t_states_taken as u32)
        }
        0x40..=0x7F => {
            data::mov(cpu, op);
            Ok(t.t_states_taken as u32)
        }

        // ALU/Logic register-source family
        0x80..=0x87 => {
            alu::add(cpu, op);
            Ok(t.t_states_taken as u32)
        }
        0x88..=0x8F => {
            alu::adc(cpu, op);
            Ok(t.t_states_taken as u32)
        }
        0x90..=0x97 => {
            alu::sub(cpu, op);
            Ok(t.t_states_taken as u32)
        }
        0x98..=0x9F => {
            alu::sbb(cpu, op);
            Ok(t.t_states_taken as u32)
        }
        0xA0..=0xA7 => {
            logic::ana(cpu, op);
            Ok(t.t_states_taken as u32)
        }
        0xA8..=0xAF => {
            logic::xra(cpu, op);
            Ok(t.t_states_taken as u32)
        }
        0xB0..=0xB7 => {
            logic::ora(cpu, op);
            Ok(t.t_states_taken as u32)
        }
        0xB8..=0xBF => {
            alu::cmp(cpu, op);
            Ok(t.t_states_taken as u32)
        }

        // Immediate ALU/logic
        0xC6 => {
            alu::adi(cpu);
            Ok(t.t_states_taken as u32)
        }
        0xCE => {
            alu::aci(cpu);
            Ok(t.t_states_taken as u32)
        }
        0xD6 => {
            alu::sui(cpu);
            Ok(t.t_states_taken as u32)
        }
        0xDE => {
            alu::sbi(cpu);
            Ok(t.t_states_taken as u32)
        }
        0xE6 => {
            logic::ani(cpu);
            Ok(t.t_states_taken as u32)
        }
        0xEE => {
            logic::xri(cpu);
            Ok(t.t_states_taken as u32)
        }
        0xF6 => {
            logic::ori(cpu);
            Ok(t.t_states_taken as u32)
        }
        0xFE => {
            alu::cpi(cpu);
            Ok(t.t_states_taken as u32)
        }

        // Stack
        0xC1 | 0xD1 | 0xE1 | 0xF1 => {
            stack::pop(cpu, op);
            Ok(t.t_states_taken as u32)
        }
        0xC5 | 0xD5 | 0xE5 | 0xF5 => {
            stack::push(cpu, op);
            Ok(t.t_states_taken as u32)
        }

        // Control flow
        0xC3 => {
            control::jmp(cpu);
            Ok(t.t_states_taken as u32)
        }
        0xC2 | 0xCA | 0xD2 | 0xDA | 0xE2 | 0xEA | 0xF2 | 0xFA => {
            let cond = Cond::from_ccc((op >> 3) & 0b111);
            control::jcc(cpu, cond);
            // JMP cc is 10 either way; pick from timing.
            Ok(t.t_states_taken as u32)
        }
        0xCD => {
            control::call(cpu);
            Ok(t.t_states_taken as u32)
        }
        0xC4 | 0xCC | 0xD4 | 0xDC | 0xE4 | 0xEC | 0xF4 | 0xFC => {
            let cond = Cond::from_ccc((op >> 3) & 0b111);
            let taken = control::ccc(cpu, cond);
            Ok(if taken {
                t.t_states_taken as u32
            } else {
                t.t_states_not_taken as u32
            })
        }
        0xC9 => {
            control::ret(cpu);
            Ok(t.t_states_taken as u32)
        }
        0xC0 | 0xC8 | 0xD0 | 0xD8 | 0xE0 | 0xE8 | 0xF0 | 0xF8 => {
            let cond = Cond::from_ccc((op >> 3) & 0b111);
            let taken = control::rcc(cpu, cond);
            Ok(if taken {
                t.t_states_taken as u32
            } else {
                t.t_states_not_taken as u32
            })
        }
        0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF => {
            control::rst(cpu, op);
            Ok(t.t_states_taken as u32)
        }
        0xE9 => {
            control::pchl(cpu);
            Ok(t.t_states_taken as u32)
        }

        // I/O & interrupts
        0xDB => {
            misc::input(cpu, bus);
            Ok(t.t_states_taken as u32)
        }
        0xD3 => {
            misc::output(cpu, bus);
            Ok(t.t_states_taken as u32)
        }
        0xF3 => {
            misc::di(cpu);
            Ok(t.t_states_taken as u32)
        }
        0xFB => {
            misc::ei(cpu);
            Ok(t.t_states_taken as u32)
        }

        // The undocumented slots are filtered out at the dispatcher entry.
        _ => unreachable!("undocumented opcode {:#04X} reached dispatch", op),
    }
}
