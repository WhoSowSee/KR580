use crate::AppError;
use k580_core::{Cpu8080State, InstructionOutcome, RegisterName, TactOutcome};
use k580_devices::DeviceSnapshot;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppCommand {
    ResetCpu,
    LoadSnapshot(PathBuf),
    SaveSnapshot(PathBuf),
    LoadSubprogram { path: PathBuf, base_address: u16 },
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
    SetPc(u16),
    SetMemory(u16, u8),
    ExportTxt(PathBuf),
    ExportXlsx(PathBuf),
    ExportDocx(PathBuf),
    Shutdown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppSnapshot {
    pub cpu: Cpu8080State,
    pub devices: DeviceSnapshot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppEvent {
    StateChanged(Box<AppSnapshot>),
    InstructionBoundaryReached(InstructionOutcome),
    TactAdvanced(TactOutcome),
    PortRead { port: u8, value: u8 },
    PortWritten { port: u8, value: u8 },
    HaltStateChanged(bool),
    ErrorRaised(AppError),
    Stopped,
}
