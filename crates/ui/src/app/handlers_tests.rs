use super::handlers::{alt_shortcut, ctrl_shortcut, plain_shortcut};
use crate::app::Message;
use iced::keyboard;
use iced::keyboard::key::{Code, Physical};
use std::mem::discriminant;

fn char_key(value: &str) -> keyboard::Key {
    keyboard::Key::Character(value.into())
}

fn physical(code: Code) -> Physical {
    Physical::Code(code)
}

fn assert_message(actual: Option<Message>, expected: Message) {
    let actual = actual.expect("shortcut should resolve");
    assert_eq!(discriminant(&actual), discriminant(&expected));
}

fn assert_jump_message(actual: Option<Message>, expected: u16) {
    match actual {
        Some(Message::JumpMemoryTo(address)) => assert_eq!(address, expected),
        other => panic!("expected JumpMemoryTo({expected:04X}), got {other:?}"),
    }
}

#[test]
fn plain_shortcuts_use_physical_key_for_russian_layout() {
    assert_message(
        plain_shortcut(
            &char_key("e"),
            physical(Code::KeyE),
            keyboard::Modifiers::NONE,
        ),
        Message::OpenOpcodePicker,
    );
    assert_message(
        plain_shortcut(
            &char_key("у"),
            physical(Code::KeyE),
            keyboard::Modifiers::NONE,
        ),
        Message::OpenOpcodePicker,
    );
}

#[test]
fn ctrl_shortcuts_use_physical_key_for_russian_layout() {
    let ctrl = keyboard::Modifiers::COMMAND;

    for (typed, code, expected) in [
        ("т", Code::KeyN, Message::NewFile),
        ("щ", Code::KeyO, Message::OpenSnapshot),
        ("ы", Code::KeyS, Message::SaveSnapshot),
        ("ш", Code::KeyI, Message::Import),
        ("у", Code::KeyE, Message::Export),
        ("к", Code::KeyR, Message::ToggleRun),
        ("е", Code::KeyT, Message::StepInstruction),
        ("н", Code::KeyY, Message::StepTact),
        ("р", Code::KeyH, Message::OpenHelp),
        ("в", Code::KeyD, Message::OpenHdd),
        ("а", Code::KeyF, Message::OpenFloppy),
        ("ф", Code::KeyA, Message::OpenNetwork),
        ("з", Code::KeyP, Message::OpenPrinter),
        ("я", Code::KeyZ, Message::Undo),
        ("ь", Code::KeyM, Message::OpenMonitor),
    ] {
        assert_message(
            ctrl_shortcut(&char_key(typed), physical(code), ctrl),
            expected,
        );
    }
}

#[test]
fn alt_shortcuts_jump_to_memory_boundaries() {
    for (typed, code, expected) in [
        ("q", Code::KeyQ, 0x0000),
        ("й", Code::KeyQ, 0x0000),
        ("e", Code::KeyE, 0xFFFF),
        ("у", Code::KeyE, 0xFFFF),
    ] {
        assert_jump_message(
            alt_shortcut(&char_key(typed), physical(code), keyboard::Modifiers::ALT),
            expected,
        );
    }
}

#[test]
fn shifted_and_alt_ctrl_shortcuts_use_physical_key_for_russian_layout() {
    assert_message(
        ctrl_shortcut(
            &char_key("Ы"),
            physical(Code::KeyS),
            keyboard::Modifiers::COMMAND | keyboard::Modifiers::SHIFT,
        ),
        Message::SaveSnapshotAs,
    );
    assert_message(
        ctrl_shortcut(
            &char_key("К"),
            physical(Code::KeyR),
            keyboard::Modifiers::COMMAND | keyboard::Modifiers::SHIFT,
        ),
        Message::ResetRam,
    );
    assert_message(
        ctrl_shortcut(
            &char_key("П"),
            physical(Code::KeyG),
            keyboard::Modifiers::COMMAND | keyboard::Modifiers::SHIFT,
        ),
        Message::ResetCpu,
    );
    assert_message(
        ctrl_shortcut(
            &char_key("Р"),
            physical(Code::KeyH),
            keyboard::Modifiers::COMMAND | keyboard::Modifiers::SHIFT,
        ),
        Message::ClearHalt,
    );
    assert_message(
        ctrl_shortcut(
            &char_key("С"),
            physical(Code::KeyC),
            keyboard::Modifiers::COMMAND | keyboard::Modifiers::SHIFT,
        ),
        Message::ToggleStackView,
    );
    assert_message(
        ctrl_shortcut(
            &char_key("Я"),
            physical(Code::KeyZ),
            keyboard::Modifiers::COMMAND | keyboard::Modifiers::SHIFT,
        ),
        Message::Redo,
    );
    assert_message(
        ctrl_shortcut(
            &char_key("б"),
            physical(Code::Comma),
            keyboard::Modifiers::COMMAND,
        ),
        Message::OpenSettings,
    );
}
