use super::DesktopApp;
use crate::app::test_support::settle_backend;
use crate::app::{Message, REGISTER_NAME_INPUT_ID, REGISTER_VALUE_INPUT_ID, RegisterInlineTarget};
use k580_core::RegisterName;

#[test]
fn tab_to_register_value_starts_replacement_and_enter_keeps_it_for_next_register() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    app.select_register_target(RegisterInlineTarget::Mux(RegisterName::B));
    let focused = iced::widget::Id::new(REGISTER_NAME_INPUT_ID);

    let _ = app.cycle_focus(focused, false);
    assert_eq!(app.interaction.focused_input, Some(REGISTER_VALUE_INPUT_ID));
    assert!(app.register.register_value_input.is_empty());
    assert_eq!(app.input_placeholder(REGISTER_VALUE_INPUT_ID, "00"), "00");

    let _ = app.apply_register_and_step(false);
    settle_backend(&mut app);

    assert_eq!(app.register.selected_register, RegisterName::C);
    assert!(app.register.register_value_input.is_empty());
    assert_eq!(app.snapshot.cpu.registers.b, 0x00);
}

#[test]
fn double_click_register_edit_keeps_replacement_mode_on_next_register() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    let target = RegisterInlineTarget::Mux(RegisterName::B);
    app.enter_inline_register_replacing(target);

    assert!(app.register.register_value_input.is_empty());
    assert_eq!(
        app.input_placeholder(crate::app::REGISTER_INLINE_INPUT_ID, "00"),
        "00"
    );

    let _ = app.apply_inline_register_value(target, false);
    settle_backend(&mut app);

    assert_eq!(app.register.selected_register, RegisterName::C);
    assert!(app.register.register_value_input.is_empty());
    assert_eq!(app.snapshot.cpu.registers.b, 0x00);
}

#[test]
fn value_input_uses_register_a_when_the_register_field_is_empty() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    app.register.register_name_input.clear();
    app.register.register_value_input.clear();
    app.register.selected_register = RegisterName::C;

    let _ = app.update(crate::app::Message::RegisterValueChanged("41".to_owned()));

    assert_eq!(app.register.register_name_input, "A");
    assert_eq!(app.register.selected_register, RegisterName::A);
    assert_eq!(app.register.register_value_input, "41");
}

#[test]
fn esc_in_register_value_discards_pending_value_and_clears_editor() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    app.register.selected_register = RegisterName::B;
    app.snapshot.cpu.registers.b = 0x22;
    app.register.register_name_input = "B".to_owned();
    app.register.register_value_input = "22".to_owned();
    let memory_address_before = app.memory.memory_address_input.clone();
    let memory_value_before = app.memory.memory_value_input.clone();

    let _ = app.update(Message::RegisterValueChanged("41".to_owned()));

    assert_eq!(app.register.active_register_target, None);

    let _ = app.update(Message::EscPressed);

    assert_eq!(app.snapshot.cpu.registers.b, 0x22);
    assert!(app.register.register_name_input.is_empty());
    assert!(app.register.register_value_input.is_empty());
    assert_eq!(app.memory.memory_address_input, memory_address_before);
    assert_eq!(app.memory.memory_value_input, memory_value_before);
}

#[test]
fn invalid_value_does_not_fill_an_empty_register_field() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    app.register.register_name_input.clear();

    let _ = app.update(Message::RegisterValueChanged("GG".to_owned()));

    assert!(app.register.register_name_input.is_empty());
}
