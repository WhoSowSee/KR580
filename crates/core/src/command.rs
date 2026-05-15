use crate::{RegisterName, ValidationError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CoreCommand {
    ResetCpu,
    ResetRam,
    ResetRegisters,
    StepTact,
    RunForTStates(u64),
    StepInstruction,
    Run,
    Stop,
    ReadPort(u8),
    WritePort(u8, u8),
    SetRegister(RegisterName, u8),
    SetMemory(u16, u8),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CoreEvent {
    RegisterUpdated(RegisterName, u8),
    MemoryUpdated(u16, u8),
    InstructionBoundaryReached { pc: u16, t_states: u8 },
    ErrorRaised(String),
    HaltStateChanged(bool),
    ValidationFailed(ValidationError),
}
