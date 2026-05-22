use crate::{AppCommand, AppError, AppEvent, AppSnapshot};
use k580_core::{Cpu8080State, PortBus};
use k580_devices::IoBus;
use k580_persistence::{
    ExportModel, Exporters, Importers, Snapshot580Serializer, SubprogramSerializer,
};
use std::time::Duration;

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
}

impl Default for Emulator {
    fn default() -> Self {
        Self {
            cpu: Cpu8080State::default(),
            bus: IoBus::default(),
            running: false,
            instructions_since_run: 0,
            step_interval: DEFAULT_STEP_INTERVAL,
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
        events.push(AppEvent::StateChanged(Box::new(self.snapshot())));
        events
    }

    fn apply(&mut self, command: AppCommand) -> Result<Vec<AppEvent>, AppError> {
        let mut events = Vec::new();
        match command {
            AppCommand::ResetCpu => {
                self.cpu.reset_cpu();
                self.running = false;
                self.instructions_since_run = 0;
            }
            AppCommand::ResetRam => self.cpu.reset_ram(),
            AppCommand::SetRegister(register, value) => self.cpu.set_register(register, value),
            AppCommand::SetPc(address) => self.cpu.pc = address,
            AppCommand::SetMemory(address, value) => self.cpu.set_memory(address, value),
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
