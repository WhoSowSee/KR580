use super::jobs::run;
use super::{IoCompletion, IoGeneration, IoJob};
use crate::backend::{AppError, RequestId};
use crossbeam_channel::{Sender, TrySendError, bounded};

pub(crate) struct IoWorker {
    sender: Sender<Work>,
}

enum Work {
    Request(Box<(RequestId, IoGeneration, IoJob)>),
    Barrier(Sender<()>),
}

impl IoWorker {
    pub(crate) fn new(completion: Sender<IoCompletion>) -> Result<Self, AppError> {
        let (sender, receiver) = bounded(8);
        std::thread::Builder::new()
            .name("kr580-persistence".into())
            .spawn(move || {
                for work in receiver {
                    let (id, generation, job) = match work {
                        Work::Request(request) => *request,
                        Work::Barrier(done) => {
                            let _ = done.send(());
                            continue;
                        }
                    };
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

    pub(crate) fn finish_pending(&self) -> Result<(), AppError> {
        let (done, waiting) = bounded(0);
        self.sender
            .send(Work::Barrier(done))
            .map_err(|_| AppError::WorkerStopped)?;
        waiting.recv().map_err(|_| AppError::WorkerStopped)
    }

    pub(super) fn enqueue(
        &self,
        id: RequestId,
        generation: IoGeneration,
        job: IoJob,
    ) -> Result<(), AppError> {
        self.sender
            .try_send(Work::Request(Box::new((id, generation, job))))
            .map_err(|error| match error {
                TrySendError::Full(_) => crate::devices::DeviceError::Busy.into(),
                TrySendError::Disconnected(_) => AppError::WorkerStopped,
            })
    }
}
