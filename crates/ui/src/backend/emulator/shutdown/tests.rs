use super::Emulator;
use crate::backend::{AppCommand, AppError, AppEvent, CommandResult, RequestId};

fn parked_emulator() -> Emulator {
    Emulator {
        io_runtime: tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap(),
        ..Default::default()
    }
}

fn shutdown(emulator: &mut Emulator) -> Result<CommandResult, AppError> {
    emulator
        .handle_command(AppCommand::Request {
            id: RequestId(1),
            command: Box::new(AppCommand::Shutdown),
        })
        .into_iter()
        .find_map(|event| match event {
            AppEvent::CommandFinished {
                id: RequestId(1),
                result,
            } => Some(result),
            _ => None,
        })
        .expect("Shutdown must publish its request receipt")
}

#[test]
fn shutdown_receipt_follows_all_device_writes() {
    let directory = tempfile::tempdir().unwrap();
    let floppy = directory.path().join("floppy.kpd");
    let hdd = directory.path().join("hdd.kpd");
    let printer = directory.path().join("printer.txt");
    let mut emulator = parked_emulator();
    emulator.handle_command(AppCommand::AttachFloppyImage(floppy.clone()));
    emulator.handle_command(AppCommand::AttachHddFile(hdd.clone()));
    emulator
        .bus
        .printer
        .attach_export_path(&printer, emulator.io_runtime.handle());
    let bytes: Vec<_> = (0..65_536).map(|index| index as u8).collect();
    for &byte in &bytes {
        emulator.bus_mut().floppy.write_byte(byte).unwrap();
        emulator.bus_mut().hdd.write_byte(byte).unwrap();
    }
    emulator.bus_mut().printer.output_byte(b'P').unwrap();
    emulator.bus_mut().printer.print_spool().unwrap();
    assert!(matches!(
        shutdown(&mut emulator),
        Ok(CommandResult::Completed)
    ));
    let floppy_bytes = std::fs::read(floppy).unwrap();
    let hdd_bytes = std::fs::read(hdd).unwrap();
    assert_eq!(floppy_bytes.len(), bytes.len(), "floppy drain");
    assert_eq!(hdd_bytes.len(), bytes.len(), "HDD drain");
    assert_eq!(floppy_bytes, bytes, "floppy bytes");
    assert_eq!(hdd_bytes, bytes, "HDD bytes");
    assert_eq!(std::fs::read(printer).unwrap(), b"P");
    assert!(!emulator.bus().floppy.state().worker_alive);
    assert!(!emulator.bus().hdd.state().worker_alive);
}

#[test]
fn shutdown_reports_writer_failure_after_draining_other_devices() {
    let directory = tempfile::tempdir().unwrap();
    let floppy = directory.path().join("read-only.kpd");
    let hdd = directory.path().join("hdd.kpd");
    std::fs::write(&floppy, b"original").unwrap();
    let mut emulator = parked_emulator();
    emulator
        .bus
        .floppy
        .attach_open_file(
            floppy.clone(),
            std::fs::File::open(&floppy).unwrap(),
            emulator.io_runtime.handle(),
        )
        .unwrap();
    emulator.handle_command(AppCommand::AttachHddFile(hdd.clone()));
    emulator.bus_mut().floppy.write_byte(0xAA).unwrap();
    emulator.bus_mut().hdd.write_byte(0xBB).unwrap();
    assert!(shutdown(&mut emulator).is_err());
    assert!(emulator.bus().floppy.state().last_error.is_some());
    assert_eq!(std::fs::read(floppy).unwrap(), b"original");
    assert_eq!(std::fs::read(hdd).unwrap(), [0xBB]);
    shutdown(&mut emulator).unwrap();
}

#[test]
fn shutdown_reports_printer_export_failure() {
    let directory = tempfile::tempdir().unwrap();
    let mut emulator = parked_emulator();
    emulator
        .bus
        .printer
        .attach_export_path(directory.path(), emulator.io_runtime.handle());
    emulator.bus_mut().printer.output_byte(b'P').unwrap();
    emulator.bus_mut().printer.print_spool().unwrap();
    assert!(shutdown(&mut emulator).is_err());
    assert!(emulator.bus().printer.state().last_error.is_some());
}

#[test]
fn shutdown_retains_writer_ownership_across_close_and_reattach() {
    let directory = tempfile::tempdir().unwrap();
    let first = directory.path().join("first.kpd");
    let second = directory.path().join("second.kpd");
    let mut emulator = parked_emulator();
    emulator.handle_command(AppCommand::AttachFloppyImage(first.clone()));
    emulator.bus_mut().floppy.write_byte(0x11).unwrap();
    emulator.bus_mut().floppy.close().unwrap();
    emulator.handle_command(AppCommand::AttachFloppyImage(second.clone()));
    emulator.bus_mut().floppy.write_byte(0x22).unwrap();
    shutdown(&mut emulator).unwrap();
    assert_eq!(std::fs::read(first).unwrap(), [0x11]);
    assert_eq!(std::fs::read(second).unwrap(), [0x22]);
}
