use crate::app::{DesktopApp, MEMORY_INLINE_INPUT_ID, Message, OPCODE_SEARCH_INPUT_ID};
use std::thread;
use std::time::Duration;

fn app_with_clean_startup() -> DesktopApp {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    for _ in 0..4 {
        thread::sleep(Duration::from_millis(5));
        app.pull_events();
    }
    app
}

#[test]
fn opcode_search_navigation_walks_filtered_results_and_wraps() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    app.toggle_opcode_dropdown(0x1234);
    app.change_opcode_search("MVI".to_owned());

    assert_eq!(app.highlighted_opcode_value(), Some(0x06));

    app.step_opcode_highlight(1);
    assert_eq!(app.highlighted_opcode_value(), Some(0x0E));

    app.step_opcode_highlight(-1);
    assert_eq!(app.highlighted_opcode_value(), Some(0x06));

    app.step_opcode_highlight(-1);
    assert_eq!(app.highlighted_opcode_value(), Some(0x3E));
}

#[test]
fn opcode_search_keyboard_messages_control_highlight() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    app.toggle_opcode_dropdown(0x1234);
    app.change_opcode_search("MVI".to_owned());

    let _ = app.update(Message::FocusCycle { backward: false });
    assert_eq!(app.highlighted_opcode_value(), Some(0x0E));
    assert_eq!(app.focused_input, Some(OPCODE_SEARCH_INPUT_ID));

    let _ = app.update(Message::FocusCycle { backward: true });
    assert_eq!(app.highlighted_opcode_value(), Some(0x06));

    let _ = app.update(Message::ArrowKey(-1));
    assert_eq!(app.highlighted_opcode_value(), Some(0x0E));

    let _ = app.update(Message::ArrowKey(1));
    assert_eq!(app.highlighted_opcode_value(), Some(0x06));
}

#[test]
fn enter_applies_highlighted_opcode() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    app.toggle_opcode_dropdown(0x1234);
    app.change_opcode_search("MVI A".to_owned());

    assert_eq!(app.highlighted_opcode_value(), Some(0x3E));

    let _ = app.update(Message::EnterPressed);

    assert_eq!(app.opcode_dropdown_address, None);
    assert_eq!(app.opcode_search_input, "");
    assert_eq!(app.memory_address_input, "1234");
    assert_eq!(app.memory_value_input, "3E");
}

#[test]
fn pasting_hex_bytes_writes_consecutive_memory_cells_immediately() {
    let mut app = app_with_clean_startup();

    app.change_inline_memory_value(0x0100, "3E 41 D3 03 76".to_owned());
    for _ in 0..10 {
        if app.snapshot.cpu.memory.read(0x0100) == 0x3E {
            break;
        }
        thread::sleep(Duration::from_millis(5));
        app.pull_events();
    }

    assert_eq!(
        &app.snapshot.cpu.memory.as_slice()[0x0100..0x0105],
        &[0x3E, 0x41, 0xD3, 0x03, 0x76]
    );
}

#[test]
fn pasted_hex_bytes_replace_existing_inline_value_after_the_caret() {
    let mut app = app_with_clean_startup();
    app.select_opcode(0x0100, 0xA5);

    app.change_inline_memory_value(0x0100, "A53E 41 D3 03 76".to_owned());
    for _ in 0..10 {
        if app.snapshot.cpu.memory.read(0x0100) == 0x3E {
            break;
        }
        thread::sleep(Duration::from_millis(5));
        app.pull_events();
    }

    assert_eq!(
        &app.snapshot.cpu.memory.as_slice()[0x0100..0x0105],
        &[0x3E, 0x41, 0xD3, 0x03, 0x76]
    );
}

#[test]
fn pasted_hex_bytes_replace_existing_inline_value_before_the_caret() {
    let mut app = app_with_clean_startup();
    app.select_opcode(0x0100, 0xA5);

    app.change_inline_memory_value(0x0100, "3E 41 D3 03 76A5".to_owned());
    for _ in 0..10 {
        if app.snapshot.cpu.memory.read(0x0100) == 0x3E {
            break;
        }
        thread::sleep(Duration::from_millis(5));
        app.pull_events();
    }

    assert_eq!(
        &app.snapshot.cpu.memory.as_slice()[0x0100..0x0105],
        &[0x3E, 0x41, 0xD3, 0x03, 0x76]
    );
}

