use crate::{AppCommand, AppError, AppEvent, AppSnapshot, RunMode};
use k580_core::{Cpu8080State, PortBus};
use k580_devices::IoBus;
use k580_persistence::{
    ExportModel, Exporters, Importers, Snapshot580Serializer, SubprogramSerializer,
};
use std::time::{Duration, Instant};

/// Default delay between instructions while `Run` is armed. The user
/// can change it through the speed slider; this value is what the
/// emulator boots with so a freshly opened program runs at a
/// human-readable pace instead of finishing in microseconds.
pub const DEFAULT_STEP_INTERVAL: Duration = Duration::from_millis(100);

/// Hard cap on how many instructions a single `Run` session is
/// allowed to execute before the worker auto-pauses. Mirrors the old
/// `run_until_halt(100_000)` budget so we keep the same protection
/// against runaway programs that never reach `HLT`. The user can
/// always re-arm `Run` to keep going.
const MAX_INSTRUCTIONS_PER_RUN: u64 = 100_000;

#[derive(Debug)]
pub struct Emulator {
    cpu: Cpu8080State,
    bus: IoBus,
    running: bool,
    /// Number of instructions executed since the most recent `Run`
    /// arm. Reset on every `Run`/`Stop`/`ResetCpu` so the budget
    /// applies per session, not per process lifetime.
    instructions_since_run: u64,
    step_interval: Duration,
    /// Current run mode. `Paced` is the default — one instruction
    /// per worker `tick()`, one `StateChanged` per instruction.
    /// `Burst { slice }` makes `tick()` churn instructions in a
    /// tight loop for up to `slice` wall-time and publish a single
    /// coalesced snapshot, so the worker stops paying the
    /// per-instruction timer + crossbeam + redraw overhead the user
    /// observed as "Максимум по итогу не быстрее Высоко".
    run_mode: RunMode,
}

impl Default for Emulator {
    fn default() -> Self {
        Self {
            cpu: Cpu8080State::default(),
            bus: IoBus::default(),
            running: false,
            instructions_since_run: 0,
            step_interval: DEFAULT_STEP_INTERVAL,
            run_mode: RunMode::Paced,
        }
    }
}

impl Emulator {
    pub fn new(cpu: Cpu8080State, bus: IoBus) -> Self {
        Self {
            cpu,
            bus,
            running: false,
            instructions_since_run: 0,
            step_interval: DEFAULT_STEP_INTERVAL,
            run_mode: RunMode::Paced,
        }
    }

    pub fn cpu(&self) -> &Cpu8080State {
        &self.cpu
    }

    pub fn bus(&self) -> &IoBus {
        &self.bus
    }

    pub fn bus_mut(&mut self) -> &mut IoBus {
        &mut self.bus
    }

    /// Whether the paced `Run` loop is currently armed. The actor
    /// thread checks this between `recv` calls to decide whether to
    /// schedule a tick.
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Current inter-instruction delay used by `Run`. The actor
    /// turns this into a `crossbeam_channel::after` deadline.
    pub fn step_interval(&self) -> Duration {
        self.step_interval
    }

    /// Current run mode. The actor reads this to decide between the
    /// `step_interval` deadline (Paced) and the `slice` deadline
    /// (Burst), and `tick()` reads it to decide between one
    /// instruction and a tight inner loop.
    pub fn run_mode(&self) -> RunMode {
        self.run_mode
    }

    pub fn snapshot(&self) -> AppSnapshot {
        AppSnapshot {
            cpu: self.cpu.clone(),
            devices: self.bus.snapshot(),
        }
    }

    pub fn handle_command(&mut self, command: AppCommand) -> Vec<AppEvent> {
        let result = self.apply(command);
        let mut events = match result {
            Ok(events) => events,
            Err(error) => vec![AppEvent::ErrorRaised(error)],
        };
        events.push(AppEvent::StateChanged(Box::new(self.snapshot())));
        events
    }

