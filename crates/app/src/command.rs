use crate::AppError;
use k580_core::{Cpu8080State, InstructionOutcome, RegisterName, TactOutcome};
use k580_devices::DeviceSnapshot;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppCommand {
    ResetCpu,
    LoadSnapshot(PathBuf),
    SaveSnapshot(PathBuf),
    LoadSubprogram {
        path: PathBuf,
        base_address: u16,
    },
    ResetRam,
    StepTact,
    RunForTStates(u64),
    StepInstruction,
    Run,
    Stop,
    /// Reconfigure the inter-instruction delay used by the paced
    /// `Run` loop. The UI exposes this through the speed slider in
    /// the left-hand schematic panel: smaller intervals make the
    /// program rip through the listing, larger ones let a human eye
    /// follow each PC update. Has no effect while `running` is
    /// false; the next `Run` picks the new value up.
    SetStepInterval(Duration),
    ReadPort(u8),
    WritePort(u8, u8),
    SetRegister(RegisterName, u8),
    SetPc(u16),
    SetMemory(u16, u8),
    ExportTxt(PathBuf),
    ExportXlsx(PathBuf),
    ImportTxt(PathBuf),
    ImportXlsx(PathBuf),
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
