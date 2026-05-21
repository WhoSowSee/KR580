//! Tab / Shift+Tab focus cycling between the editor inputs.

use crate::app::{
    DesktopApp, MEMORY_ADDRESS_INPUT_ID, MEMORY_INLINE_INPUT_ID, MEMORY_VALUE_INPUT_ID, Message,
    REGISTER_NAME_INPUT_ID, REGISTER_VALUE_INPUT_ID,
};
use iced::Task;
use iced::widget::operation;

impl DesktopApp {
    /// Resolves Tab/Shift+Tab inside one of the panels. Each focus group is a
    /// closed cycle so the user cannot accidentally tab from the memory
    /// editor into the register editor or into the inline list.
    pub(crate) fn cycle_focus(
        &mut self,
        focused: iced::widget::Id,
        backward: bool,
    ) -> Task<Message> {
        // Two-element rings: the panels each have exactly two inputs, so Tab
        // and Shift+Tab both simply swap the pair. That matches the user's
        // expectation of "go to the other field" without surprises.
        if focused == iced::widget::Id::new(MEMORY_ADDRESS_INPUT_ID) {
            self.focused_input = Some(MEMORY_VALUE_INPUT_ID);
            return operation::focus(MEMORY_VALUE_INPUT_ID);
        }
        if focused == iced::widget::Id::new(MEMORY_VALUE_INPUT_ID) {
            self.focused_input = Some(MEMORY_ADDRESS_INPUT_ID);
            return operation::focus(MEMORY_ADDRESS_INPUT_ID);
        }
        if focused == iced::widget::Id::new(REGISTER_NAME_INPUT_ID) {
            self.focused_input = Some(REGISTER_VALUE_INPUT_ID);
            return operation::focus(REGISTER_VALUE_INPUT_ID);
        }
        if focused == iced::widget::Id::new(REGISTER_VALUE_INPUT_ID) {
            self.focused_input = Some(REGISTER_NAME_INPUT_ID);
            return operation::focus(REGISTER_NAME_INPUT_ID);
        }
        if focused == iced::widget::Id::new(MEMORY_INLINE_INPUT_ID) {
            // The inline editor lives on whatever address is currently
            // selected. Tab moves the selection to the next/previous
            // address; the same id is then rendered for the new row, so
            // refocusing it keeps the user typing without grabbing the
            // mouse. Reuse `step_memory_address` to keep highlight, scroll,
            // and search-pattern bookkeeping consistent with arrow keys.
            self.focused_input = Some(MEMORY_INLINE_INPUT_ID);
            let step = if backward { -1 } else { 1 };
            let scroll_task = self.step_memory_address(step);
            return scroll_task.chain(operation::focus(MEMORY_INLINE_INPUT_ID));
        }
        Task::none()
    }
}
