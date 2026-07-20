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
                let movement = if direction > 0 {
                    RegisterMove::Up
                } else {
                    RegisterMove::Down
                };
                self.navigate_inline_register_target(movement)
            }
            Some(MEMORY_VALUE_INPUT_ID) => {
                self.step_memory_value_input(direction);
                Task::none()
            }
            Some(MEMORY_INLINE_INPUT_ID) => {
                let replacing = self.replacement_input == Some(MEMORY_INLINE_INPUT_ID);
                if replacing {
                    self.finish_replacement();
                }
                let scroll = self.step_memory_address_browse(-direction);
                if replacing {
                    self.begin_replacement(MEMORY_INLINE_INPUT_ID);
                }
                scroll.chain(Task::done(Message::RefocusInline))
            }
            None if self.active_register_target.is_some() => {
                let movement = if direction > 0 {
                    RegisterMove::Up
                } else {
                    RegisterMove::Down
                };
                self.navigate_active_register_target(movement);
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
        let movement = if direction < 0 {
            RegisterMove::Left
        } else {
            RegisterMove::Right
        };
        if self.focused_input == Some(REGISTER_INLINE_INPUT_ID) {
            return self.navigate_inline_register_target(movement);
        }
        if self.focused_input.is_none() && self.active_register_target.is_some() {
            self.navigate_active_register_target(movement);
        }
        Task::none()
    }
}

#[cfg(test)]
mod tests {
    use super::DesktopApp;
    use crate::app::{MEMORY_INLINE_INPUT_ID, REGISTER_INLINE_INPUT_ID, RegisterInlineTarget};
    use k580_core::RegisterName;

    #[test]
    fn memory_arrow_keeps_replacement_mode_on_the_next_cell() {
        let (mut app, _) = DesktopApp::with_initial_path(None);
        app.snapshot.cpu.pc = 0x0010;
        app.snapshot.cpu.memory.write(0x0010, 0x3E);
        app.snapshot.cpu.memory.write(0x0011, 0x41);
        app.enter_inline_memory_replacing(0x0010);
        app.focused_input = Some(MEMORY_INLINE_INPUT_ID);

        let _ = app.handle_arrow_key(-1);

        assert_eq!(app.selected_memory_address(), Some(0x0011));
        assert!(app.memory_inline_value_input.is_empty());
        assert_eq!(app.input_placeholder(MEMORY_INLINE_INPUT_ID, "00"), "41");
    }

    #[test]
    fn mux_vertical_arrow_keeps_replacement_mode_on_the_next_register() {
        let (mut app, _) = DesktopApp::with_initial_path(None);
        app.snapshot.cpu.registers.set(RegisterName::B, 0x12);
        app.snapshot.cpu.registers.set(RegisterName::D, 0x34);
        app.enter_inline_register_replacing(RegisterInlineTarget::Mux(RegisterName::B));
        app.focused_input = Some(REGISTER_INLINE_INPUT_ID);

        let _ = app.handle_arrow_key(-1);

        assert_eq!(
            app.inline_register_target,
            Some(RegisterInlineTarget::Mux(RegisterName::D))
        );
        assert!(app.register_value_input.is_empty());
        assert_eq!(app.input_placeholder(REGISTER_INLINE_INPUT_ID, "00"), "34");
    }

    #[test]
    fn schematic_horizontal_arrow_keeps_replacement_mode_on_the_next_register() {
        let (mut app, _) = DesktopApp::with_initial_path(None);
        app.snapshot.cpu.registers.set(RegisterName::A, 0x12);
        app.snapshot.cpu.registers.set(RegisterName::B, 0x34);
        app.enter_inline_register_replacing(RegisterInlineTarget::Schematic(RegisterName::A));
        app.focused_input = Some(REGISTER_INLINE_INPUT_ID);

        let _ = app.handle_horizontal_arrow_key(1);

        assert_eq!(
            app.inline_register_target,
            Some(RegisterInlineTarget::Schematic(RegisterName::B))
        );
        assert!(app.register_value_input.is_empty());
        assert_eq!(app.input_placeholder(REGISTER_INLINE_INPUT_ID, "00"), "34");
    }

    #[test]
    fn schematic_vertical_arrows_do_not_leave_the_register_strip() {
        let (mut app, _) = DesktopApp::with_initial_path(None);

        for register in [RegisterName::A, RegisterName::B, RegisterName::C] {
            let target = RegisterInlineTarget::Schematic(register);
            app.select_register_target(target);
            app.focused_input = None;

            let _ = app.handle_arrow_key(1);
            assert_eq!(app.active_register_target, Some(target));

            let _ = app.handle_arrow_key(-1);
            assert_eq!(app.active_register_target, Some(target));
        }
    }
}
