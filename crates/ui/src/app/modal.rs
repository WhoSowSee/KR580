use super::{DesktopApp, Message, PendingAction};
use iced::Task;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DiscardModalButton {
    Cancel,
    Confirm,
}

impl DiscardModalButton {
    fn next(self) -> Self {
        match self {
            Self::Cancel => Self::Confirm,
            Self::Confirm => Self::Cancel,
        }
    }

    fn previous(self) -> Self {
        match self {
            Self::Cancel => Self::Confirm,
            Self::Confirm => Self::Cancel,
        }
    }
}

impl DesktopApp {
    pub(crate) fn open_discard_modal(&mut self, action: PendingAction) {
        self.pending_action = Some(action);
        self.discard_modal_focus = DiscardModalButton::Cancel;
        self.open_menu = None;
        self.hide_opcode_dropdown();
    }

    pub(crate) fn route_discard_modal_message(
        &mut self,
        message: &Message,
    ) -> Option<Task<Message>> {
        self.pending_action.as_ref()?;

        match message {
            Message::Tick
            | Message::CursorMoved(_)
            | Message::ModifiersChanged(_)
            | Message::WindowOpened(_)
            | Message::WindowResized(_)
            | Message::FrameRendered
            | Message::WindowMaximizedChanged(_)
            | Message::CloseAbout
            | Message::OpenUrl(_) => None,
            Message::ConfirmDiscard => Some(self.confirm_discard()),
            Message::CancelDiscard | Message::EscPressed => {
                self.cancel_discard();
                Some(Task::none())
            }
            Message::FocusCycle { backward } => {
                self.cycle_discard_modal_focus(*backward);
                Some(Task::none())
            }
            Message::EnterPressed => Some(self.submit_discard_modal_focus()),
            _ => Some(Task::none()),
        }
    }

    pub(crate) fn cycle_discard_modal_focus(&mut self, backward: bool) {
        self.discard_modal_focus = if backward {
            self.discard_modal_focus.previous()
        } else {
            self.discard_modal_focus.next()
        };
    }

    pub(crate) fn submit_discard_modal_focus(&mut self) -> Task<Message> {
        match self.discard_modal_focus {
            DiscardModalButton::Cancel => {
                self.cancel_discard();
                Task::none()
            }
            DiscardModalButton::Confirm => self.confirm_discard(),
        }
    }

    pub(crate) fn confirm_discard(&mut self) -> Task<Message> {
        let Some(action) = self.pending_action.take() else {
            return Task::none();
        };
        self.discard_modal_focus = DiscardModalButton::Cancel;
        self.dirty = false;
        match action {
            PendingAction::OpenSnapshot => Task::done(Message::OpenSnapshot),
            PendingAction::NewFile => Task::done(Message::NewFile),
            PendingAction::Import => Task::done(Message::Import),
            PendingAction::CloseWindow => Task::done(Message::WindowClose),
        }
    }

    pub(crate) fn cancel_discard(&mut self) {
        self.pending_action = None;
        self.discard_modal_focus = DiscardModalButton::Cancel;
    }

    pub(crate) fn close_titlebar_popup_before_drag(&mut self) -> bool {
        if self.opcode_dropdown_address.is_some() {
            self.hide_opcode_dropdown();
            return true;
        }
        if self.open_menu.is_some() {
            self.open_menu = None;
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::super::{DesktopApp, MenuId, Message, PendingAction};
    use super::DiscardModalButton;

    #[test]
    fn discard_modal_blocks_memory_navigation_keys() {
        let (mut app, _task) = DesktopApp::with_initial_path(None);
        app.memory_address_input = "000A".to_owned();
        app.open_discard_modal(PendingAction::OpenSnapshot);

        let _task = app.update(Message::ArrowKey(1));

        assert_eq!(app.memory_address_input, "000A");
        assert!(matches!(
            app.pending_action,
            Some(PendingAction::OpenSnapshot)
        ));
    }

    #[test]
    fn discard_modal_blocks_run_shortcut_messages() {
        let (mut app, _task) = DesktopApp::with_initial_path(None);
        app.snapshot.cpu.memory.write(0, 0x13);
        app.open_discard_modal(PendingAction::OpenSnapshot);

        let _task = app.update(Message::ToggleRun);

        assert!(!app.running);
        assert!(matches!(
            app.pending_action,
            Some(PendingAction::OpenSnapshot)
        ));
    }

    #[test]
    fn enter_chooses_cancel_by_default_in_discard_modal() {
        let (mut app, _task) = DesktopApp::with_initial_path(None);
        app.dirty = true;
        app.open_discard_modal(PendingAction::OpenSnapshot);

        let _task = app.update(Message::EnterPressed);

        assert!(app.pending_action.is_none());
        assert!(app.dirty);
    }

    #[test]
    fn tab_cycles_discard_modal_buttons_in_a_ring() {
        let (mut app, _task) = DesktopApp::with_initial_path(None);
        app.open_discard_modal(PendingAction::OpenSnapshot);

        let _task = app.update(Message::FocusCycle { backward: false });
        assert_eq!(app.discard_modal_focus, DiscardModalButton::Confirm);

        let _task = app.update(Message::FocusCycle { backward: false });
        assert_eq!(app.discard_modal_focus, DiscardModalButton::Cancel);

        let _task = app.update(Message::FocusCycle { backward: true });
        assert_eq!(app.discard_modal_focus, DiscardModalButton::Confirm);
    }

    #[test]
    fn enter_confirms_when_confirm_button_is_focused() {
        let (mut app, _task) = DesktopApp::with_initial_path(None);
        app.dirty = true;
        app.open_discard_modal(PendingAction::NewFile);

        let _task = app.update(Message::FocusCycle { backward: false });
        let _task = app.update(Message::EnterPressed);

        assert!(app.pending_action.is_none());
        assert!(!app.dirty);
    }

    #[test]
    fn esc_closes_open_top_menu() {
        let (mut app, _task) = DesktopApp::with_initial_path(None);
        app.open_menu = Some(MenuId::File);

        let _task = app.update(Message::EscPressed);

        assert_eq!(app.open_menu, None);
    }

    #[test]
    fn titlebar_empty_press_closes_opcode_dropdown_before_dragging() {
        let (mut app, _task) = DesktopApp::with_initial_path(None);
        app.opcode_dropdown_address = Some(0x0010);
        app.opcode_search_input = "mov".to_owned();

        let _task = app.update(Message::WindowDragStart);

        assert_eq!(app.opcode_dropdown_address, None);
        assert!(app.opcode_search_input.is_empty());
    }

    #[test]
    fn titlebar_empty_press_closes_top_menu_before_dragging() {
        let (mut app, _task) = DesktopApp::with_initial_path(None);
        app.open_menu = Some(MenuId::Mp);

        let _task = app.update(Message::WindowDragStart);

        assert_eq!(app.open_menu, None);
    }
}
