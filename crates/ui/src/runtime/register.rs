//! Register editor helpers attached to `DesktopApp`.

use crate::app::{
    DesktopApp, Message, REGISTER_NAME_INPUT_ID, REGISTER_ORDER, REGISTER_VALUE_INPUT_ID,
    parse_register_name, register_name,
};
use iced::Task;
use iced::widget::operation;
use k580_app::AppCommand;
use k580_core::RegisterName;

use super::parse::{
    bounded_hex_input, bounded_register_input, parse_hex_u8, register_index, saturating_step_u8,
};

impl DesktopApp {
    pub(crate) fn select_register(&mut self, register: RegisterName) {
        self.selected_register = register;
        self.register_name_input = register_name(register).to_owned();
        self.register_value_input = format!("{:02X}", self.snapshot.cpu.registers.get(register));
    }

    pub(crate) fn change_register_name(&mut self, value: String) {
        let Some(value) = bounded_register_input(&value) else {
            return;
        };

        self.register_name_input = value;
        if let Some(register) = parse_register_name(&self.register_name_input) {
            self.selected_register = register;
            self.register_value_input =
                format!("{:02X}", self.snapshot.cpu.registers.get(register));
        }
    }

    pub(crate) fn change_register_value(&mut self, value: String) {
        if let Some(value) = bounded_hex_input(&value, 2) {
            self.register_value_input = value;
        }
    }

    /// Bumps the register value buffered in the editor by `delta`,
    /// saturating at `0x00`/`0xFF`. The byte is *not* written to the CPU
    /// here — applying still requires Enter, the same contract the user
    /// gets when typing the value manually. This way ArrowUp on `00` or
    /// ArrowDown on `FF` simply does nothing instead of silently
    /// committing a wrap-around write.
    pub(crate) fn step_register_value_input(&mut self, delta: i32) {
        let current = parse_hex_u8(&self.register_value_input).unwrap_or(0);
        let next = saturating_step_u8(current, delta);
        self.register_value_input = format!("{next:02X}");
    }

    pub(crate) fn step_register(&mut self, delta: i32) {
        let index = register_index(self.selected_register);
        let len = REGISTER_ORDER.len() as i32;
        let next = (index as i32 + delta).rem_euclid(len) as usize;
        self.select_register(REGISTER_ORDER[next]);
    }

    pub(crate) fn apply_register(&mut self) {
        if let Some(register) = parse_register_name(&self.register_name_input) {
            self.selected_register = register;
        } else {
            self.register_name_input = register_name(self.selected_register).to_owned();
        }

        match parse_hex_u8(&self.register_value_input) {
            Ok(value) => self.dispatch(AppCommand::SetRegister(self.selected_register, value)),
            Err(error) => self.status = error,
        }
    }

    /// Plain Enter handler for the register editor: applies the typed
    /// register value, cycles to the next/previous register, and keeps
    /// focus inside the register editor (whichever of the two fields the
    /// user was in). The memory list and the memory editor are not
    /// touched.
    pub(crate) fn apply_register_and_step(&mut self, backward: bool) -> Task<Message> {
        let stay_on_value = self.focused_input == Some(REGISTER_VALUE_INPUT_ID);
        self.apply_register();
        let delta = if backward { -1 } else { 1 };
        self.step_register(delta);
        let target = if stay_on_value {
            REGISTER_VALUE_INPUT_ID
        } else {
            REGISTER_NAME_INPUT_ID
        };
        self.focused_input = Some(target);
        operation::focus(target)
    }
}
