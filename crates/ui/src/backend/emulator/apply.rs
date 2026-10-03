use super::{AppCommand, AppError, AppEvent, Emulator, RunMode};
use crate::persistence::{Exporters, Importers, ProgramSerializer, SubprogramSerializer};
use k580_core::PortBus;
use std::time::Duration;

impl Emulator {
    pub(super) fn apply(&mut self, command: AppCommand) -> Result<Vec<AppEvent>, AppError> {
        let mut events = Vec::new();
        match command {
            AppCommand::Request { .. } => {
                return Err(AppError::Io("nested backend request".to_owned().into()));
            }
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
            AppCommand::SetMemoryBlock { start, values } => {
                self.cpu.set_memory_block(start, &values)?;
            }
            AppCommand::ApplyCpuState(state) => {
                self.document_generation = self.document_generation.wrapping_add(1);
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
            AppCommand::ApplyCpuDelta { metadata, memory } => {
                self.document_generation = self.document_generation.wrapping_add(1);
                let was_running = self.running;
                self.cpu.apply_metadata(metadata);
                match memory {
                    crate::backend::MemoryUpdate::Cells(cells) => {
                        for (address, value) in cells {
                            self.cpu.memory.write(address, value);
                        }
                    }
                    crate::backend::MemoryUpdate::Replace(memory) => self.cpu.memory = memory,
                }
                self.running = false;
                self.instructions_since_run = 0;
                if was_running {
                    events.push(AppEvent::Stopped);
                }
                events.push(AppEvent::HaltStateChanged(self.cpu.halted));
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
            AppCommand::SaveProgram(path) => {
                ProgramSerializer::save_file(path, &self.cpu)?;
            }
            AppCommand::LoadProgram(path) => {
                self.cpu = ProgramSerializer::load_file(path)?;
                self.document_generation = self.document_generation.wrapping_add(1);
            }
            AppCommand::LoadSubprogram { path, start } => {
                let was_running = self.running;
                let end = SubprogramSerializer::load_into_state(&path, start, &mut self.cpu)?;
                self.document_generation = self.document_generation.wrapping_add(1);
                self.running = false;
                self.instructions_since_run = 0;
                if was_running {
                    events.push(AppEvent::Stopped);
                }
                events.push(AppEvent::SubprogramLoaded { path, start, end });
            }
            AppCommand::SaveSubprogram { path, start, end } => {
                SubprogramSerializer::save_file(path, &self.cpu, start, end)?;
            }
            AppCommand::ExportTxt(path) => Exporters::write_txt(path, &self.export_model())?,
            AppCommand::ExportXlsx(path) => Exporters::write_xlsx(path, &self.export_model())?,
            AppCommand::ExportTxtWithOptions(path, options) => {
                if options.text_sections.is_empty() {
                    Exporters::write_txt(path, &self.export_model_with_options(&options))?
                } else {
                    Exporters::write_txt_sections(path, &self.export_text_models(&options))?
                }
            }
            AppCommand::ExportXlsxWithOptions(path, options) => {
                if options.xlsx_pages.is_empty() {
                    Exporters::write_xlsx_with_options(
                        path,
                        &self.export_model_with_options(&options),
                        &options,
                    )?
                } else {
                    Exporters::write_xlsx_pages(path, &self.export_xlsx_models(&options))?
                }
            }
            AppCommand::ImportTxt(path) => {
                let model = Importers::read_txt(path)?;
                model.apply_to(&mut self.cpu)?;
            }
            AppCommand::ImportXlsx(path) => {
                let model = Importers::read_xlsx(path)?;
                model.apply_to(&mut self.cpu)?;
            }
            AppCommand::ImportTxtSection(path, section) => {
                let model = Importers::read_txt_section(path, &section)?;
                model.apply_to(&mut self.cpu)?;
            }
            AppCommand::ImportXlsxSheet(path, sheet) => {
                let model = Importers::read_xlsx_sheet(path, &sheet)?;
                model.apply_to(&mut self.cpu)?;
            }
            AppCommand::ClearMonitorBuffer => {
                self.bus.monitor.clear();
            }
            AppCommand::ClearFloppyBuffer => {
                self.bus.floppy.clear_visible_buffer();
            }
            AppCommand::AttachFloppyImage(path) => {
                self.bus
                    .floppy
                    .attach_file(path, self.io_runtime.handle())
                    .map_err(AppError::from)?;
            }
            AppCommand::DetachFloppyImage => {
                self.bus.floppy.detach_file();
            }
            AppCommand::AttachHddFile(path) => {
                self.bus
                    .hdd
                    .attach_file(path, self.io_runtime.handle())
                    .map_err(AppError::from)?;
            }
            AppCommand::SetHddDebugBuffer(enabled) => {
                self.bus.hdd.set_debug_buffer(enabled);
            }
            AppCommand::DetachHddFile => {
                self.bus.hdd.detach_file();
            }
            AppCommand::ClearHddBuffer => {
                self.bus.hdd.clear_visible_buffer();
            }
            AppCommand::SetFloppyDebugBuffer(enabled) => {
                self.bus.floppy.set_debug_buffer(enabled);
            }
            AppCommand::ConfigureNetwork { mode, host, port } => {
                self.bus.network.configure(mode, host, port);
                self.bus.network.start_worker(self.io_runtime.handle());
            }
            AppCommand::ClearNetworkBuffers => {
                self.bus.network.clear_buffers();
            }
            AppCommand::ClearPrinterBuffer => {
                self.bus.printer.clear();
            }
            AppCommand::PrintPrinterNative(settings) => {
                self.bus
                    .printer
                    .print_native(settings, self.io_runtime.handle())
                    .map_err(AppError::from)?;
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
}
