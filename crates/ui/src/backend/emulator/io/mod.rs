mod dispatch;
mod jobs;
#[cfg(test)]
mod tests;

use crate::backend::{AppError, AppEvent, CommandResult, RequestId};
use crate::persistence::import::CpuPatch;
use crossbeam_channel::Sender;
use k580_core::Cpu8080State;

use super::Emulator;
use jobs::{IoJob, run};

pub(crate) enum IoUpdate {
    Program(Box<Cpu8080State>),
    Import(CpuPatch<'static>),
    Subprogram { start: u16, values: Vec<u8> },
}

pub(crate) struct IoCompletion {
    pub id: RequestId,
    pub generation: u64,
    pub result: Result<(CommandResult, Option<IoUpdate>), AppError>,
}

fn spawn(id: RequestId, generation: u64, job: IoJob, completion_tx: Sender<IoCompletion>) {
    std::thread::spawn(move || {
        let _ = completion_tx.send(IoCompletion {
            id,
            generation,
            result: run(job),
        });
    });
}

impl Emulator {
    pub(in crate::backend) fn finish_io(&mut self, completion: IoCompletion) -> Vec<AppEvent> {
        if completion.generation != self.document_generation {
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
                }
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
