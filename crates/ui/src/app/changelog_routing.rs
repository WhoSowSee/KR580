use iced::Task;

use super::{DesktopApp, Message};

impl DesktopApp {
    pub(crate) fn route_changelog_dialog_message(
        &mut self,
        message: &Message,
    ) -> Option<Task<Message>> {
        self.changelog_dialog.as_ref()?;

        match message {
            Message::Tick
            | Message::CursorMoved(_)
            | Message::ModifiersChanged(_)
            | Message::FocusReconciled { .. }
            | Message::ResolveFocusedTracker(_)
            | Message::MousePressed
            | Message::MousePressedIgnored => None,
            Message::CloseChangelog
            | Message::ChangelogReleaseSelected(_)
            | Message::ChangelogTextAction(_) => None,
            Message::EscPressed => Some(Task::done(Message::CloseChangelog)),
            _ => Some(Task::none()),
        }
    }
}
