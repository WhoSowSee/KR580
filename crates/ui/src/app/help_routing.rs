use iced::Task;

use super::messages::Message;
use super::state::DesktopApp;

impl DesktopApp {
    pub(crate) fn route_help_dialog_message(&mut self, message: &Message) -> Option<Task<Message>> {
        self.help_dialog.as_ref()?;

        match message {
            Message::Tick
            | Message::CursorMoved(_)
            | Message::ModifiersChanged(_)
            | Message::FocusReconciled { .. }
            | Message::ResolveFocusedTracker(_)
            | Message::MousePressed
            | Message::MousePressedIgnored => None,
            Message::CloseHelp
            | Message::MenuBatch(_)
            | Message::HelpNodeSelected(_)
            | Message::HelpNodeToggled(_)
            | Message::HelpSearchChanged(_)
            | Message::HelpSearchFinished(_)
            | Message::HelpTextAction(_)
            | Message::HelpToggleExpandAll => None,
            Message::EscPressed => Some(Task::done(Message::CloseHelp)),
            _ => Some(Task::none()),
        }
    }
}
