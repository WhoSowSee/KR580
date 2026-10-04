mod dispatch;
mod jobs;
#[cfg(test)]
mod tests;
mod worker;

use crate::backend::{AppError, AppEvent, CommandResult, RequestId};
use crate::devices::{DeviceError, StorageKind};
use crate::persistence::import::CpuPatch;
use k580_core::Cpu8080State;

use super::Emulator;
use jobs::IoJob;
pub(crate) use worker::IoWorker;

pub(crate) enum IoUpdate {
    Program(Box<Cpu8080State>),
    Import(CpuPatch<'static>),
    Subprogram {
        start: u16,
        values: Vec<u8>,
    },
    Storage {
        kind: StorageKind,
        path: std::path::PathBuf,
        file: Result<std::fs::File, DeviceError>,
    },
}

#[derive(Clone, Copy)]
pub(crate) enum IoGeneration {
    Document(u64),
    Storage(StorageKind, u64),
}

pub(super) fn storage_slot(kind: StorageKind) -> usize {
    match kind {
        StorageKind::Floppy => 0,
        StorageKind::Hdd => 1,
    }
}

pub(crate) struct IoCompletion {
    pub id: RequestId,
    pub generation: IoGeneration,
    pub result: Result<(CommandResult, Option<IoUpdate>), AppError>,
}

impl Emulator {
    pub(in crate::backend) fn finish_io(&mut self, completion: IoCompletion) -> Vec<AppEvent> {
        let current = match completion.generation {
            IoGeneration::Document(generation) => generation == self.document_generation,
            IoGeneration::Storage(kind, generation) => {
                generation == self.storage_generation[storage_slot(kind)]
            }
        };
        if !current {
            return vec![AppEvent::CommandFinished {
                id: completion.id,
                result: Ok(CommandResult::Superseded),
            }];
        }
        let mut events = Vec::new();
        let result = completion.result.and_then(|(result, update)| {
            if let Some(update) = update {
                match update {
                    IoUpdate::Program(state) => self.cpu = *state,
                    IoUpdate::Import(patch) => patch.apply_to(&mut self.cpu),
                    IoUpdate::Subprogram { start, values } => {
                        self.cpu.set_memory_block(start, &values)?;
                    }
                    IoUpdate::Storage { kind, path, file } => {
                        let device = match kind {
                            StorageKind::Floppy => &mut self.bus.floppy,
                            StorageKind::Hdd => &mut self.bus.hdd,
                        };
                        match file {
                            Ok(file) => {
                                device.attach_open_file(path, file, self.io_runtime.handle())?
                            }
                            Err(error) => device.record_attachment_failure(path, error)?,
                        }
                        self.revision = self.revision.wrapping_add(1);
                        return Ok(result);
                    }
                }
                self.revision = self.revision.wrapping_add(1);
                self.running = false;
                self.instructions_since_run = 0;
            }
            Ok(result)
        });
        if let Err(error) = &result {
            events.push(AppEvent::ErrorRaised(error.clone()));
        }
        events.push(AppEvent::StateChanged(Box::new(self.snapshot())));
        events.push(AppEvent::CommandFinished {
            id: completion.id,
            result,
        });
        events
    }
}
