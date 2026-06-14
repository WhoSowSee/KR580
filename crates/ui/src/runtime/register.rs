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
        if let Some(next) = next {
            self.apply_register_with_step_selection(next.register());
            self.enter_inline_register(next);
            if replacing {
                self.begin_replacement(REGISTER_INLINE_INPUT_ID);
            }
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
            Ok(value) => {
                self.undo_stack.break_coalescing();
                let before = self.snapshot.cpu.clone();
                self.dispatch_sync(AppCommand::SetRegister(self.selected_register, value));
                let after = self.snapshot.cpu.clone();
                if before != after {
                    self.recompute_dirty();
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

        self.apply_register_with_step_selection(register_after);
        self.select_register(register_after);
        let target = if stay_on_value {
            REGISTER_VALUE_INPUT_ID
        } else {
            REGISTER_NAME_INPUT_ID
        };
        if replacement == Some(target) {
            self.begin_replacement(target);
        }
        self.focused_input = Some(target);
        operation::focus(target)
    }
}

#[cfg(test)]
mod tests {
    use super::DesktopApp;
    use crate::app::{REGISTER_NAME_INPUT_ID, REGISTER_VALUE_INPUT_ID, RegisterInlineTarget};
    use k580_core::RegisterName;

    #[test]
    fn tab_to_register_value_starts_replacement_and_enter_keeps_it_for_next_register() {
        let (mut app, _) = DesktopApp::with_initial_path(None);
        app.select_register_target(RegisterInlineTarget::Mux(RegisterName::B));
        let focused = iced::widget::Id::new(REGISTER_NAME_INPUT_ID);

        let _ = app.cycle_focus(focused, false);
        assert_eq!(app.focused_input, Some(REGISTER_VALUE_INPUT_ID));
        assert!(app.register_value_input.is_empty());
        assert_eq!(app.input_placeholder(REGISTER_VALUE_INPUT_ID, "00"), "00");

        let _ = app.apply_register_and_step(false);

        assert_eq!(app.selected_register, RegisterName::C);
        assert!(app.register_value_input.is_empty());
        assert_eq!(app.snapshot.cpu.registers.b, 0x00);
    }

    #[test]
    fn double_click_register_edit_keeps_replacement_mode_on_next_register() {
        let (mut app, _) = DesktopApp::with_initial_path(None);
        let target = RegisterInlineTarget::Mux(RegisterName::B);
        app.enter_inline_register_replacing(target);

        assert!(app.register_value_input.is_empty());
        assert_eq!(
            app.input_placeholder(crate::app::REGISTER_INLINE_INPUT_ID, "00"),
            "00"
        );

        let _ = app.apply_inline_register_value(target, false);

        assert_eq!(app.selected_register, RegisterName::C);
        assert!(app.register_value_input.is_empty());
        assert_eq!(app.snapshot.cpu.registers.b, 0x00);
    }

    #[test]
    fn value_input_uses_register_a_when_the_register_field_is_empty() {
        let (mut app, _) = DesktopApp::with_initial_path(None);
        app.register_name_input.clear();
        app.register_value_input.clear();
        app.selected_register = RegisterName::C;

        let _ = app.update(crate::app::Message::RegisterValueChanged("41".to_owned()));

        assert_eq!(app.register_name_input, "A");
        assert_eq!(app.selected_register, RegisterName::A);
        assert_eq!(app.register_value_input, "41");
    }

    #[test]
    fn invalid_value_does_not_fill_an_empty_register_field() {
        let (mut app, _) = DesktopApp::with_initial_path(None);
        app.register_name_input.clear();

        let _ = app.update(crate::app::Message::RegisterValueChanged("GG".to_owned()));

        assert!(app.register_name_input.is_empty());
    }
}
