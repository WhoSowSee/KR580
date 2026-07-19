use crate::backend::AppError;
use crate::devices::printer::PrinterSettings;
use crate::devices::{DeviceSnapshot, NetworkMode};
use crate::persistence::ExportOptions;
use k580_core::{Cpu8080State, InstructionOutcome, RegisterName, TactOutcome};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppCommand {
    ResetCpu,
    ClearHalt,
    SetHalted(bool),
    LoadProgram(PathBuf),
    SaveProgram(PathBuf),
    ResetRam,
    StepTact,
    RunForTStates(u64),
    StepInstruction,
    Run,
    Stop,
    SetStepInterval(Duration),
    SetRunMode(RunMode),
    ReadPort(u8),
    WritePort(u8, u8),
    SetRegister(RegisterName, u8),
    SetPc(u16),
    SetMemory(u16, u8),
    SetMemoryBlock {
        start: u16,
        values: Vec<u8>,
    },
    ApplyCpuState(Box<Cpu8080State>),
    ExportTxt(PathBuf),
    ExportXlsx(PathBuf),
    ExportTxtWithOptions(PathBuf, ExportOptions),
    ExportXlsxWithOptions(PathBuf, ExportOptions),
    ImportTxt(PathBuf),
    ImportXlsx(PathBuf),
    ImportTxtSection(PathBuf, String),
    ImportXlsxSheet(PathBuf, String),
    ClearMonitorBuffer,
    ClearFloppyBuffer,
    AttachFloppyImage(PathBuf),
    DetachFloppyImage,
    SetFloppyDebugBuffer(bool),
    ClearHddBuffer,
    DetachHddFile,
    SetHddDebugBuffer(bool),
    AttachHddFile(PathBuf),
    ConfigureNetwork {
        mode: NetworkMode,
        host: String,
        port: u16,
    },
    ClearNetworkBuffers,
    ClearPrinterBuffer,
    PrintPrinterNative(Option<PrinterSettings>),
    Shutdown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunMode {
    Paced,
    Burst { slice: Duration },
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
