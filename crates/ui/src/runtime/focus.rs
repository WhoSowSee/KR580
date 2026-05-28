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
            return operation::focus(MEMORY_VALUE_INPUT_ID);
        }
        if focused == iced::widget::Id::new(MEMORY_VALUE_INPUT_ID) {
            return operation::focus(MEMORY_ADDRESS_INPUT_ID);
        }
        if focused == iced::widget::Id::new(REGISTER_NAME_INPUT_ID) {
            return operation::focus(REGISTER_VALUE_INPUT_ID);
        }
        if focused == iced::widget::Id::new(REGISTER_VALUE_INPUT_ID) {
            return operation::focus(REGISTER_NAME_INPUT_ID);
        }
        if focused == iced::widget::Id::new(REGISTER_INLINE_INPUT_ID) {
            return operation::focus(REGISTER_VALUE_INPUT_ID);
        }
        if focused == iced::widget::Id::new(MEMORY_INLINE_INPUT_ID) {
            // Tab on the inline editor steps the selection; the editor is
            // re-rendered against the new row under the same id.
            let step = if backward { -1 } else { 1 };
            let scroll_task = self.step_memory_address(step);
            return scroll_task.chain(operation::focus(MEMORY_INLINE_INPUT_ID));
        }
        Task::none()
    }
}
