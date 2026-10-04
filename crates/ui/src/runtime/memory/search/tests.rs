use super::DesktopApp;
use crate::app::{MEMORY_ADDRESS_INPUT_ID, MEMORY_INLINE_INPUT_ID, Message};
use iced::keyboard;

fn load_lxi_b_d16(app: &mut DesktopApp) {
    app.snapshot.cpu.memory.write(0x0000, 0x01);
    app.snapshot.cpu.memory.write(0x0001, 0x34);
    app.snapshot.cpu.memory.write(0x0002, 0x12);
}

fn select_address(app: &mut DesktopApp, address: u16) {
    app.memory.memory_address_input = format!("{address:04X}");
    app.refresh_memory_value(address);
}

#[test]
fn jump_memory_to_first_cell_relocates_view_to_start() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    app.memory.memory_address_input = "1234".to_owned();
    app.scroll_memory(0xFFFF as f32);

    let _ = app.update(Message::JumpMemoryTo(0x0000));

    assert_eq!(app.memory.memory_address_input, "0000");
    assert_eq!(app.memory.memory_scroll_first_row, 0);
}

#[test]
fn jump_memory_to_last_cell_relocates_view_to_end() {
    let (mut app, _) = DesktopApp::with_initial_path(None);

    let _ = app.update(Message::JumpMemoryTo(0xFFFF));

    assert_eq!(app.memory.memory_address_input, "FFFF");
    assert_eq!(app.memory.memory_scroll_first_row, u16::MAX);
}

#[test]
fn jump_memory_to_first_cell_exits_stack_view() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    app.enable_stack_view();

    let _ = app.update(Message::JumpMemoryTo(0x0000));

    assert!(!app.memory.view.is_stack());
    assert_eq!(app.memory.memory_address_input, "0000");
}

#[test]
fn alt_enter_on_low_operand_byte_relocates_view_to_target() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    load_lxi_b_d16(&mut app);
    app.memory.memory_address_input = "0001".to_owned();
    app.refresh_memory_value(0x0001);
    app.interaction.keyboard_modifiers = keyboard::Modifiers::ALT;

    let _ = app.update(Message::MemoryCellAction);

    assert_eq!(app.memory.memory_address_input, "1234");
}

#[test]
fn alt_enter_on_high_operand_byte_relocates_view_to_same_target() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    load_lxi_b_d16(&mut app);
    app.memory.memory_address_input = "0002".to_owned();
    app.refresh_memory_value(0x0002);
    app.interaction.keyboard_modifiers = keyboard::Modifiers::ALT;

    let _ = app.update(Message::MemoryCellAction);

    assert_eq!(app.memory.memory_address_input, "1234");
}

#[test]
fn alt_enter_on_opcode_byte_keeps_current_address() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    load_lxi_b_d16(&mut app);
    app.memory.memory_address_input = "0000".to_owned();
    app.refresh_memory_value(0x0000);
    app.interaction.keyboard_modifiers = keyboard::Modifiers::ALT;

    let _ = app.update(Message::MemoryCellAction);

    assert_eq!(app.memory.memory_address_input, "0000");
}

#[test]
fn alt_shift_enter_returns_to_low_address_operand_after_jump() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    load_lxi_b_d16(&mut app);
    select_address(&mut app, 0x0001);
    app.interaction.keyboard_modifiers = keyboard::Modifiers::ALT;

    let _ = app.update(Message::MemoryCellAction);
    app.interaction.keyboard_modifiers = keyboard::Modifiers::ALT | keyboard::Modifiers::SHIFT;
    let _ = app.update(Message::MemoryCellReturn);

    assert_eq!(app.memory.memory_address_input, "0001");
}

#[test]
fn alt_shift_enter_restores_operand_view_position_after_jump() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    let instruction = 0x0104;
    app.snapshot.cpu.memory.write(instruction, 0x01);
    app.snapshot.cpu.memory.write(instruction + 1, 0x34);
    app.snapshot.cpu.memory.write(instruction + 2, 0x12);
    app.scroll_memory(0x0100 as f32 * crate::app::MEMORY_ROW_HEIGHT);
    select_address(&mut app, instruction + 1);
    let original_scroll_offset = app.memory.memory_scroll_offset;
    let original_first_row = app.memory.memory_scroll_first_row;

    let _ = app.update(Message::MemoryCellAction);
    let _ = app.update(Message::MemoryCellReturn);

    assert_eq!(app.memory.memory_address_input, "0105");
    assert_eq!(app.memory.memory_scroll_offset, original_scroll_offset);
    assert_eq!(app.memory.memory_scroll_first_row, original_first_row);
}

#[test]
fn alt_shift_enter_returns_to_high_address_operand_after_jump() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    load_lxi_b_d16(&mut app);
    select_address(&mut app, 0x0002);
    app.interaction.keyboard_modifiers = keyboard::Modifiers::ALT;

    let _ = app.update(Message::MemoryCellAction);
    app.interaction.keyboard_modifiers = keyboard::Modifiers::ALT | keyboard::Modifiers::SHIFT;
    let _ = app.update(Message::MemoryCellReturn);

    assert_eq!(app.memory.memory_address_input, "0002");
}

