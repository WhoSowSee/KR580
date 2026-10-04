mod completion;
mod worker;

pub(crate) use completion::{FileCompletion, FileRequest, FileResult, SettingsAction};
pub(crate) use worker::FileWorker;

use crate::app::DesktopApp;
use crate::backend::AppError;

impl DesktopApp {
    pub(crate) fn queue_file_work(
        &mut self,
        request: FileRequest,
        work: impl FnOnce() -> Result<FileResult, AppError> + Send + 'static,
    ) {
        if self.requests.file_worker.is_none() {
            match FileWorker::new() {
                Ok(worker) => self.requests.file_worker = Some(worker),
                Err(error) => {
                    self.apply_file_completion(FileCompletion {
                        request,
                        result: Err(error),
                    });
                    return;
                }
            }
        }
        match self
            .requests
            .file_worker
            .as_mut()
            .unwrap()
            .enqueue(request, work)
        {
            Ok(task) => self.requests.tasks.push(task),
            Err(completion) => self.apply_file_completion(*completion),
        }
    }

    pub(crate) fn pull_file_completions(&mut self) {
        loop {
            let completion = self
                .requests
                .file_worker
                .as_mut()
                .and_then(FileWorker::take_completion);
            let Some(completion) = completion else { break };
            self.apply_file_completion(completion);
        }
    }
}
