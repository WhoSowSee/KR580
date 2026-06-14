use crate::app::{
    DesktopApp, MEMORY_ADDRESS_INPUT_ID, MEMORY_INLINE_INPUT_ID, MEMORY_VALUE_INPUT_ID,
    REGISTER_INLINE_INPUT_ID, REGISTER_NAME_INPUT_ID, REGISTER_VALUE_INPUT_ID,
};
use iced::advanced::mouse;

impl DesktopApp {
    pub(crate) fn handle_replacement_double_click(&mut self, generation: u64) {
        let click = mouse::Click::new(
            self.latest_cursor_position,
            mouse::Button::Left,
            self.previous_left_click,
        );
        self.previous_left_click = Some(click);

        if click.kind() != mouse::click::Kind::Double || self.running {
            return;
        }

        match self.focused_input {
            Some(MEMORY_INLINE_INPUT_ID) if self.selected_memory_address().is_some() => {
                self.begin_replacement(MEMORY_INLINE_INPUT_ID);
                self.replacement_reconcile_guard = Some((generation, MEMORY_INLINE_INPUT_ID));
            }
            Some(REGISTER_INLINE_INPUT_ID) if self.inline_register_target.is_some() => {
                self.begin_replacement(REGISTER_INLINE_INPUT_ID);
                self.replacement_reconcile_guard = Some((generation, REGISTER_INLINE_INPUT_ID));
            }
            _ => {}
        }
    }

    pub(crate) fn begin_replacement(&mut self, input: &'static str) {
        self.finish_replacement();
        let original = self.input_value(input).to_owned();
        self.replacement_placeholder = if original.is_empty() {
            replacement_fallback(input).to_owned()
        } else {
            original.clone()
        };
        self.replacement_original_value = original;
        self.replacement_input = Some(input);
        self.input_value_mut(input).clear();
    }

    pub(crate) fn continue_replacement(&mut self, input: &'static str) {
        if self.replacement_input == Some(input) {
            self.begin_replacement(input);
        }
    }

    pub(crate) fn finish_replacement(&mut self) {
        let Some(input) = self.replacement_input.take() else {
            return;
        };
        if self.input_value(input).is_empty() {
            let original = std::mem::take(&mut self.replacement_original_value);
            *self.input_value_mut(input) = original;
        }
        self.replacement_placeholder.clear();
        self.replacement_original_value.clear();
    }

    pub(crate) fn commit_replacement(&mut self, input: &'static str) {
        if self.replacement_input == Some(input) && self.input_value(input).is_empty() {
            *self.input_value_mut(input) = self.replacement_placeholder.clone();
        }
    }

    pub(crate) fn materialize_input_fallback(&mut self, input: &'static str) -> bool {
        if !self.input_value(input).is_empty() {
            return false;
        }
        let fallback = replacement_fallback(input);
        if fallback.is_empty() {
            return false;
        }
        *self.input_value_mut(input) = fallback.to_owned();
        true
    }

    pub(crate) fn input_placeholder<'a>(
        &'a self,
        input: &'static str,
        fallback: &'a str,
    ) -> &'a str {
        if self.replacement_input == Some(input) {
            &self.replacement_placeholder
        } else {
            fallback
        }
    }

    fn input_value(&self, input: &'static str) -> &str {
        match input {
            MEMORY_ADDRESS_INPUT_ID => &self.memory_address_input,
            MEMORY_VALUE_INPUT_ID => &self.memory_value_input,
            MEMORY_INLINE_INPUT_ID => &self.memory_inline_value_input,
            REGISTER_NAME_INPUT_ID => &self.register_name_input,
            REGISTER_VALUE_INPUT_ID | REGISTER_INLINE_INPUT_ID => &self.register_value_input,
            _ => "",
        }
    }

    fn input_value_mut(&mut self, input: &'static str) -> &mut String {
        match input {
            MEMORY_ADDRESS_INPUT_ID => &mut self.memory_address_input,
            MEMORY_VALUE_INPUT_ID => &mut self.memory_value_input,
            MEMORY_INLINE_INPUT_ID => &mut self.memory_inline_value_input,
            REGISTER_NAME_INPUT_ID => &mut self.register_name_input,
            REGISTER_VALUE_INPUT_ID | REGISTER_INLINE_INPUT_ID => &mut self.register_value_input,
            _ => unreachable!(),
        }
    }
}

