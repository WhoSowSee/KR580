use iced::Task;

use super::constants::{
    MEMORY_INLINE_INPUT_ID, MEMORY_VALUE_INPUT_ID, REGISTER_INLINE_INPUT_ID,
    REGISTER_NAME_INPUT_ID, REGISTER_VALUE_INPUT_ID,
};
use super::messages::{Message, RegisterInlineTarget};
use super::register_inline::RegisterMove;
use super::state::DesktopApp;

impl DesktopApp {
    /// `direction`: `+1` for ArrowUp, `-1` for ArrowDown.
    pub(crate) fn handle_arrow_key(&mut self, direction: i32) -> Task<Message> {
        if self.opcode_dropdown_address.is_some() {
            self.step_opcode_highlight(if direction > 0 { -1 } else { 1 });
            return Task::none();
        }

        match self.focused_input {
            Some(REGISTER_NAME_INPUT_ID) => {
                if self.register_name_input.is_empty() {
                    self.select_register_target(RegisterInlineTarget::for_register(
                        k580_core::RegisterName::A,
                    ));
                } else {
                    self.step_register(-direction);
                }
                Task::none()
            }
            Some(REGISTER_VALUE_INPUT_ID) => {
                if self.register_name_input.is_empty() {
                    self.select_register_target(RegisterInlineTarget::for_register(
                        k580_core::RegisterName::A,
                    ));
                } else {
                    self.step_register_value_input(direction);
                }
                Task::none()
            }
            Some(REGISTER_INLINE_INPUT_ID) => {
                if self.register_name_input.is_empty() {
                    self.select_register_target(RegisterInlineTarget::for_register(
                        k580_core::RegisterName::A,
                    ));
                    Task::none()
                } else {
                    self.step_register_value_input(direction);
                    iced::widget::operation::focus(REGISTER_INLINE_INPUT_ID)
                }
            }
            Some(MEMORY_VALUE_INPUT_ID) => {
                self.step_memory_value_input(direction);
                Task::none()
            }
            Some(MEMORY_INLINE_INPUT_ID) => {
                // Stepping the address rebuilds the inline input
                // under the new row, so we defer focus via
                // `RefocusInline`. `step_memory_address_browse` skips
                // the `SetPc` round-trip that ate focus.
                let scroll = self.step_memory_address_browse(-direction);
                scroll.chain(Task::done(Message::RefocusInline))
            }
            None if self.active_register_target.is_some() => {
                self.step_register(-direction);
                Task::none()
            }
            _ => {
                if self.register_name_input.is_empty() && self.memory_address_input.is_empty() {
                    self.select_register_target(RegisterInlineTarget::for_register(
                        k580_core::RegisterName::A,
                    ));
                    Task::none()
                } else {
                    self.step_memory_address(-direction)
                }
            }
        }
    }

    pub(crate) fn handle_horizontal_arrow_key(&mut self, direction: i32) -> Task<Message> {
        if self.focused_input.is_none() && self.active_register_target.is_some() {
            let movement = if direction < 0 {
                RegisterMove::Left
            } else {
                RegisterMove::Right
            };
            self.navigate_active_register_target(movement);
        }
        Task::none()
    }
}