fn load_out_port(app: &mut DesktopApp, address: u16, port: u8) {
    app.snapshot.cpu.memory.write(address, 0xD3);
    app.snapshot.cpu.memory.write(address.wrapping_add(1), port);
}

#[test]
fn alt_enter_on_out_port_opens_matching_device() {
    let cases = [
        (0x0000u16, 0x00u8, "monitor"),
        (0x0002, 0x01, "floppy"),
        (0x0004, 0x02, "hdd"),
        (0x0006, 0x03, "network"),
        (0x0008, 0x04, "printer"),
    ];
    for (start, port, label) in cases {
        let (mut app, _) = DesktopApp::with_initial_path(None);
        load_out_port(&mut app, start, port);
        let operand = start.wrapping_add(1);
        app.memory.memory_address_input = format!("{operand:04X}");
        app.refresh_memory_value(operand);
        app.interaction.keyboard_modifiers = keyboard::Modifiers::ALT;

        let _ = app.update(Message::MemoryCellAction);

        match label {
            "monitor" => assert!(
                app.panels.monitor_open,
                "monitor not opened for port 0x{port:02X}"
            ),
            "floppy" => assert!(
                app.panels.floppy_open,
                "floppy not opened for port 0x{port:02X}"
            ),
            "hdd" => assert!(app.panels.hdd_open, "hdd not opened for port 0x{port:02X}"),
            "network" => {
                assert!(
                    app.panels.network_open,
                    "network not opened for port 0x{port:02X}"
                )
            }
            "printer" => {
                assert!(
                    app.panels.printer_open,
                    "printer not opened for port 0x{port:02X}"
                )
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn alt_enter_on_unknown_port_does_not_open_device() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    load_out_port(&mut app, 0x0000, 0x7F);
    app.memory.memory_address_input = "0001".to_owned();
    app.refresh_memory_value(0x0001);
    app.interaction.keyboard_modifiers = keyboard::Modifiers::ALT;

    let _ = app.update(Message::MemoryCellAction);

    assert!(!app.panels.monitor_open);
    assert!(!app.panels.floppy_open);
    assert!(!app.panels.hdd_open);
    assert!(!app.panels.network_open);
    assert!(!app.panels.printer_open);
    assert_eq!(app.memory.memory_address_input, "0001");
}

#[test]
fn alt_enter_on_out_opcode_byte_does_not_open_device() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    load_out_port(&mut app, 0x0000, 0x04);
    app.memory.memory_address_input = "0000".to_owned();
    app.refresh_memory_value(0x0000);
    app.interaction.keyboard_modifiers = keyboard::Modifiers::ALT;

    let _ = app.update(Message::MemoryCellAction);

    assert!(!app.panels.printer_open);
}

#[test]
fn alt_enter_on_data_operand_still_falls_through() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    app.snapshot.cpu.memory.write(0x0000, 0x06);
    app.snapshot.cpu.memory.write(0x0001, 0x42);
    app.memory.memory_address_input = "0001".to_owned();
    app.refresh_memory_value(0x0001);
    app.interaction.keyboard_modifiers = keyboard::Modifiers::ALT;

    let _ = app.update(Message::MemoryCellAction);

    assert!(!app.panels.monitor_open);
    assert_eq!(app.memory.memory_address_input, "0001");
}

#[test]
fn alt_shift_enter_on_port_operand_does_not_open_device() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    load_out_port(&mut app, 0x0000, 0x04);
    select_address(&mut app, 0x0001);
    app.interaction.keyboard_modifiers = keyboard::Modifiers::ALT | keyboard::Modifiers::SHIFT;

    let _ = app.update(Message::MemoryCellReturn);

    assert!(!app.panels.printer_open);
    assert_eq!(app.memory.memory_address_input, "0001");
}

#[test]
fn alt_shift_enter_on_data_operand_does_not_enter_inline_editor() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    app.snapshot.cpu.memory.write(0x0000, 0x06);
    app.snapshot.cpu.memory.write(0x0001, 0x42);
    select_address(&mut app, 0x0001);
    app.interaction.keyboard_modifiers = keyboard::Modifiers::ALT | keyboard::Modifiers::SHIFT;

    let _ = app.update(Message::MemoryCellReturn);

    assert_ne!(app.interaction.focused_input, Some(MEMORY_INLINE_INPUT_ID));
    assert_eq!(app.memory.memory_address_input, "0001");
}

#[test]
fn memory_cell_action_from_address_input_jumps_to_typed_cell() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    app.memory.memory_address_input = "1000".to_owned();
    app.interaction.focused_input = Some(MEMORY_ADDRESS_INPUT_ID);

    let _ = app.update(Message::MemoryCellAction);

    assert_eq!(app.memory.memory_address_input, "1000");
    assert_eq!(app.memory.memory_scroll_first_row, 0x1000);
}