#[test]
fn value_input_uses_address_zero_when_the_address_field_is_empty() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    app.memory_address_input.clear();
    app.memory_value_input.clear();

    let _ = app.update(Message::MemoryValueChanged("3E".to_owned()));

    assert_eq!(app.memory_address_input, "0000");
    assert_eq!(app.memory_value_input, "3E");
}

#[test]
fn pasted_bytes_use_address_zero_when_the_address_field_is_empty() {
    let mut app = app_with_clean_startup();
    app.memory_address_input.clear();

    let _ = app.update(Message::MemoryValueChanged("3E 41".to_owned()));
    for _ in 0..10 {
        if app.snapshot.cpu.memory.read(0x0000) == 0x3E {
            break;
        }
        thread::sleep(Duration::from_millis(5));
        app.pull_events();
    }

    assert_eq!(app.memory_address_input, "0000");
    assert_eq!(&app.snapshot.cpu.memory.as_slice()[..2], &[0x3E, 0x41]);
}

#[test]
fn invalid_value_does_not_fill_an_empty_address_field() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    app.memory_address_input.clear();

    let _ = app.update(Message::MemoryValueChanged("GG".to_owned()));

    assert!(app.memory_address_input.is_empty());
}

#[test]
fn invalid_hex_byte_sequence_does_not_change_memory() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    app.lang = crate::i18n::Lang::Ru;

    app.change_inline_memory_value(0x0100, "3E nope 76".to_owned());

    assert_eq!(
        &app.snapshot.cpu.memory.as_slice()[0x0100..0x0103],
        &[0x00, 0x00, 0x00]
    );
    assert_eq!(
        app.status,
        "Некорректные байты: используйте HEX-пары через пробел"
    );
    assert!(!app.status.contains("nope"));
}

#[test]
fn invalid_single_pasted_token_reports_a_clear_error() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    app.lang = crate::i18n::Lang::Ru;

    app.change_inline_memory_value(0x0100, "feature".to_owned());

    assert_eq!(
        app.status,
        "Некорректные байты: используйте HEX-пары через пробел"
    );
    assert!(!app.status.contains("feature"));
}

#[test]
fn invalid_short_hex_token_reports_a_clear_error() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    app.lang = crate::i18n::Lang::Ru;

    app.change_inline_memory_value(0x0100, "GG".to_owned());

    assert_eq!(
        app.status,
        "Некорректные байты: используйте HEX-пары через пробел"
    );
}

#[test]
fn overflowing_hex_byte_sequence_does_not_change_memory() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    app.lang = crate::i18n::Lang::Ru;

    app.change_inline_memory_value(0xFFFE, "3E 41 76".to_owned());

    assert_eq!(&app.snapshot.cpu.memory.as_slice()[0xFFFE..], &[0x00, 0x00]);
    assert_eq!(app.status, "Последовательность не помещается в ОЗУ");
}

#[test]
fn inline_memory_enter_keeps_replacement_mode_on_next_cell() {
    let mut app = app_with_clean_startup();
    app.select_opcode(0x0010, 0x3E);
    app.enter_inline_memory_replacing(0x0010);

    assert!(app.memory_inline_value_input.is_empty());
    assert_eq!(app.input_placeholder(MEMORY_INLINE_INPUT_ID, "00"), "3E");

    let _ = app.update(Message::ApplyInlineMemoryValue(0x0010));

    assert_eq!(app.memory_address_input, "0011");
    assert!(app.memory_inline_value_input.is_empty());
    assert_eq!(app.input_placeholder(MEMORY_INLINE_INPUT_ID, "00"), "00");
    assert_eq!(app.snapshot.cpu.memory.read(0x0010), 0x3E);
}
