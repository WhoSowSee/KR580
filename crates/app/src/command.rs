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
    /// Switch the worker between paced and burst run modes.
    ///
    /// - `RunMode::Paced` is the default: `tick()` executes one
    ///   instruction every `step_interval`, publishes a snapshot,
    ///   and the UI renders each step. Use this for the Slow /
    ///   Medium / High tiers, where the user wants to *see* the
    ///   program move.
    /// - `RunMode::Burst { slice }` tells `tick()` to keep stepping
    ///   in a tight loop for up to `slice` of wall time (or until
    ///   halt / budget exhaustion / error) and publish only the
    ///   final snapshot. The actor still sets the timer to `slice`
    ///   so a `Stop` press lands within one slice. Use this for the
    ///   Max tier, where the user explicitly chose "доведи программу
    ///   до конца, мне не нужно смотреть на каждый шаг".
    SetRunMode(RunMode),
    ReadPort(u8),
    WritePort(u8, u8),
    SetRegister(RegisterName, u8),
    SetPc(u16),
    SetMemory(u16, u8),
    /// Replace the entire CPU state (registers, PC, SP, flags,
    /// memory, halt bit, cycle counter — everything inside
    /// `Cpu8080State`) with the supplied snapshot. Used by the UI's
    /// undo/redo stack: the stack stores a `Cpu8080State` snapshot
    /// taken *before* every mutating gesture (`SetMemory`,
    /// `SetRegister`, `ResetCpu`, `ResetRam`, snapshot/import loads),
    /// and Ctrl+Z replays the most recent one through this command.
    /// We model it as a single command rather than fanning the diff
    /// out into individual `SetMemory` writes because (a) destructive
    /// gestures touch all 64 KiB and 8 registers at once, and (b)
    /// the worker can swap in the snapshot in one go without holding
    /// a 64K-entry undo journal in memory per step.
    ///
    /// Stops the run loop the same way the reset commands do:
    /// applying a fresh state under a live worker would race the
    /// next `step_instruction` against the user's restored bytes.
    /// The handler emits `Stopped` when `running` was true, mirroring
    /// the `ResetCpu` / `ResetRam` contract so the UI's play/pause
    /// toggle returns to its idle state.
    ApplyCpuState(Box<Cpu8080State>),
    ExportTxt(PathBuf),
    ExportXlsx(PathBuf),
    ImportTxt(PathBuf),
    ImportXlsx(PathBuf),
    Shutdown,
}

/// How the paced `Run` loop dispatches work to the CPU.
///
/// `Paced` keeps the original behaviour: one instruction per worker
/// `tick()`, one `StateChanged` event per instruction, the UI renders
/// every step. `Burst` is the new "Максимум" path — the worker runs
/// instructions in a tight loop for up to `slice` wall-time and
/// publishes a single coalesced snapshot per tick, so the user is
/// no longer paying the round-trip cost of one timer fire + one
/// crossbeam send + one iced redraw per instruction. The wall-time
/// budget is what bounds the worker's responsiveness to a `Stop`
/// press: the actor's `select!` re-arms the timer with the same
/// slice, so a press lands within at most `slice` even when the
/// program is in the middle of a 100k-instruction sprint.
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
