mod dispatch;
mod execution;

use super::{DesktopApp, Message};
use iced::Task;

impl DesktopApp {
    pub(crate) fn update(&mut self, message: Message) -> Task<Message> {
        if matches!(message, Message::Tick | Message::FileWorkReady) {
            self.pull_file_completions();
        }
        if matches!(message, Message::Tick) {
            self.pull_events();
        }
        self.shell.updates.tick(std::time::Instant::now());
        let task = if let Some(task) = self.update_release_message(&message) {
            task
        } else if matches!(message, Message::FileWorkReady) {
            Task::none()
        } else {
            self.update_inner(message)
        };
        Task::batch(std::iter::once(task).chain(std::mem::take(&mut self.requests.tasks)))
    }
}
