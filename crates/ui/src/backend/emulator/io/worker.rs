use super::jobs::run;
use super::{IoCompletion, IoGeneration, IoJob};
use crate::backend::{AppError, RequestId};
use crossbeam_channel::{Sender, TrySendError, bounded};

pub(crate) struct IoWorker {
    sender: Sender<(RequestId, IoGeneration, IoJob)>,
}

impl IoWorker {
    pub(crate) fn new(completion: Sender<IoCompletion>) -> Result<Self, AppError> {
        let (sender, receiver) = bounded(8);
        std::thread::Builder::new()
            .name("kr580-persistence".into())
            .spawn(move || {
                for (id, generation, job) in receiver {
                    let result =
                        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(job)))
                            .unwrap_or_else(|_| {
                                Err(AppError::Io("persistence worker panicked".into()))
                            });
                    if completion
                        .send(IoCompletion {
                            id,
                            generation,
                            result,
                        })
                        .is_err()
                    {
                        break;
                    }
                }
            })?;
        Ok(Self { sender })
    }

    pub(super) fn enqueue(
        &self,
        id: RequestId,
        generation: IoGeneration,
        job: IoJob,
    ) -> Result<(), AppError> {
        self.sender
            .try_send((id, generation, job))
            .map_err(|error| match error {
                TrySendError::Full(_) => crate::devices::DeviceError::Busy.into(),
                TrySendError::Disconnected(_) => AppError::WorkerStopped,
            })
    }
}