    /// Drive one step of the paced `Run` loop. Returns the events
    /// the actor should publish, including the post-step snapshot.
    /// No-op when `running` is false or the per-session budget is
    /// exhausted — the actor still gets a snapshot so the UI sees
    /// the auto-pause status update.
    ///
    /// The body is split between `Paced` (one instruction, one
    /// snapshot, one boundary event — the original behaviour) and
    /// `Burst { slice }` (tight inner loop bounded by wall-time and
    /// the per-session budget, only the final snapshot is
    /// published). Burst skips `InstructionBoundaryReached` for the
    /// in-flight instructions on purpose: the user opted into
    /// "Максимум" precisely to stop paying the per-step UI cost,
    /// and flooding the channel with thousands of boundary events
    /// would re-introduce that cost on the iced side.
    pub fn tick(&mut self) -> Vec<AppEvent> {
        let mut events = Vec::new();
        if !self.running {
            // Defensive: if the actor dispatched a tick after
            // `Stop`, just publish the current snapshot. The
            // alternative — silently dropping the tick — would
            // leave the UI without an obvious recovery path if a
            // race ever sneaks one through.
            events.push(AppEvent::StateChanged(Box::new(self.snapshot())));
            return events;
        }
        if self.cpu.halted {
            self.running = false;
            events.push(AppEvent::HaltStateChanged(true));
            events.push(AppEvent::Stopped);
            events.push(AppEvent::StateChanged(Box::new(self.snapshot())));
            return events;
        }
        if self.instructions_since_run >= MAX_INSTRUCTIONS_PER_RUN {
            self.running = false;
            events.push(AppEvent::Stopped);
            events.push(AppEvent::StateChanged(Box::new(self.snapshot())));
            return events;
        }

        match self.run_mode {
            RunMode::Paced => self.tick_paced(&mut events),
            RunMode::Burst { slice } => self.tick_burst(slice, &mut events),
        }

        events.push(AppEvent::StateChanged(Box::new(self.snapshot())));
        events
    }

    /// One-instruction-per-tick body. Mirrors the original `tick()`
    /// implementation: emits an `InstructionBoundaryReached` per
    /// step so the UI can update its per-instruction counters and
    /// the highlighted memory row walks one cell at a time.
    fn tick_paced(&mut self, events: &mut Vec<AppEvent>) {
        match self.cpu.step_instruction(&mut self.bus) {
            Ok(outcome) => {
                self.instructions_since_run += 1;
                events.push(AppEvent::InstructionBoundaryReached(outcome));
                if self.cpu.halted {
                    self.running = false;
                    events.push(AppEvent::HaltStateChanged(true));
                    events.push(AppEvent::Stopped);
                }
            }
            Err(error) => {
                self.running = false;
                events.push(AppEvent::ErrorRaised(error.into()));
                events.push(AppEvent::Stopped);
            }
        }
    }

    /// Burst body — inner loop runs instructions back-to-back until
    /// `slice` wall-time elapses, the per-session budget is
    /// exhausted, the CPU halts, or an instruction errors. The
    /// instruction count is bumped per step (so the budget still
    /// applies and `cycle_count` on the published snapshot is
    /// correct), but no per-step events are emitted: the UI sees
    /// only the final snapshot for this slice. Wall-time is
    /// re-checked every 64 instructions so the loop doesn't spend
    /// the slice burning syscalls on `Instant::now()` for short
    /// instructions.
    fn tick_burst(&mut self, slice: Duration, events: &mut Vec<AppEvent>) {
        // 0-length slice would degenerate to "do nothing forever";
        // floor at the same 1 ms the actor's `SetStepInterval` does.
        let slice = slice.max(Duration::from_millis(1));
        let started = Instant::now();
        let mut since_check: u32 = 0;
        loop {
            if self.instructions_since_run >= MAX_INSTRUCTIONS_PER_RUN {
                self.running = false;
                events.push(AppEvent::Stopped);
                return;
            }
            match self.cpu.step_instruction(&mut self.bus) {
                Ok(_outcome) => {
                    self.instructions_since_run += 1;
                    if self.cpu.halted {
                        self.running = false;
                        events.push(AppEvent::HaltStateChanged(true));
                        events.push(AppEvent::Stopped);
                        return;
                    }
                }
                Err(error) => {
                    self.running = false;
                    events.push(AppEvent::ErrorRaised(error.into()));
                    events.push(AppEvent::Stopped);
                    return;
                }
            }
            since_check = since_check.wrapping_add(1);
            // 64 is a balance: small enough that a single slow
            // instruction can't blow the slice by much, large
            // enough that `Instant::now()` doesn't dominate the
            // hot loop. Empirically 8080 instructions land in the
            // 4..18 cycle range, so 64 instructions ≈ a few hundred
            // T-states, well under any reasonable slice.
            if since_check >= 64 {
                since_check = 0;
                if started.elapsed() >= slice {
                    return;
                }
            }
        }
    }

