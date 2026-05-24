use crate::AppError;
use k580_core::{Cpu8080State, InstructionOutcome, RegisterName, TactOutcome};
use k580_devices::DeviceSnapshot;
use k580_persistence::Snapshot580Flavour;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppCommand {
    ResetCpu,
    /// Clear the halt flip-flop without touching anything else: PC,
    /// registers, flags, SP, RAM, and `cycle_count` all stay where
    /// they were when HLT was executed. The UI exposes this through
    /// "Сбросить флаг HLT" in the МП-Система menu and through the
    /// register-editor toggle on the halt bit; the contract is "the
    /// least destructive way to leave halt-state". Useful when the
    /// program reached HLT as a way of waiting for an interrupt
    /// (the classic 8080 idiom) and the user wants execution to
    /// continue with the very next instruction without rewinding
    /// the machine. A no-op on a CPU that is not halted; emits
    /// `HaltStateChanged(false)` only when the bit actually
    /// flipped, mirroring what `ResetCpu` does on the same path.
    ClearHalt,
    LoadSnapshot(PathBuf),
    /// Load a `.580` file of unknown flavour: probes the bytes and
    /// dispatches to the modern (K580 v1) or legacy (65 549-byte flat)
    /// deserializer based on the magic / length. Used by the
    /// double-click / `argv[1]` path, where the UI cannot tell ahead
    /// of time which format the user dropped on us — both flavours
    /// share the `.580` extension. Emits `SnapshotFlavourLoaded` so
    /// the UI can route a subsequent "Сохранить" gesture to the
    /// matching serializer instead of silently round-tripping a
    /// legacy file into the modern format.
    LoadAnySnapshot(PathBuf),
    SaveSnapshot(PathBuf),
    /// Load a legacy 65 549-byte `.580` produced by the original
    /// emulator the project was based on (raw RAM + 13-byte trailer
    /// carrying PC and an `FF FF` end marker; no registers, flags, or
    /// SP). Replaces the live state with RAM from the file and the
    /// recovered PC; everything else comes back as default. See
    /// `Snapshot580Serializer::from_legacy_bytes`.
    LoadLegacySnapshot(PathBuf),
    /// Write the current CPU state out in the legacy 65 549-byte
    /// layout. RAM and PC round-trip; flags, registers, SP, halt,
    /// and timing are dropped because the reference format simply
    /// does not have slots for them. See
    /// `Snapshot580Serializer::to_legacy_bytes`.
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
    /// A `.580` file was just loaded through the auto-detect path
    /// (`AppCommand::LoadAnySnapshot`) and the worker resolved it to
    /// this flavour. The UI consumes this to decide which "current
    /// path" slot to populate (`current_snapshot_path` for Modern,
    /// `current_legacy_snapshot_path` for Legacy) so Ctrl+S /
    /// Ctrl+Alt+S routes a subsequent save through the matching
    /// serializer instead of silently re-encoding a legacy file in
    /// the modern format.
    SnapshotFlavourLoaded(Snapshot580Flavour),
    ErrorRaised(AppError),
    Stopped,
}
