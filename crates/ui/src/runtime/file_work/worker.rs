use super::{FileCompletion, FileRequest, FileResult};
use crate::app::Message;
use crate::backend::AppError;
use crossbeam_channel::{Receiver, Sender, TrySendError, bounded};
use iced::Task;
use tokio::sync::oneshot;

const CAPACITY: usize = 8;
type Work = Box<dyn FnOnce() -> Result<FileResult, AppError> + Send>;

struct Job {
    request: FileRequest,
    work: Work,
    wake: oneshot::Sender<()>,
}

#[cfg(test)]
mod tests;

pub(crate) struct FileWorker {
    sender: Sender<Job>,
    completed: Receiver<FileCompletion>,
    pending: usize,
}

impl FileWorker {
    pub(super) fn new() -> Result<Self, AppError> {
        let (sender, receiver) = bounded::<Job>(CAPACITY);
        let (done, completed) = bounded(CAPACITY);
        std::thread::Builder::new()
            .name("kr580-ui-files".into())
            .spawn(move || {
                for job in receiver {
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(job.work))
                        .unwrap_or_else(|_| Err(AppError::Io("UI file worker panicked".into())));
                    let _ = done.send(FileCompletion {
                        request: job.request,
                        result,
                    });
                    let _ = job.wake.send(());
                }
            })?;
        Ok(Self {
            sender,
            completed,
            pending: 0,
        })
    }

    pub(super) fn enqueue(
        &mut self,
        request: FileRequest,
        work: impl FnOnce() -> Result<FileResult, AppError> + Send + 'static,
    ) -> Result<Task<Message>, Box<FileCompletion>> {
        if self.pending == CAPACITY {
            return Err(Box::new(FileCompletion {
                request,
                result: Err(k580_ui::devices::DeviceError::Busy.into()),
            }));
        }
        let (wake, waiting) = oneshot::channel();
        let job = Job {
            request,
            work: Box::new(work),
            wake,
        };
        if let Err(error) = self.sender.try_send(job) {
            let (job, error) = match error {
                TrySendError::Full(job) => (job, k580_ui::devices::DeviceError::Busy.into()),
                TrySendError::Disconnected(job) => (job, AppError::WorkerStopped),
            };
            return Err(Box::new(FileCompletion {
                request: job.request,
                result: Err(error),
            }));
        }
        self.pending += 1;
        Ok(Task::perform(
            async move {
                let _ = waiting.await;
            },
            |()| Message::FileWorkReady,
        ))
    }

    pub(super) fn take_completion(&mut self) -> Option<FileCompletion> {
        let completion = self.completed.try_recv().ok()?;
        self.pending -= 1;
        Some(completion)
    }

    #[cfg(test)]
    pub(crate) fn pending(&self) -> usize {
        self.pending
    }
}
