use crate::app::{
    DesktopApp, Message, REGISTER_INLINE_INPUT_ID, REGISTER_NAME_INPUT_ID, REGISTER_ORDER,
    REGISTER_VALUE_INPUT_ID, RegisterInlineTarget, RegisterMove, StatusKind, parse_register_name,
    register_name,
};
use crate::backend::AppCommand;
use iced::Task;
use iced::widget::operation;
use k580_core::RegisterName;

use super::parse::{
    bounded_hex_input, bounded_register_input, parse_hex_u8, register_index, saturating_step_u8,
};

impl DesktopApp {
    pub(crate) fn select_register(&mut self, register: RegisterName) {
        self.finish_replacement();
        self.selected_register = register;
        self.register_name_input = register_name(register).to_owned();
        self.register_value_input = format!("{:02X}", self.snapshot.cpu.registers.get(register));
        self.active_register_target = None;
        self.inline_register_target = None;
    }

    pub(crate) fn select_register_target(&mut self, target: RegisterInlineTarget) {
        self.finish_replacement();
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
        self.finish_replacement();
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

    pub(crate) fn enter_inline_register_replacing(&mut self, target: RegisterInlineTarget) {
        self.enter_inline_register(target);
        self.begin_replacement(REGISTER_INLINE_INPUT_ID);
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
            self.active_register_target = Some(RegisterInlineTarget::for_register(register));
        } else {
            self.register_value_input.clear();
        }
    }

    pub(crate) fn change_register_value(&mut self, value: String) {
        if let Some(value) = bounded_hex_input(&value, 2) {
            if !value.is_empty() && self.materialize_input_fallback(REGISTER_NAME_INPUT_ID) {
                self.selected_register = RegisterName::A;
            }
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
            if self.register_value_input.is_empty()
                && (self.replacement_input == Some(REGISTER_VALUE_INPUT_ID)
                    || self.replacement_input == Some(REGISTER_INLINE_INPUT_ID))
            {
                self.replacement_placeholder.clone()
            } else {
                self.register_value_input.clone()
            }
        } else {
            format!("{:02X}", self.snapshot.cpu.registers.get(register))
        }
    }

    pub(crate) fn apply_inline_register_value(
        &mut self,
        target: RegisterInlineTarget,
        backward: bool,
    ) -> Task<Message> {
        let replacing = self.replacement_input == Some(REGISTER_INLINE_INPUT_ID);
        self.selected_register = target.register();
        self.register_name_input = register_name(self.selected_register).to_owned();
        self.inline_register_target = Some(target);
        let next = target.adjacent(backward);
        let selection = next.map(|next| (self.selected_register, next.register()));
        let value = super::parse::parse_hex_u8(&self.register_value_input).unwrap_or(0);
        self.apply_register_inner(
            selection,
            crate::app::BackendAction::Register {
                source: self.selected_register,
                value,
                target: crate::app::RegisterCompletion::Inline {
                    source: target,
                    next,
                },
                replacing,
            },
        );
        Task::none()
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
        iced::advanced::widget::operate(crate::runtime::unfocus_except(
            iced::advanced::widget::Id::new("__nothing__"),
        ))
        .discard()
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

        let replacing = self.replacement_input == Some(REGISTER_INLINE_INPUT_ID);
        self.enter_inline_register(next);
        if replacing {
            self.begin_replacement(REGISTER_INLINE_INPUT_ID);
        }
        self.focused_input = Some(REGISTER_INLINE_INPUT_ID);
        operation::focus(REGISTER_INLINE_INPUT_ID)
    }

    pub(crate) fn step_register_value_input(&mut self, delta: i32) {
        self.commit_replacement(REGISTER_VALUE_INPUT_ID);
        let current = parse_hex_u8(&self.register_value_input).unwrap_or(0);
        let next = saturating_step_u8(current, delta);
        self.register_value_input = format!("{next:02X}");
    }

    pub(crate) fn step_register(&mut self, delta: i32) {
        let index = register_index(self.selected_register);
        let len = REGISTER_ORDER.len() as i32;
        let next = (index as i32 + delta).rem_euclid(len) as usize;
        self.select_register_target(RegisterInlineTarget::for_register(REGISTER_ORDER[next]));
    }

