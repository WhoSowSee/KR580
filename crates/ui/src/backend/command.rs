mod change;

pub use change::{ChangeDirection, CpuChange};

use crate::backend::AppError;
use crate::devices::printer::PrinterSettings;
use crate::devices::{DeviceSnapshot, NetworkMode};
use crate::persistence::ExportOptions;
use k580_core::{
    Cpu8080State, CpuMetadata, InstructionOutcome, Memory64K, RegisterName, TactOutcome,
};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RequestId(pub u64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppCommand {
    Request {
        id: RequestId,
        command: Box<AppCommand>,
    },
    Edit(Box<AppCommand>),
    RequestSnapshot,
    ResetCpu,
    ClearHalt,
    SetHalted(bool),
    ToggleHalt,
    LoadProgram(PathBuf),
    SaveProgram(PathBuf),
    LoadSubprogram {
        path: PathBuf,
        start: u16,
    },
    SaveSubprogram {
        path: PathBuf,
        start: u16,
        end: u16,
    },
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
    ApplyCpuDelta {
        metadata: CpuMetadata,
        memory: MemoryUpdate,
    },
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
    ToggleFloppyDebugBuffer,
    ClearHddBuffer,
    DetachHddFile,
    SetHddDebugBuffer(bool),
    ToggleHddDebugBuffer,
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
pub enum MemoryUpdate {
    Cells(Vec<(u16, u8)>),
    Replace(Memory64K),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppSnapshot {
    pub revision: u64,
    pub cpu: Cpu8080State,
    pub devices: DeviceSnapshot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommandResult {
    Completed,
    CpuChanged {
        revision: u64,
        change: Box<CpuChange>,
    },
    Superseded,
    SavedProgram {
        state: Box<Cpu8080State>,
    },
    LoadedProgram,
    SavedSubprogram {
        state: Box<Cpu8080State>,
    },
    LoadedSubprogram {
        end: u16,
    },
    Exported,
    Imported,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppEvent {
    SubprogramLoaded {
        path: PathBuf,
        start: u16,
        end: u16,
    },
    StateChanged(Box<AppSnapshot>),
    InstructionBoundaryReached(InstructionOutcome),
    TactAdvanced(TactOutcome),
    PortRead {
        port: u8,
        value: u8,
    },
    PortWritten {
        port: u8,
        value: u8,
    },
    HaltStateChanged(bool),
    ErrorRaised(AppError),
    Stopped,
    WorkerStopped,
    CommandFinished {
        id: RequestId,
        result: Result<CommandResult, AppError>,
    },
}
