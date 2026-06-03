mod tick;

use crate::{AppCommand, AppError, AppEvent, AppSnapshot, RunMode};
use k580_core::{Cpu8080State, PortBus};
use k580_devices::IoBus;
use k580_persistence::{
    ExportModel, Exporters, Importers, Snapshot580Serializer, SubprogramSerializer,
};
use std::time::Duration;

pub const DEFAULT_STEP_INTERVAL: Duration = Duration::from_millis(100);

pub(super) const MAX_INSTRUCTIONS_PER_RUN: u64 = 100_000;

#[derive(Debug)]
pub struct Emulator {
    pub(super) cpu: Cpu8080State,
    pub(super) bus: IoBus,
    pub(super) io_runtime: tokio::runtime::Runtime,
    pub(super) running: bool,
    /// Reset on every `Run`/`Stop`/`ResetCpu` so the budget is per-session.
    pub(super) instructions_since_run: u64,
    pub(super) step_interval: Duration,
    pub(super) run_mode: RunMode,
}

impl Default for Emulator {
    fn default() -> Self {
        Self {
            cpu: Cpu8080State::default(),
            bus: IoBus::default(),
            io_runtime: tokio::runtime::Runtime::new().expect("storage I/O runtime"),
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
            io_runtime: tokio::runtime::Runtime::new().expect("storage I/O runtime"),
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

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn step_interval(&self) -> Duration {
        self.step_interval
    }

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

    fn apply(&mut self, command: AppCommand) -> Result<Vec<AppEvent>, AppError> {
        let mut events = Vec::new();
        match command {
            // Reset/halt commands all share the same hygiene: stop the run
            // loop, lift the halt flip-flop, and emit `Stopped` /
            // `HaltStateChanged` only on actual transitions so the UI's
            // play/pause and halt notice stay in sync with the worker.
            AppCommand::ResetCpu => {
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
            AppCommand::SetHalted(target) => {
                let was_running = self.running;
                let was_halted_before = self.cpu.halted;
                if was_halted_before != target {
                    self.cpu.halted = target;
                }
                self.running = false;
                self.instructions_since_run = 0;
                if was_running {
                    events.push(AppEvent::Stopped);
                }
                if was_halted_before != target {
                    events.push(AppEvent::HaltStateChanged(target));
                }
            }
            AppCommand::ResetRam => {
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
                let was_running = self.running;
                self.cpu = *state;
                self.running = false;
                self.instructions_since_run = 0;
                if was_running {
                    events.push(AppEvent::Stopped);
                }
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
                self.step_interval = interval.max(Duration::from_millis(1));
            }
            AppCommand::SetRunMode(mode) => {
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
            AppCommand::ClearMonitorBuffer => {
                self.bus.monitor.clear();
            }
            AppCommand::ClearFloppyBuffer => {
                self.bus.floppy.clear_visible_buffer();
            }
            AppCommand::AttachFloppyImage(path) => {
                self.bus.floppy.attach_file(path, self.io_runtime.handle());
            }
            AppCommand::DetachFloppyImage => {
                self.bus.floppy.detach_file();
            }
            AppCommand::SetFloppyDebugBuffer(enabled) => {
                self.bus.floppy.set_debug_buffer(enabled);
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
