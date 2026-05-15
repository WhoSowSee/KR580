use crate::{AppCommand, AppError, AppEvent, AppSnapshot};
use k580_core::{Cpu8080State, PortBus};
use k580_devices::IoBus;
use k580_persistence::{ExportModel, Exporters, Snapshot580Serializer, SubprogramSerializer};

#[derive(Debug, Default)]
pub struct Emulator {
    cpu: Cpu8080State,
    bus: IoBus,
    running: bool,
}

impl Emulator {
    pub fn new(cpu: Cpu8080State, bus: IoBus) -> Self {
        Self {
            cpu,
            bus,
            running: false,
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
            AppCommand::ResetCpu => self.cpu.reset_cpu(),
            AppCommand::ResetRam => self.cpu.reset_ram(),
            AppCommand::ResetRegisters => self.cpu.reset_registers(),
            AppCommand::SetRegister(register, value) => self.cpu.set_register(register, value),
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
                self.running = true;
                self.cpu.run_until_halt(&mut self.bus, 100_000)?;
                self.running = false;
            }
            AppCommand::Stop => {
                self.running = false;
                events.push(AppEvent::Stopped);
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
            AppCommand::ExportDocx(path) => Exporters::write_docx(path, &self.export_model())?,
            AppCommand::Shutdown => events.push(AppEvent::Stopped),
        }
        if self.cpu.halted {
            events.push(AppEvent::HaltStateChanged(true));
        }
        Ok(events)
    }

    fn export_model(&self) -> ExportModel {
        ExportModel::from_cpu(&self.cpu, 0, 256)
    }
}
