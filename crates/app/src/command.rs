use crate::AppError;
use k580_core::{Cpu8080State, InstructionOutcome, RegisterName, TactOutcome};
use k580_devices::DeviceSnapshot;
use k580_persistence::{ExportOptions, Snapshot580Flavour};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppCommand {
    ResetCpu,
    /// Clears the halt flip-flop only. Leaves PC, registers, flags, SP,
    /// RAM, `cycle_count` intact. No-op on a non-halted CPU.
    ClearHalt,
    SetHalted(bool),
    LoadSnapshot(PathBuf),
    /// Probes a `.580` and dispatches to the modern (K580 v1) or legacy
    /// (65 549-byte flat) deserializer. Emits `SnapshotFlavourLoaded`.
    LoadAnySnapshot(PathBuf),
    SaveSnapshot(PathBuf),
    /// Legacy 65 549-byte `.580` (RAM + PC only).
    LoadLegacySnapshot(PathBuf),
    SaveLegacySnapshot(PathBuf),
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
    SetStepInterval(Duration),
    SetRunMode(RunMode),
    ReadPort(u8),
    WritePort(u8, u8),
    SetRegister(RegisterName, u8),
    SetPc(u16),
    SetMemory(u16, u8),
    /// Replaces the entire CPU snapshot in one shot. Used by Ctrl+Z to
    /// rewind. Stops the run loop the same way the reset commands do.
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
    Shutdown,
}

/// `Paced` runs one instruction per `tick()` with a snapshot per step.
/// `Burst` runs a tight loop bounded by `slice` wall-time and publishes
/// one coalesced snapshot per tick. `slice` doubles as the `Stop`
/// responsiveness floor.
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
    SnapshotFlavourLoaded(Snapshot580Flavour),
    ErrorRaised(AppError),
    Stopped,
}
