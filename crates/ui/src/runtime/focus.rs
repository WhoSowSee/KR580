use crate::app::{
    DesktopApp, MEMORY_ADDRESS_INPUT_ID, MEMORY_INLINE_INPUT_ID, MEMORY_VALUE_INPUT_ID, Message,
    REGISTER_INLINE_INPUT_ID, REGISTER_NAME_INPUT_ID, REGISTER_VALUE_INPUT_ID,
};
use iced::Task;
use iced::widget::operation;

impl DesktopApp {
    pub(crate) fn cycle_focus(
        &mut self,
        focused: iced::widget::Id,
        backward: bool,
    ) -> Task<Message> {
        if focused == iced::widget::Id::new(MEMORY_ADDRESS_INPUT_ID) {
            self.begin_replacement(MEMORY_VALUE_INPUT_ID);
            self.focused_input = Some(MEMORY_VALUE_INPUT_ID);
            return operation::focus(MEMORY_VALUE_INPUT_ID);
        }
        if focused == iced::widget::Id::new(MEMORY_VALUE_INPUT_ID) {
            self.begin_replacement(MEMORY_ADDRESS_INPUT_ID);
            self.focused_input = Some(MEMORY_ADDRESS_INPUT_ID);
            return operation::focus(MEMORY_ADDRESS_INPUT_ID);
        }
        if focused == iced::widget::Id::new(REGISTER_NAME_INPUT_ID) {
            self.begin_replacement(REGISTER_VALUE_INPUT_ID);
            self.focused_input = Some(REGISTER_VALUE_INPUT_ID);
            return operation::focus(REGISTER_VALUE_INPUT_ID);
        }
        if focused == iced::widget::Id::new(REGISTER_VALUE_INPUT_ID) {
            self.begin_replacement(REGISTER_NAME_INPUT_ID);
            self.focused_input = Some(REGISTER_NAME_INPUT_ID);
            return operation::focus(REGISTER_NAME_INPUT_ID);
        }
        if focused == iced::widget::Id::new(REGISTER_INLINE_INPUT_ID) {
            return self.cycle_register_target_focus(backward);
        }
        if focused == iced::widget::Id::new(MEMORY_INLINE_INPUT_ID) {
            self.finish_replacement();
            let step = if backward { -1 } else { 1 };
            let scroll_task = self.step_memory_address(step);
            self.begin_replacement(MEMORY_INLINE_INPUT_ID);
            self.focused_input = Some(MEMORY_INLINE_INPUT_ID);
            return scroll_task.chain(operation::focus(MEMORY_INLINE_INPUT_ID));
        }
        Task::none()
    }

    pub(crate) fn cycle_register_target_focus(&mut self, backward: bool) -> Task<Message> {
        let editing = self.focused_input == Some(REGISTER_INLINE_INPUT_ID);
        let target = if editing {
            self.inline_register_target
        } else {
            self.active_register_target
        };
        let Some(target) = target else {
            return Task::none();
        };
        let next = target.tab_adjacent(backward);
        if editing {
            let replacing = self.replacement_input == Some(REGISTER_INLINE_INPUT_ID);
            self.enter_inline_register(next);
            if replacing {
                self.begin_replacement(REGISTER_INLINE_INPUT_ID);
            }
            self.focused_input = Some(REGISTER_INLINE_INPUT_ID);
            operation::focus(REGISTER_INLINE_INPUT_ID)
        } else {
            self.select_register_target(next);
            Task::none()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::app::{DesktopApp, Message, RegisterInlineTarget};
    use k580_core::RegisterName;

    #[test]
    fn tab_cycles_selected_registers_across_schematic_and_mux() {
        let (mut app, _) = DesktopApp::with_initial_path(None);
        use RegisterInlineTarget::{Mux, Schematic};

        for (current, backward, expected) in [
            (Schematic(RegisterName::C), false, Mux(RegisterName::B)),
            (Mux(RegisterName::L), false, Schematic(RegisterName::A)),
            (Schematic(RegisterName::A), true, Mux(RegisterName::L)),
            (Mux(RegisterName::B), true, Schematic(RegisterName::C)),
        ] {
            app.select_register_target(current);
            let _ = app.update(Message::FocusCycle { backward });
            assert_eq!(app.active_register_target, Some(expected));
        }
    }
}