    fn apply(&mut self, command: AppCommand) -> Result<Vec<AppEvent>, AppError> {
        let mut events = Vec::new();
        match command {
            AppCommand::ResetCpu => {
                // If the user clicked "Сброс регистров" while a
                // paced or burst Run was armed, the previous
                // implementation flipped `running = false` silently
                // and never published `Stopped`. The UI's
                // `consume_event` only clears `DesktopApp::running`
                // on `Stopped` / `HaltStateChanged(true)` /
                // `ErrorRaised`, so the play/pause toggle stayed
                // red and the next Tick kept calling
                // `follow_pc_during_run` against a CPU that the
                // worker had already deactivated. The user reported
                // this as «программа не перестаёт выполняться при
                // сбросе регистров». Publishing `Stopped` (and
                // `HaltStateChanged(false)`, since reset clears the
                // halt bit even if the CPU was halted before)
                // brings every UI surface back into agreement with
                // the worker's view of the world.
                let was_running = self.running;
                let was_halted_before = self.cpu.halted;
                self.cpu.reset_cpu();
                self.running = false;
                self.instructions_since_run = 0;
                if was_running {
                    events.push(AppEvent::Stopped);
                }
                if was_halted_before {
                    events.push(AppEvent::HaltStateChanged(false));
                }
            }
            AppCommand::ClearHalt => {
                // Strictly weaker reset than `ResetCpu`: snap the
                // halt flip-flop back to `false` and leave PC,
                // registers, flags, SP, RAM, and `cycle_count`
                // alone. The user just wants the run-block lifted —
                // typically because they reached HLT as the 8080
                // "wait for interrupt" idiom and want the next
                // instruction (the byte at `pc`, which already
                // sits one past HLT) to keep executing as if an
                // interrupt had arrived. We also disarm the run
                // loop and publish `Stopped` for symmetry with
                // every other halt-clearing command: a UI that has
                // its play/pause toggle out of sync with the
                // worker is the same UX bug the `ResetCpu` block
                // documents above, only landing through a different
                // gesture. The `HaltStateChanged(false)` event is
                // gated on the bit actually flipping so the
                // command is a true no-op on a non-halted CPU
                // instead of bouncing the UI's halt notice through
                // a clear/re-arm cycle.
                let was_running = self.running;
                let was_halted_before = self.cpu.halted;
                if was_halted_before {
                    self.cpu.halted = false;
                }
                self.running = false;
                self.instructions_since_run = 0;
                if was_running {
                    events.push(AppEvent::Stopped);
                }
                if was_halted_before {
                    events.push(AppEvent::HaltStateChanged(false));
                }
            }
            AppCommand::ResetRam => {
                // Wiping RAM under a running program would let the
                // worker keep stepping into a sea of zero bytes
                // (NOPs) until either the per-session budget hit
                // or the user clicked Pause manually — neither
                // matches «очистил память — программа должна
                // прекратиться». Stop the worker first so the same
                // gesture that erases code also yields control
                // back to the UI; the `Stopped` event lets the
                // play/pause toggle revert to its idle state.
                //
                // We also lift the halt flip-flop here, mirroring
                // `ResetCpu` and `ClearHalt`. The user's mental
                // model is "сброс — это сброс": after wiping the
                // program, there is no HLT instruction left to be
                // halted on, so the bit becoming `true` would be a
                // pure UI artifact (the post-HLT run-block keeps the
                // execution chips greyed even though the program
                // that triggered it is gone). `HaltStateChanged` is
                // gated on the bit actually flipping so a non-halted
                // CPU does not bounce the halt notice through a
                // clear/re-arm cycle.
                let was_running = self.running;
                let was_halted_before = self.cpu.halted;
                self.cpu.reset_ram();
                if was_halted_before {
                    self.cpu.halted = false;
                }
                if was_running {
                    self.running = false;
                    events.push(AppEvent::Stopped);
                }
                if was_halted_before {
                    events.push(AppEvent::HaltStateChanged(false));
                }
            }
            AppCommand::SetRegister(register, value) => self.cpu.set_register(register, value),
            AppCommand::SetPc(address) => self.cpu.pc = address,
            AppCommand::SetMemory(address, value) => self.cpu.set_memory(address, value),
            AppCommand::ApplyCpuState(state) => {
                // Replace the whole CPU snapshot in one shot. Same
                // run-loop hygiene the reset commands implement: an
                // armed `Run` would race the next `step_instruction`
                // against the freshly restored bytes, so we disarm
                // and publish `Stopped` for the UI's play/pause
                // toggle. We also reset the per-arming budget so a
                // subsequent `Run` gets the full 100k window — the
                // user just rewound to a clean state, anything else
                // would carry stale bookkeeping forward.
                let was_running = self.running;
                self.cpu = *state;
                self.running = false;
                self.instructions_since_run = 0;
                if was_running {
                    events.push(AppEvent::Stopped);
                }
                // Halt bit could go either way: the snapshot we just
                // restored may have been taken on a halted CPU
                // (rewinding past a HLT) or on a running one
                // (rewinding past a reset that itself cleared halt).
                // The trailing `if self.cpu.halted` block at the end
                // of `apply` covers the "now halted" case; we only
                // need to publish the unhalted transition explicitly,
                // so the UI clears the halt notice when the user
                // rewinds back to a pre-HLT state.
                if !self.cpu.halted {
                    events.push(AppEvent::HaltStateChanged(false));
                }
            }
            AppCommand::StepInstruction => {
                let outcome = self.cpu.step_instruction(&mut self.bus)?;
                events.push(AppEvent::InstructionBoundaryReached(outcome));
            }
            AppCommand::StepTact => {
                let outcome = self.cpu.step_tact(&mut self.bus)?;
                events.push(AppEvent::TactAdvanced(outcome));
            }
            AppCommand::RunForTStates(t_states) => {
                self.cpu.run_for_t_states(&mut self.bus, t_states)?
            }
            AppCommand::Run => {
                // Arm the paced run loop. The heavy lifting happens
                // in `tick()`, which the actor calls on every
                // `step_interval` deadline. We only flip the flag
                // and reset the per-session counter here so the
                // command-handling path stays cheap and the UI
                // gets its post-arm snapshot immediately.
                if !self.cpu.halted {
                    self.running = true;
                    self.instructions_since_run = 0;
                }
            }
            AppCommand::Stop => {
                self.running = false;
                events.push(AppEvent::Stopped);
            }
            AppCommand::SetStepInterval(interval) => {
                // Floor at 1 ms so a runaway slider can't turn the
                // tick channel into a busy loop. The UI never asks
                // for zero, but treating an out-of-range value as
                // "fastest sensible speed" is friendlier than
                // erroring out and leaves room for future
                // free-form input.
                self.step_interval = interval.max(Duration::from_millis(1));
            }
            AppCommand::SetRunMode(mode) => {
                // Switch between paced (one instruction per
                // `step_interval`, one snapshot per step) and burst
                // (tight inner loop bounded by `slice`, one
                // coalesced snapshot per slice). The actor reads
                // `run_mode()` between selects, so a switch lands
                // on the next iteration without restarting the
                // armed `Run`.
                self.run_mode = match mode {
                    RunMode::Burst { slice } => RunMode::Burst {
                        slice: slice.max(Duration::from_millis(1)),
                    },
                    paced => paced,
                };
            }
            AppCommand::ReadPort(port) => {
                let value = self.bus.input(port)?;
                events.push(AppEvent::PortRead { port, value });
            }
            AppCommand::WritePort(port, value) => {
                self.bus.output(port, value)?;
                events.push(AppEvent::PortWritten { port, value });
            }
            AppCommand::SaveSnapshot(path) => {
                std::fs::write(path, Snapshot580Serializer::to_bytes(&self.cpu))?
            }
            AppCommand::LoadSnapshot(path) => {
                self.cpu = Snapshot580Serializer::from_bytes(&std::fs::read(path)?)?;
            }
            AppCommand::LoadAnySnapshot(path) => {
                // Auto-detect path: probe the bytes and dispatch to
                // the matching deserializer. Used by double-click /
                // `argv[1]` / "Открыть…" gestures where the UI
                // cannot tell ahead of time which `.580` flavour
                // landed on disk. The worker emits the resolved
                // flavour back to the UI so a subsequent "Сохранить"
                // round-trips into the same format the user opened
                // with — without that hint a legacy file would
                // silently re-encode as modern on the next save.
                let bytes = std::fs::read(path)?;
                let (cpu, flavour) = Snapshot580Serializer::from_any_bytes(&bytes)?;
                self.cpu = cpu;
                events.push(AppEvent::SnapshotFlavourLoaded(flavour));
            }
            AppCommand::SaveLegacySnapshot(path) => {
                std::fs::write(path, Snapshot580Serializer::to_legacy_bytes(&self.cpu))?
            }
            AppCommand::LoadLegacySnapshot(path) => {
                self.cpu = Snapshot580Serializer::from_legacy_bytes(&std::fs::read(path)?)?;
            }
            AppCommand::LoadSubprogram { path, base_address } => {
                let subprogram = SubprogramSerializer::load_file(path, base_address)?;
                SubprogramSerializer::load_into_state(&mut self.cpu, &subprogram)?;
            }
            AppCommand::ExportTxt(path) => Exporters::write_txt(path, &self.export_model())?,
            AppCommand::ExportXlsx(path) => Exporters::write_xlsx(path, &self.export_model())?,
            AppCommand::ImportTxt(path) => {
                let model = Importers::read_txt(path)?;
                model.apply_to(&mut self.cpu)?;
            }
            AppCommand::ImportXlsx(path) => {
                let model = Importers::read_xlsx(path)?;
                model.apply_to(&mut self.cpu)?;
            }
            AppCommand::Shutdown => {
                self.running = false;
                events.push(AppEvent::Stopped);
            }
        }
        if self.cpu.halted {
            events.push(AppEvent::HaltStateChanged(true));
        }
        Ok(events)
    }

    fn export_model(&self) -> ExportModel {
        ExportModel::from_cpu(&self.cpu)
    }
}
