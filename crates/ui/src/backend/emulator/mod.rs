mod apply;
mod changes;
mod devices;
mod export_model;
pub(crate) mod io;
mod tick;

use crate::backend::{AppCommand, AppError, AppEvent, AppSnapshot, RunMode};
use crate::devices::IoBus;
use k580_core::Cpu8080State;
use std::time::Duration;

pub const DEFAULT_STEP_INTERVAL: Duration = Duration::from_millis(100);

pub(super) const MAX_INSTRUCTIONS_PER_RUN: u64 = 100_000;

#[derive(Debug)]
pub struct Emulator {
    pub(super) cpu: Cpu8080State,
    pub(super) bus: IoBus,
    pub(super) io_runtime: tokio::runtime::Runtime,
    pub(super) running: bool,
    pub(super) instructions_since_run: u64,
    pub(super) step_interval: Duration,
    pub(super) run_mode: RunMode,
    document_generation: u64,
    storage_generation: [u64; 2],
    revision: u64,
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
            document_generation: 0,
            storage_generation: [0; 2],
            revision: 0,
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
            document_generation: 0,
            storage_generation: [0; 2],
            revision: 0,
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
            revision: self.revision,
            cpu: self.cpu.clone(),
            devices: self.bus.snapshot(),
        }
    }

    pub fn handle_command(&mut self, command: AppCommand) -> Vec<AppEvent> {
        match command {
            AppCommand::Request { id, command } => self.handle_command_inner(Some(id), *command),
            command => self.handle_command_inner(None, command),
        }
    }

    fn handle_command_inner(
        &mut self,
        request_id: Option<crate::backend::RequestId>,
        command: AppCommand,
    ) -> Vec<AppEvent> {
        if matches!(command, AppCommand::RequestSnapshot) {
            let mut events = vec![AppEvent::StateChanged(Box::new(self.snapshot()))];
            if let Some(id) = request_id {
                events.push(AppEvent::CommandFinished {
                    id,
                    result: Ok(crate::backend::CommandResult::Completed),
                });
            }
            return events;
        }
        if matches!(&command, AppCommand::ClearNetworkBuffers) {
            let network = self.bus.network.state();
            if network.rx_buffer.is_empty() && network.tx_buffer.is_empty() {
                return request_id
                    .map(|id| {
                        vec![AppEvent::CommandFinished {
                            id,
                            result: Ok(crate::backend::CommandResult::Completed),
                        }]
                    })
                    .unwrap_or_default();
            }
        }
        let (checkpoint, command, allowed) = match command {
            AppCommand::Edit(command) => {
                let checkpoint =
                    request_id.and_then(|_| changes::CpuCheckpoint::capture(&self.cpu, &command));
                let allowed = checkpoint.is_some();
                (checkpoint, *command, allowed)
            }
            command => (None, command, true),
        };
        let result = if allowed {
            self.apply(command)
        } else {
            Err(AppError::Io(
                "tracked edit requires a CPU mutation request".into(),
            ))
        };
        self.revision = self.revision.wrapping_add(1);
        let mut events = match result {
            Ok(events) => events,
            Err(error) => vec![AppEvent::ErrorRaised(error)],
        };
        let completion = if matches!(events.last(), Some(AppEvent::SubprogramLoaded { .. })) {
            events.pop()
        } else {
            None
        };
        events.push(AppEvent::StateChanged(Box::new(self.snapshot())));
        events.extend(completion);
        if let Some(id) = request_id {
            let result = if events
                .iter()
                .any(|event| matches!(event, AppEvent::ErrorRaised(_)))
            {
                events
                    .iter()
                    .find_map(|event| match event {
                        AppEvent::ErrorRaised(error) => Some(Err(error.clone())),
                        _ => None,
                    })
                    .unwrap_or(Ok(crate::backend::CommandResult::Completed))
            } else {
                match checkpoint {
                    Some(checkpoint) => Ok(crate::backend::CommandResult::CpuChanged {
                        revision: self.revision,
                        change: Box::new(checkpoint.finish(&self.cpu)),
                    }),
                    None => Ok(crate::backend::CommandResult::Completed),
                }
            };
            events.push(AppEvent::CommandFinished { id, result });
        }
        events
    }
}
