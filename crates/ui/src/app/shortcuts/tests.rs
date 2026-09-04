use super::{ShortcutContext, binding_from_event, shortcut_message, shortcut_message_for_context};
use crate::app::{DesktopApp, MEMORY_ADDRESS_INPUT_ID, Message};
use crate::persistence::{
    ShortcutAction, ShortcutBinding, ShortcutKey, ShortcutSettings, default_binding,
};
use iced::keyboard;
use iced::keyboard::key::{Code, Physical};
use std::mem::discriminant;

fn physical(code: Code) -> Physical {
    Physical::Code(code)
}

fn assert_message(actual: Option<Message>, expected: Message) {
    let actual = actual.expect("shortcut should resolve");
    assert_eq!(discriminant(&actual), discriminant(&expected));
}

#[test]
fn default_shortcuts_use_physical_qwerty_positions() {
    assert_message(
        shortcut_message(
            &ShortcutSettings::default(),
            physical(Code::KeyM),
            keyboard::Modifiers::COMMAND,
        ),
        Message::OpenMonitor,
    );
}

#[test]
fn default_enter_shortcuts_resolve_memory_actions() {
    for (modifiers, context, message) in [
        (
            keyboard::Modifiers::COMMAND,
            ShortcutContext::MemoryCell,
            Message::MemoryCellReplace,
        ),
        (
            keyboard::Modifiers::COMMAND,
            ShortcutContext::MemoryEditor,
            Message::MemoryPatternSearch,
        ),
        (
            keyboard::Modifiers::COMMAND | keyboard::Modifiers::SHIFT,
            ShortcutContext::MemoryEditor,
            Message::MemoryPatternSearch,
        ),
        (
            keyboard::Modifiers::ALT,
            ShortcutContext::General,
            Message::MemoryCellAction,
        ),
        (
            keyboard::Modifiers::ALT | keyboard::Modifiers::SHIFT,
            ShortcutContext::General,
            Message::MemoryCellReturn,
        ),
    ] {
        assert_message(
            shortcut_message_for_context(
                &ShortcutSettings::default(),
                physical(Code::Enter),
                modifiers,
                context,
            ),
            message,
        );
    }
}

#[test]
fn default_memory_cell_return_shortcut_label_is_alt_shift_enter() {
    assert_eq!(
        default_binding(ShortcutAction::MemoryCellReturn).map(ShortcutBinding::label),
        Some("Shift+Alt+Enter".to_owned())
    );
}

#[test]
fn default_memory_cell_shortcut_label_is_alt_enter() {
    assert_eq!(
        default_binding(ShortcutAction::MemoryCellAction).map(ShortcutBinding::label),
        Some("Alt+Enter".to_owned())
    );
}

#[test]
fn custom_memory_cell_shortcut_resolves_without_alt_modifier() {
    let mut settings = ShortcutSettings::default();
    settings.assign(
        ShortcutAction::MemoryCellAction,
        ShortcutBinding::new(true, false, false, ShortcutKey::J),
    );

    assert_message(
        shortcut_message(
            &settings,
            physical(Code::KeyJ),
            keyboard::Modifiers::COMMAND,
        ),
        Message::MemoryCellAction,
    );
    assert!(shortcut_message(&settings, physical(Code::Enter), keyboard::Modifiers::ALT).is_none());
}

#[test]
fn custom_memory_cell_shortcut_resolves_alt_letter_binding() {
    let mut settings = ShortcutSettings::default();
    settings.assign(
        ShortcutAction::MemoryCellAction,
        ShortcutBinding::new(false, false, true, ShortcutKey::L),
    );

    assert_message(
        shortcut_message(&settings, physical(Code::KeyL), keyboard::Modifiers::ALT),
        Message::MemoryCellAction,
    );
}

#[test]
fn custom_memory_cell_return_shortcut_resolves_alt_letter_binding() {
    let mut settings = ShortcutSettings::default();
    settings.assign(
        ShortcutAction::MemoryCellReturn,
        ShortcutBinding::new(false, false, true, ShortcutKey::K),
    );

    assert_message(
        shortcut_message(&settings, physical(Code::KeyK), keyboard::Modifiers::ALT),
        Message::MemoryCellReturn,
    );
    assert!(
        shortcut_message(
            &settings,
            physical(Code::Enter),
            keyboard::Modifiers::ALT | keyboard::Modifiers::SHIFT,
        )
        .is_none()
    );
}

#[test]
fn memory_replace_and_pattern_search_bindings_are_independent() {
    let mut settings = ShortcutSettings::default();
    let shared = ShortcutBinding::new(true, false, false, ShortcutKey::J);
    settings.assign(ShortcutAction::MemoryPatternSearch, shared);
    settings.assign(ShortcutAction::MemoryCellReplace, shared);

    assert_eq!(
        settings.binding(ShortcutAction::MemoryPatternSearch),
        Some(shared)
    );
    assert_eq!(
        settings.binding(ShortcutAction::MemoryCellReplace),
        Some(shared)
    );

    settings.assign(ShortcutAction::OpenMonitor, shared);
    assert_eq!(settings.binding(ShortcutAction::MemoryPatternSearch), None);
    assert_eq!(settings.binding(ShortcutAction::MemoryCellReplace), None);
}

#[test]
fn pattern_search_action_uses_memory_editor_focus() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    app.memory_address_input = "FF".to_owned();
    app.focused_input = Some(MEMORY_ADDRESS_INPUT_ID);

    let _ = app.update(Message::MemoryPatternSearch);

    assert_eq!(app.memory_address_input, "01FF");
}

#[test]
fn custom_shortcuts_support_all_three_modifiers() {
    let mut settings = ShortcutSettings::default();
    settings.assign(
        ShortcutAction::OpenMonitor,
        ShortcutBinding::new(true, true, true, ShortcutKey::M),
    );

    assert_message(
        shortcut_message(
            &settings,
            physical(Code::KeyM),
            keyboard::Modifiers::COMMAND | keyboard::Modifiers::SHIFT | keyboard::Modifiers::ALT,
        ),
        Message::OpenMonitor,
    );
    assert!(
        shortcut_message(
            &settings,
            physical(Code::KeyM),
            keyboard::Modifiers::COMMAND,
        )
        .is_none()
    );
}

#[test]
fn assigning_existing_binding_unbinds_previous_action() {
    let mut settings = ShortcutSettings::default();
    let printer = default_binding(ShortcutAction::OpenPrinter).unwrap();
    settings.assign(ShortcutAction::OpenMonitor, printer);

    assert_eq!(settings.binding(ShortcutAction::OpenPrinter), None);
    assert_message(
        shortcut_message(
            &settings,
            physical(Code::KeyP),
            keyboard::Modifiers::COMMAND,
        ),
        Message::OpenMonitor,
    );
}

#[test]
fn captured_binding_keeps_command_shift_alt_flags() {
    let binding = binding_from_event(
        physical(Code::KeyM),
        keyboard::Modifiers::COMMAND | keyboard::Modifiers::SHIFT | keyboard::Modifiers::ALT,
    )
    .unwrap();

    assert!(binding.modifiers.ctrl);
    assert!(binding.modifiers.shift);
    assert!(binding.modifiers.alt);
    assert_eq!(binding.key, ShortcutKey::M);
}

#[test]
fn captured_binding_supports_enter_key() {
    let binding = binding_from_event(physical(Code::Enter), keyboard::Modifiers::ALT).unwrap();

    assert!(binding.modifiers.alt);
    assert_eq!(binding.key, ShortcutKey::Enter);
}
