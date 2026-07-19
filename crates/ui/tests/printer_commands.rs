use k580_ui::backend::{AppCommand, Emulator};

#[test]
fn clear_printer_buffer_command_clears_spool() {
    let mut emulator = Emulator::default();
    emulator.handle_command(AppCommand::WritePort(0x04, b'P'));

    emulator.handle_command(AppCommand::ClearPrinterBuffer);

    let printer = emulator.snapshot().devices.printer;
    assert!(printer.spool.is_empty());
    assert_eq!(printer.bytes_buffered, 0);
}

#[test]
fn printer_demo_program_writes_test_line_to_port_four() {
    let program = vec![
        0x21, 0x0F, 0x00, 0x7E, 0xB7, 0xCA, 0x0E, 0x00, 0xD3, 0x04, 0x23, 0xC3, 0x03, 0x00, 0x76,
        b'T', b'E', b'S', b'T', b' ', b'P', b'R', b'I', b'N', b'T', b'E', b'R', b'\r', b'\n', 0x00,
    ];
    let mut emulator = Emulator::default();
    emulator.handle_command(AppCommand::SetMemoryBlock {
        start: 0,
        values: program,
    });

    for _ in 0..200 {
        emulator.handle_command(AppCommand::StepInstruction);
        if emulator.cpu().halted {
            break;
        }
    }

    assert!(emulator.cpu().halted);
    assert_eq!(
        emulator.snapshot().devices.printer.spool,
        b"TEST PRINTER\r\n"
    );
}