fn replacement_fallback(input: &'static str) -> &'static str {
    match input {
        MEMORY_ADDRESS_INPUT_ID => "0000",
        REGISTER_NAME_INPUT_ID => "A",
        MEMORY_VALUE_INPUT_ID
        | MEMORY_INLINE_INPUT_ID
        | REGISTER_VALUE_INPUT_ID
        | REGISTER_INLINE_INPUT_ID => "00",
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::DesktopApp;
    use crate::app::{
        MEMORY_ADDRESS_INPUT_ID, MEMORY_INLINE_INPUT_ID, MEMORY_VALUE_INPUT_ID, Message,
        REGISTER_INLINE_INPUT_ID, REGISTER_NAME_INPUT_ID, REGISTER_VALUE_INPUT_ID,
        RegisterInlineTarget,
    };
    use iced::Point;
    use k580_core::RegisterName;

    #[test]
    fn second_click_replaces_selected_inline_memory_value() {
        let (mut app, _) = DesktopApp::with_initial_path(None);
        app.select_memory(0x0010);
        app.memory_value_input = "3E".to_owned();
        app.memory_inline_value_input = "3E".to_owned();
        app.focused_input = Some(MEMORY_INLINE_INPUT_ID);
        app.latest_cursor_position = Point::new(10.0, 10.0);

        let _ = app.update(Message::MousePressed);
        let _ = app.update(Message::MousePressed);
        let _ = app.update(Message::FocusReconciled {
            generation: 1,
            hit: Some(iced::widget::Id::new(MEMORY_INLINE_INPUT_ID)),
        });
        let _ = app.update(Message::FocusReconciled {
            generation: 2,
            hit: None,
        });
        let _ = app.update(Message::FocusReconciled {
            generation: 1,
            hit: None,
        });

        assert!(app.memory_inline_value_input.is_empty());
        assert_eq!(app.input_placeholder(MEMORY_INLINE_INPUT_ID, "00"), "3E");
    }

    #[test]
    fn second_click_replaces_schematic_and_mux_register_values() {
        let targets = [
            RegisterInlineTarget::Schematic(RegisterName::A),
            RegisterInlineTarget::Schematic(RegisterName::B),
            RegisterInlineTarget::Schematic(RegisterName::C),
            RegisterInlineTarget::Mux(RegisterName::D),
        ];

        for target in targets {
            let (mut app, _) = DesktopApp::with_initial_path(None);
            app.snapshot.cpu.registers.set(target.register(), 0x41);
            app.enter_inline_register(target);
            app.focused_input = Some(REGISTER_INLINE_INPUT_ID);
            app.latest_cursor_position = Point::new(10.0, 10.0);

            let _ = app.update(Message::MousePressed);
            let _ = app.update(Message::MousePressed);
            let _ = app.update(Message::FocusReconciled {
                generation: 1,
                hit: Some(iced::widget::Id::new(REGISTER_INLINE_INPUT_ID)),
            });
            let _ = app.update(Message::FocusReconciled {
                generation: 2,
                hit: None,
            });

            assert!(app.register_value_input.is_empty(), "target: {target:?}");
            assert_eq!(
                app.input_placeholder(REGISTER_INLINE_INPUT_ID, "00"),
                "41",
                "target: {target:?}"
            );
        }
    }

    #[test]
    fn replacement_of_an_already_empty_field_keeps_its_visible_placeholder() {
        for (input, fallback) in [
            (MEMORY_ADDRESS_INPUT_ID, "0000"),
            (MEMORY_VALUE_INPUT_ID, "00"),
            (MEMORY_INLINE_INPUT_ID, "00"),
            (REGISTER_NAME_INPUT_ID, "A"),
            (REGISTER_VALUE_INPUT_ID, "00"),
            (REGISTER_INLINE_INPUT_ID, "00"),
        ] {
            let (mut app, _) = DesktopApp::with_initial_path(None);
            app.input_value_mut(input).clear();

            app.begin_replacement(input);

            assert!(app.input_value(input).is_empty(), "input: {input}");
            assert_eq!(
                app.input_placeholder(input, fallback),
                fallback,
                "input: {input}"
            );
            app.commit_replacement(input);
            assert_eq!(app.input_value(input), fallback, "input: {input}");
        }
    }

    #[test]
    fn leaving_an_empty_replacement_field_does_not_materialize_its_fallback() {
        for input in [
            MEMORY_ADDRESS_INPUT_ID,
            MEMORY_VALUE_INPUT_ID,
            REGISTER_NAME_INPUT_ID,
            REGISTER_VALUE_INPUT_ID,
        ] {
            let (mut app, _) = DesktopApp::with_initial_path(None);
            app.input_value_mut(input).clear();

            app.begin_replacement(input);
            app.finish_replacement();

            assert!(app.input_value(input).is_empty(), "input: {input}");
        }
    }

    #[test]
    fn memory_focus_cycle_keeps_initially_empty_fields_empty() {
        let (mut app, _) = DesktopApp::with_initial_path(None);
        app.memory_address_input.clear();
        app.memory_value_input.clear();

        let _ = app.cycle_focus(iced::widget::Id::new(MEMORY_ADDRESS_INPUT_ID), false);
        let _ = app.cycle_focus(iced::widget::Id::new(MEMORY_VALUE_INPUT_ID), true);
        let _ = app.cycle_focus(iced::widget::Id::new(MEMORY_ADDRESS_INPUT_ID), false);

        assert!(app.memory_address_input.is_empty());
        assert!(app.memory_value_input.is_empty());
    }

    #[test]
    fn register_focus_cycle_keeps_initially_empty_fields_empty() {
        let (mut app, _) = DesktopApp::with_initial_path(None);
        app.register_name_input.clear();
        app.register_value_input.clear();

        let _ = app.cycle_focus(iced::widget::Id::new(REGISTER_NAME_INPUT_ID), false);
        let _ = app.cycle_focus(iced::widget::Id::new(REGISTER_VALUE_INPUT_ID), true);
        let _ = app.cycle_focus(iced::widget::Id::new(REGISTER_NAME_INPUT_ID), false);

        assert!(app.register_name_input.is_empty());
        assert!(app.register_value_input.is_empty());
    }

    #[test]
    fn focusing_another_field_after_escape_keeps_empty_memory_value_empty() {
        let (mut app, _) = DesktopApp::with_initial_path(None);
        app.memory_address_input.clear();
        app.memory_value_input.clear();
        app.focused_input = Some(MEMORY_ADDRESS_INPUT_ID);

        let _ = app.cycle_focus(iced::widget::Id::new(MEMORY_ADDRESS_INPUT_ID), false);
        let _ = app.handle_esc();

        assert_eq!(app.replacement_input, None);
        assert!(app.memory_value_input.is_empty());

        let _ = app.update(Message::ResolveFocusedTracker(None));
        let _ = app.handle_focus_reconciled(
            app.mouse_press_generation,
            Some(iced::widget::Id::new(MEMORY_ADDRESS_INPUT_ID)),
        );

        assert!(app.memory_value_input.is_empty());
    }
}
