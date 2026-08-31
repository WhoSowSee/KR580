use crate::app::{
    DesktopApp, MEMORY_INLINE_INPUT_ID, Message, OPCODE_LIST_HEIGHT, OPCODE_OPTION_HEIGHT,
    OPCODE_SEARCH_INPUT_ID, StatusKind,
};
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
fn opcode_navigation_keeps_highlight_visible_and_wraps() {
    for (forward, backward, focused_input) in [
        (Message::ArrowKey(-1), Message::ArrowKey(1), None),
        (
            Message::FocusCycle { backward: false },
            Message::FocusCycle { backward: true },
            Some(OPCODE_SEARCH_INPUT_ID),
        ),
    ] {
        let (mut app, _) = DesktopApp::with_initial_path(None);
        let _ = app.update(Message::OpcodeDropdownToggled(0x1234));
        let _ = app.update(Message::OpcodeSearchChanged("MVI".to_owned()));

        for _ in 0..5 {
            let _ = app.update(forward.clone());
        }
        assert_eq!(app.opcode_highlight_index, 5);
        assert_eq!(app.opcode_scroll_offset, 0.0);

        let _ = app.update(forward.clone());
        assert_eq!(app.opcode_highlight_index, 6);
        assert_eq!(
            app.opcode_scroll_offset,
            7.0 * OPCODE_OPTION_HEIGHT - OPCODE_LIST_HEIGHT
        );

        let _ = app.update(forward.clone());
        let _ = app.update(forward);
        assert_eq!(app.opcode_highlight_index, 0);
        assert_eq!(app.opcode_scroll_offset, 0.0);

        let _ = app.update(backward.clone());
        assert_eq!(app.opcode_highlight_index, 7);
        assert_eq!(
            app.opcode_scroll_offset,
            8.0 * OPCODE_OPTION_HEIGHT - OPCODE_LIST_HEIGHT
        );

        app.opcode_highlight_index = 2;
        let _ = app.update(backward);
        assert_eq!(app.opcode_highlight_index, 1);
        assert_eq!(app.opcode_scroll_offset, OPCODE_OPTION_HEIGHT);
        assert_eq!(app.focused_input, focused_input);
    }
}

#[test]
fn opcode_search_and_address_changes_reset_scroll() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    let _ = app.update(Message::OpcodeDropdownToggled(0x1234));
    let _ = app.update(Message::OpcodeScrolled(3_000.0));
    app.opcode_highlight_index = 7;
    let _ = app.update(Message::OpcodeSearchChanged("SUI".to_owned()));
    assert_eq!(app.opcode_highlight_index, 0);
    assert_eq!(app.opcode_scroll_offset, 0.0);

    let _ = app.update(Message::OpcodeScrolled(3_000.0));
    app.opcode_highlight_index = 7;
    let _ = app.update(Message::OpcodeDropdownToggled(0x5678));
    assert_eq!(app.opcode_dropdown_address, Some(0x5678));
    assert_eq!(app.opcode_highlight_index, 0);
    assert_eq!(app.opcode_scroll_offset, 0.0);
}

#[test]
fn enter_applies_highlighted_opcode() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    let _ = app.update(Message::OpcodeDropdownToggled(0x1234));
    let _ = app.update(Message::OpcodeSearchChanged("MVI A".to_owned()));

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
fn value_input_is_shared_with_selected_memory_row_preview() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    app.select_opcode(0x0010, 0x22);

    let _ = app.update(Message::MemoryValueChanged("3E".to_owned()));

    assert_eq!(app.memory_value_input, "3E");
    assert_eq!(app.memory_inline_value_input, "3E");
    assert_eq!(app.snapshot.cpu.memory.read(0x0010), 0x22);
}

#[test]
fn overlong_memory_value_input_is_ignored_without_status_error() {
    let (mut app, _) = DesktopApp::with_initial_path(None);

    let _ = app.update(Message::MemoryValueChanged("20".to_owned()));
    let _ = app.update(Message::MemoryValueChanged("201".to_owned()));
    let _ = app.update(Message::MemoryValueChanged("20G".to_owned()));

    assert_eq!(app.memory_value_input, "20");
    assert!(matches!(app.status_kind, StatusKind::Ready));
}

#[test]
fn overlong_inline_memory_value_input_is_ignored_without_status_error() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    app.select_opcode(0x0100, 0x20);

    let _ = app.update(Message::InlineMemoryValueChanged(0x0100, "201".to_owned()));
    let _ = app.update(Message::InlineMemoryValueChanged(0x0100, "20G".to_owned()));

    assert_eq!(app.memory_inline_value_input, "20");
    assert!(matches!(app.status_kind, StatusKind::Ready));
}

#[test]
fn pasted_bytes_use_selected_memory_cell_without_inline_edit_focus() {
    let mut app = app_with_clean_startup();
    app.select_memory(0x0100);

    let _ = app.update(Message::MemoryBytesPasted(Some("12 15 16".to_owned())));
    for _ in 0..10 {
        if app.snapshot.cpu.memory.read(0x0100) == 0x12 {
            break;
        }
        thread::sleep(Duration::from_millis(5));
        app.pull_events();
    }

    assert_eq!(app.focused_input, None);
    assert_eq!(app.memory_address_input, "0100");
    assert_eq!(app.memory_value_input, "12");
    assert_eq!(app.memory_inline_value_input, "12");
    assert_eq!(
        &app.snapshot.cpu.memory.as_slice()[0x0100..0x0103],
        &[0x12, 0x15, 0x16]
    );
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

#[test]
fn clearing_a_hex_field_reports_a_localized_status() {
    let mut app = app_with_clean_startup();
    app.lang = crate::i18n::Lang::Ru;
    app.memory_address_input = "0100".to_owned();
    app.memory_value_input = String::new();

    let _ = app.apply_memory();

    assert_eq!(app.status, "Неверное шестнадцатеричное значение байта");
    assert!(matches!(app.status_kind, StatusKind::InvalidByteHex));

    app.lang = crate::i18n::Lang::En;
    app.refresh_localized_status();
    assert_eq!(app.status, "Invalid hex byte value");
}

#[test]
fn clearing_the_address_field_reports_the_address_status() {
    let mut app = app_with_clean_startup();
    app.lang = crate::i18n::Lang::Ru;
    app.memory_address_input = String::new();

    let _ = app.jump_memory_address();

    assert_eq!(app.status, "Неверный шестнадцатеричный адрес");
    assert!(matches!(app.status_kind, StatusKind::InvalidAddressHex));
}
