use crate::app::{
    DesktopApp, Message, REGISTER_INLINE_INPUT_ID, REGISTER_NAME_INPUT_ID, REGISTER_ORDER,
    REGISTER_VALUE_INPUT_ID, RegisterInlineTarget, RegisterMove, parse_register_name,
    register_name,
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
        self.active_register_target = None;
        self.inline_register_target = None;
    }

    pub(crate) fn select_register_target(&mut self, target: RegisterInlineTarget) {
        self.selected_register = target.register();
        self.register_name_input = register_name(self.selected_register).to_owned();
        self.register_value_input = format!(
            "{:02X}",
            self.snapshot.cpu.registers.get(self.selected_register)
        );
        self.active_register_target = Some(target);
        self.inline_register_target = None;
        self.focused_input = None;
    }

    pub(crate) fn enter_inline_register(&mut self, target: RegisterInlineTarget) {
        self.selected_register = target.register();
        self.register_name_input = register_name(self.selected_register).to_owned();
        self.register_value_input = format!(
            "{:02X}",
            self.snapshot.cpu.registers.get(self.selected_register)
        );
        self.active_register_target = Some(target);
        self.inline_register_target = Some(target);
        self.inline_register_just_entered = true;
    }

    pub(crate) fn change_register_name(&mut self, value: String) {
        let Some(value) = bounded_register_input(&value) else {
            return;
        };

        let before = self.register_name_input.clone();
        self.register_name_input = value;
        self.undo_stack.push_text(
            REGISTER_NAME_INPUT_ID,
            before,
            self.register_name_input.clone(),
        );
        if let Some(register) = parse_register_name(&self.register_name_input) {
            self.selected_register = register;
            self.register_value_input =
                format!("{:02X}", self.snapshot.cpu.registers.get(register));
        }
    }

    pub(crate) fn change_register_value(&mut self, value: String) {
        if let Some(value) = bounded_hex_input(&value, 2) {
            let before = self.register_value_input.clone();
            self.register_value_input = value;
            self.undo_stack.push_text(
                REGISTER_VALUE_INPUT_ID,
                before,
                self.register_value_input.clone(),
            );
        }
    }

    pub(crate) fn change_inline_register_value(
        &mut self,
        target: RegisterInlineTarget,
        value: String,
    ) {
        if self.inline_register_target != Some(target) {
            self.enter_inline_register(target);
        }
        self.change_register_value(value);
    }

    pub(crate) fn display_register_value(&self, register: RegisterName) -> String {
        if parse_register_name(&self.register_name_input) == Some(register) {
            self.register_value_input.clone()
        } else {
            format!("{:02X}", self.snapshot.cpu.registers.get(register))
        }
    }

    pub(crate) fn apply_inline_register_value(
        &mut self,
        target: RegisterInlineTarget,
        backward: bool,
    ) -> Task<Message> {
        self.selected_register = target.register();
        self.register_name_input = register_name(self.selected_register).to_owned();
        self.inline_register_target = Some(target);
        let next = target.adjacent(backward);
        if let Some(next) = next {
            self.apply_register_with_step_selection(next.register());
            self.enter_inline_register(next);
            self.focused_input = Some(REGISTER_INLINE_INPUT_ID);
            operation::focus(REGISTER_INLINE_INPUT_ID)
        } else {
            self.apply_register_inner(None);
            self.select_register_target(target);
            self.focused_input = None;
            Task::none()
        }
    }

    pub(crate) fn cancel_inline_register_edit(&mut self) -> Task<Message> {
        if let Some(target) = self.inline_register_target {
            let register = target.register();
            self.selected_register = register;
            self.register_name_input = register_name(register).to_owned();
            self.register_value_input =
                format!("{:02X}", self.snapshot.cpu.registers.get(register));
            self.active_register_target = Some(target);
        }
        self.inline_register_target = None;
        self.focused_input = None;
        Task::none()
    }

    pub(crate) fn navigate_active_register_target(&mut self, direction: RegisterMove) {
        let Some(target) = self.active_register_target else {
            return;
        };
        if let Some(next) = target.navigate(direction) {
            self.select_register_target(next);
        }
    }

    pub(crate) fn navigate_inline_register_target(
        &mut self,
        direction: RegisterMove,
    ) -> Task<Message> {
        if self.focused_input != Some(REGISTER_INLINE_INPUT_ID) {
            return Task::none();
        }

        let Some(target) = self.inline_register_target else {
            return Task::none();
        };

        let Some(next) = target.navigate(direction) else {
            return operation::focus(REGISTER_INLINE_INPUT_ID);
        };

        self.enter_inline_register(next);
        self.focused_input = Some(REGISTER_INLINE_INPUT_ID);
        operation::focus(REGISTER_INLINE_INPUT_ID)
    }

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

    /// Tags the undo entry with `(before, after)` so Ctrl+Z restores
    /// the editor to whichever register the user was *editing*, not
    /// whichever register the follow-on step walked to.
    fn apply_register_with_step_selection(&mut self, register_after: RegisterName) {
        let register_before = self.selected_register;
        self.apply_register_inner(Some((register_before, register_after)));
    }

    /// Inlines the dispatch instead of going through `dispatch_with_undo`
    /// so the undo entry can carry the optional register selection.
    fn apply_register_inner(&mut self, register_selection: Option<(RegisterName, RegisterName)>) {
        if let Some(register) = parse_register_name(&self.register_name_input) {
            self.selected_register = register;
        } else {
            self.register_name_input = register_name(self.selected_register).to_owned();
        }

        match parse_hex_u8(&self.register_value_input) {
            Ok(value) => {
                self.undo_stack.break_coalescing();
                let before = self.snapshot.cpu.clone();
                self.dispatch_sync(AppCommand::SetRegister(self.selected_register, value));
                let after = self.snapshot.cpu.clone();
                if before != after {
                    self.dirty = true;
                }
                match register_selection {
                    Some(selection) => self.undo_stack.push_cpu_with_register_selection(
                        before,
                        after,
                        Some(selection),
                    ),
                    None => self.undo_stack.push_cpu(before, after),
                }
            }
            Err(error) => self.set_status_custom(error),
        }
    }

    pub(crate) fn apply_register_and_step(&mut self, backward: bool) -> Task<Message> {
        let stay_on_value = self.focused_input == Some(REGISTER_VALUE_INPUT_ID);
        let delta = if backward { -1 } else { 1 };
        let register_before = self.selected_register;
        let index = register_index(register_before);
        let len = REGISTER_ORDER.len() as i32;
        let next = (index as i32 + delta).rem_euclid(len) as usize;
        let register_after = REGISTER_ORDER[next];

        self.apply_register_with_step_selection(register_after);
        self.select_register(register_after);
        let target = if stay_on_value {
            REGISTER_VALUE_INPUT_ID
        } else {
            REGISTER_NAME_INPUT_ID
        };
        self.focused_input = Some(target);
        operation::focus(target)
    }
}
