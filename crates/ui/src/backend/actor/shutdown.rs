use super::{AppEvent, Emulator};
use crate::backend::AppError;
use crate::backend::emulator::io::{IoCompletion, IoWorker};
use crossbeam_channel::Receiver;

pub(super) fn drain_pending(
    emulator: &mut Emulator,
    worker: &IoWorker,
    completed: &Receiver<IoCompletion>,
) -> Result<Vec<AppEvent>, AppError> {
    let mut events = Vec::new();
    worker.finish_pending()?;
    for completion in completed.try_iter() {
        events.extend(emulator.finish_io(completion));
    }
    Ok(events)
}

#[cfg(test)]
mod tests;
