use super::{
    ShortcutContext, captured_register_arrow, command_shortcut_message,
    contextual_shortcut_message, runtime_event_message,
};
use crate::app::{DesktopApp, Message, RegisterMove};
use crate::persistence::{ShortcutAction, ShortcutBinding, ShortcutKey, ShortcutSettings};
use iced::keyboard;
use iced::keyboard::key::{Code, Physical};
use iced::{event, keyboard::Modifiers};
use std::mem::discriminant;

fn char_key(value: &str) -> keyboard::Key {
    keyboard::Key::Character(value.into())
}

fn physical(code: Code) -> Physical {
    Physical::Code(code)
}

fn tab_event(modifiers: Modifiers) -> iced::Event {
    iced::Event::Keyboard(keyboard::Event::KeyPressed {
        key: keyboard::Key::Named(keyboard::key::Named::Tab),
        modified_key: keyboard::Key::Named(keyboard::key::Named::Tab),
        physical_key: physical(Code::Tab),
        location: keyboard::Location::Standard,
        modifiers,
        text: None,
        repeat: false,
    })
}

fn assert_message(actual: Option<Message>, expected: Message) {
    let actual = actual.expect("shortcut should resolve");
    assert_eq!(discriminant(&actual), discriminant(&expected));
}

fn assert_focus_cycle(actual: Option<Message>, backward: bool) {
    assert!(matches!(actual, Some(Message::FocusCycle { backward: actual }) if actual == backward));
}

fn default_settings() -> ShortcutSettings {
    ShortcutSettings::default()
}

#[test]
fn captured_tab_reaches_export_and_import_focus_rings() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);

    app.open_export_modal();
    assert_focus_cycle(
        runtime_event_message(
            &app,
            tab_event(Modifiers::default()),
            event::Status::Captured,
            iced::window::Id::unique(),
        ),
        false,
    );
    assert_focus_cycle(
        runtime_event_message(
            &app,
            tab_event(Modifiers::SHIFT),
            event::Status::Captured,
            iced::window::Id::unique(),
        ),
        true,
    );

    app.close_export_modal();
    app.open_import_modal();
    assert_focus_cycle(
        runtime_event_message(
            &app,
            tab_event(Modifiers::default()),
            event::Status::Captured,
            iced::window::Id::unique(),
        ),
        false,
    );
    assert_focus_cycle(
        runtime_event_message(
            &app,
            tab_event(Modifiers::SHIFT),
            event::Status::Captured,
            iced::window::Id::unique(),
        ),
        true,
    );
}

#[test]
fn captured_plain_arrows_are_forwarded_to_register_navigation() {
    for (key, expected) in [
        (keyboard::key::Named::ArrowUp, RegisterMove::Up),
        (keyboard::key::Named::ArrowDown, RegisterMove::Down),
        (keyboard::key::Named::ArrowLeft, RegisterMove::Left),
        (keyboard::key::Named::ArrowRight, RegisterMove::Right),
    ] {
        let message =
            captured_register_arrow(&keyboard::Key::Named(key), keyboard::Modifiers::default());
        assert!(matches!(
            message,
            Some(Message::RegisterArrowKey(actual)) if actual == expected
        ));
    }
}

#[test]
fn modified_captured_arrows_keep_text_input_behavior() {
    let key = keyboard::Key::Named(keyboard::key::Named::ArrowRight);
    assert!(captured_register_arrow(&key, keyboard::Modifiers::SHIFT).is_none());
    assert!(captured_register_arrow(&key, keyboard::Modifiers::CTRL).is_none());
    assert!(captured_register_arrow(&key, keyboard::Modifiers::ALT).is_none());
}

#[test]
fn captured_ctrl_a_keeps_text_input_select_all() {
    for (typed, code) in [("a", Code::KeyA), ("ф", Code::KeyA)] {
        assert!(
            command_shortcut_message(
                &default_settings(),
                &char_key(typed),
                physical(code),
                Modifiers::COMMAND,
                event::Status::Captured,
                ShortcutContext::General,
            )
            .is_none()
        );
    }
}

#[test]
fn captured_ctrl_v_keeps_text_input_paste() {
    for (typed, code) in [("v", Code::KeyV), ("м", Code::KeyV)] {
        assert!(
            command_shortcut_message(
                &default_settings(),
                &char_key(typed),
                physical(code),
                Modifiers::COMMAND,
                event::Status::Captured,
                ShortcutContext::General,
            )
            .is_none()
        );
    }
}

#[test]
fn ignored_ctrl_v_requests_memory_paste() {
    assert_message(
        command_shortcut_message(
            &default_settings(),
            &char_key("м"),
            physical(Code::KeyV),
            Modifiers::COMMAND,
            event::Status::Ignored,
            ShortcutContext::General,
        ),
        Message::PasteMemoryBytesRequested,
    );
}

#[test]
fn ignored_ctrl_a_still_opens_network_adapter() {
    assert_message(
        command_shortcut_message(
            &default_settings(),
            &char_key("ф"),
            physical(Code::KeyA),
            Modifiers::COMMAND,
            event::Status::Ignored,
            ShortcutContext::General,
        ),
        Message::OpenNetwork,
    );
}

#[test]
fn captured_ctrl_s_still_saves_snapshot() {
    assert_message(
        command_shortcut_message(
            &default_settings(),
            &char_key("ы"),
            physical(Code::KeyS),
            Modifiers::COMMAND,
            event::Status::Captured,
            ShortcutContext::General,
        ),
        Message::SaveSnapshot,
    );
}

#[test]
fn captured_scoped_shortcut_is_not_dispatched_twice() {
    assert!(
        contextual_shortcut_message(
            &default_settings(),
            physical(Code::Enter),
            Modifiers::COMMAND,
            event::Status::Captured,
            ShortcutContext::MemoryEditor,
        )
        .is_none()
    );
}

#[test]
fn ignored_ctrl_v_uses_custom_shortcut_when_assigned() {
    let mut settings = ShortcutSettings::default();
    settings.assign(
        ShortcutAction::OpenMonitor,
        ShortcutBinding::new(true, false, false, ShortcutKey::V),
    );

    assert_message(
        command_shortcut_message(
            &settings,
            &char_key("м"),
            physical(Code::KeyV),
            Modifiers::COMMAND,
            event::Status::Ignored,
            ShortcutContext::General,
        ),
        Message::OpenMonitor,
    );
}