    fn apply_register_inner(
        &mut self,
        register_selection: Option<(RegisterName, RegisterName)>,
        mut action: crate::app::BackendAction,
    ) {
        self.commit_replacement(REGISTER_NAME_INPUT_ID);
        self.commit_replacement(REGISTER_VALUE_INPUT_ID);
        self.commit_replacement(REGISTER_INLINE_INPUT_ID);
        if self.register_name_input.is_empty() {
            return;
        }
        if let Some(register) = parse_register_name(&self.register_name_input) {
            self.selected_register = register;
        } else {
            self.register_name_input = register_name(self.selected_register).to_owned();
        }

        match parse_hex_u8(&self.register_value_input) {
            Some(value) => {
                if let crate::app::BackendAction::Register {
                    value: submitted, ..
                } = &mut action
                {
                    *submitted = value;
                }
                self.undo_stack.break_coalescing();
                self.dispatch_edit(
                    AppCommand::SetRegister(self.selected_register, value),
                    crate::app::UndoPolicy::Record,
                    register_selection,
                    action,
                );
            }
            None => self.set_status(StatusKind::InvalidByteHex),
        }
    }

    pub(crate) fn apply_register_and_step(&mut self, backward: bool) -> Task<Message> {
        let replacement = self.replacement_input;
        self.commit_replacement(REGISTER_NAME_INPUT_ID);
        self.commit_replacement(REGISTER_VALUE_INPUT_ID);
        if self.register_name_input.is_empty() {
            return Task::none();
        }
        let stay_on_value = self.focused_input == Some(REGISTER_VALUE_INPUT_ID);
        let delta = if backward { -1 } else { 1 };
        let register_before = self.selected_register;
        let index = register_index(register_before);
        let len = REGISTER_ORDER.len() as i32;
        let next = (index as i32 + delta).rem_euclid(len) as usize;
        let register_after = REGISTER_ORDER[next];

        let target = if stay_on_value {
            REGISTER_VALUE_INPUT_ID
        } else {
            REGISTER_NAME_INPUT_ID
        };
        let value = parse_hex_u8(&self.register_value_input).unwrap_or(0);
        self.apply_register_inner(
            Some((register_before, register_after)),
            crate::app::BackendAction::Register {
                source: register_before,
                value,
                target: crate::app::RegisterCompletion::Field {
                    next: register_after,
                    input: target,
                    focus: self.focused_input,
                },
                replacing: replacement == Some(target),
            },
        );
        Task::none()
    }

    pub(crate) fn finish_register_completion(
        &mut self,
        source: RegisterName,
        value: u8,
        target: crate::app::RegisterCompletion,
        replacing: bool,
    ) -> Task<Message> {
        if !self.register_completion_current(source, value, &target) {
            return Task::none();
        }
        let input = match target {
            crate::app::RegisterCompletion::Inline {
                next: Some(next), ..
            } => {
                self.enter_inline_register(next);
                REGISTER_INLINE_INPUT_ID
            }
            crate::app::RegisterCompletion::Inline { source, next: None } => {
                self.select_register_target(source);
                self.focused_input = None;
                return Task::none();
            }
            crate::app::RegisterCompletion::Field { next, input, .. } => {
                self.select_register(next);
                input
            }
        };
        if replacing {
            self.begin_replacement(input);
        }
        self.focused_input = Some(input);
        operation::focus(input)
    }

    pub(crate) fn register_completion_current(
        &self,
        source: RegisterName,
        value: u8,
        target: &crate::app::RegisterCompletion,
    ) -> bool {
        if self.selected_register != source
            || parse_hex_u8(&self.register_value_input) != Some(value)
        {
            return false;
        }
        match target {
            crate::app::RegisterCompletion::Inline { source, .. } => {
                self.inline_register_target == Some(*source)
            }
            crate::app::RegisterCompletion::Field { focus, .. } => self.focused_input == *focus,
        }
    }
}

#[cfg(test)]
mod tests;
