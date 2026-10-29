use super::{BackendAction, DesktopApp, Message};
use crate::backend::AppCommand;
use iced::{Task, window};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum ShutdownState {
    #[default]
    Idle,
    Draining,
    Pending,
    Ready,
}

impl DesktopApp {
    pub(crate) fn request_shutdown(&mut self) -> Task<Message> {
        match self.shell.shutdown {
            ShutdownState::Ready => self
                .shell
                .main_window_id
                .map_or_else(iced::exit, window::close),
            ShutdownState::Idle => {
                self.shell.shutdown = ShutdownState::Draining;
                self.execution.running = false;
                self.dispatch(AppCommand::Stop);
                Task::none()
            }
            ShutdownState::Draining | ShutdownState::Pending => Task::none(),
        }
    }

    pub(crate) fn advance_shutdown(&mut self) {
        if self.shell.shutdown != ShutdownState::Draining
            || !self.requests.pending_requests.is_empty()
            || self.preferences.file_association_pending
            || self
                .requests
                .file_worker
                .as_ref()
                .is_some_and(|worker| worker.pending() != 0)
        {
            return;
        }
        self.shell.shutdown = ShutdownState::Pending;
        if !self.dispatch_action(AppCommand::Shutdown, BackendAction::Shutdown) {
            self.shell.shutdown = ShutdownState::Idle;
        }
    }

    pub(crate) fn accepts_shutdown_message(&self, message: &Message) -> bool {
        self.shell.shutdown == ShutdownState::Idle
            || matches!(
                message,
                Message::FileWorkReady
                    | Message::WindowClose
                    | Message::WindowClosed(_)
                    | Message::WindowCloseRequested(_)
                    | Message::WindowOpened(_)
                    | Message::WindowResized { .. }
                    | Message::FrameRendered
                    | Message::WindowDragStart
                    | Message::DetachedWindowDragStart(_)
                    | Message::WindowMinimize
                    | Message::WindowToggleMaximize
                    | Message::WindowMaximizedChanged(_)
                    | Message::SettingsFileAssociationFinished(_)
            )
    }
}

#[cfg(test)]
mod tests;
