use k580_ui::backend::{AppCommand, AppEvent, DeviceStatus, Emulator, spawn_emulator};
use std::path::PathBuf;
use std::time::Duration;

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
fn print_printer_pdf_command_starts_pdf_export() {
    let path = unique_temp_path("app-printer.pdf");
    let mut emulator = Emulator::default();
    emulator.handle_command(AppCommand::WritePort(0x04, b'P'));

    emulator.handle_command(AppCommand::PrintPrinterPdf(path.clone()));
    assert_eq!(
        emulator.snapshot().devices.printer.status,
        DeviceStatus::Busy
    );

    for _ in 0..40 {
        std::thread::sleep(Duration::from_millis(25));
        emulator.bus_mut().printer.poll();
        if emulator.snapshot().devices.printer.status != DeviceStatus::Busy {
            break;
        }
    }

    let printer = emulator.snapshot().devices.printer;
    assert_eq!(printer.status, DeviceStatus::Ready);
    assert_eq!(printer.target_path, Some(path.clone()));
    assert!(std::fs::read(&path).unwrap().starts_with(b"%PDF-"));
    std::fs::remove_file(path).ok();
}

#[test]
fn emulator_actor_publishes_completed_printer_export() {
    let path = unique_temp_path("actor-printer.pdf");
    let handle = spawn_emulator();
    let _ = handle.drain_until_state_change(Duration::from_secs(1));
    handle.send(AppCommand::WritePort(0x04, b'P')).unwrap();
    let _ = handle.drain_until_state_change(Duration::from_secs(1));
    handle
        .send(AppCommand::PrintPrinterPdf(path.clone()))
        .unwrap();

    let mut completed = false;
    for _ in 0..40 {
        std::thread::sleep(Duration::from_millis(25));
        for event in handle.drain_events() {
            if let AppEvent::StateChanged(snapshot) = event
                && snapshot.devices.printer.status == DeviceStatus::Ready
                && snapshot.devices.printer.target_path.as_ref() == Some(&path)
            {
                completed = true;
            }
        }
        if completed {
            break;
        }
    }

    assert!(completed);
    assert!(std::fs::read(&path).unwrap().starts_with(b"%PDF-"));
    std::fs::remove_file(path).ok();
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

fn unique_temp_path(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("k580-{nanos}-{name}"))
}
