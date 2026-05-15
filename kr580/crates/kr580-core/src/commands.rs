//! Command and event types used by the runtime to drive the core.
//!
//! These mirror the contract in `prompt/09_quality_gates.md`. The UI sends
//! `CoreCommand`s through a queue and observes `CoreEvent`s. Events are
//! notifications only — the UI must always be able to re-read authoritative
//! core state.

use crate::state::Reg8;
use serde::{Deserialize, Serialize};

/// Commands accepted by the core actor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoreCommand {
    /// Hard reset CPU registers, flags, PC, SP, interrupt and halt state.
    /// RAM is preserved.
    ResetCpu,
    /// Reset registers/flags/PC/SP only, without clearing interrupt state.
    ResetRegisters,
    /// Wipe the entire 64 KiB RAM to zero.
    ResetRam,
    /// Step one instruction.
    StepInstruction,
    /// Step one CPU T-state. Debug-only.
    StepTact,
    /// Run for at least N T-states (may overshoot by one instruction).
    RunForTStates(u64),
    /// Run instructions until halt or until `max_steps` is reached.
    Run {
        /// Hard cap on the number of instructions to execute.
        max_steps: u64,
    },
    /// Stop the running core (cooperative — observed at the next boundary).
    Stop,
    /// Set an 8-bit register.
    SetRegister {
        /// Register selector.
        reg: Reg8,
        /// New value.
        value: u8,
    },
    /// Write one byte of RAM.
    SetMemory {
        /// Address.
        addr: u16,
        /// New value.
        value: u8,
    },
    /// Bulk RAM load at `start`.
    LoadMemory {
        /// Start address.
        start: u16,
        /// Bytes to load.
        bytes: Vec<u8>,
    },
    /// Write a port value through the IO bus.
    WritePort {
        /// Port address.
        port: u8,
        /// Value.
        value: u8,
    },
    /// Request a port read result. The result is published as `PortRead`.
    ReadPort {
        /// Port address.
        port: u8,
    },
    /// Take a snapshot of the current core state. Reply via event.
    SaveSnapshot,
    /// Replace core state from a snapshot byte stream.
    LoadSnapshot {
        /// `.580` payload bytes.
        bytes: Vec<u8>,
    },
}

/// Events published by the core.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoreEvent {
    /// Instruction boundary reached.
    InstructionExecuted {
        /// Program counter *before* the instruction.
        pc: u16,
        /// T-states consumed.
        t_states: u32,
    },
    /// Halt state changed.
    HaltChanged(bool),
    /// Memory range was updated.
    MemoryUpdated {
        /// Start.
        start: u16,
        /// Length.
        len: u16,
    },
    /// Register value changed.
    RegisterUpdated {
        /// Register.
        reg: Reg8,
        /// New value.
        value: u8,
    },
    /// Result of a `ReadPort` command.
    PortRead {
        /// Port.
        port: u8,
        /// Value read.
        value: u8,
    },
    /// Error raised by the core.
    Error(String),
    /// Snapshot saved.
    SnapshotSaved {
        /// `.580` payload bytes.
        bytes: Vec<u8>,
    },
    /// Snapshot loaded.
    SnapshotLoaded,
    /// Device status changed (free-form for now; structured per device in
    /// `kr580-devices`).
    DeviceStatusChanged(String),
}
