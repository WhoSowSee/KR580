use k580_app::{AppCommand, DeviceStatus, Emulator};
use std::path::PathBuf;
use std::time::Duration;

#[test]
fn attach_floppy_image_command_uses_file_backed_storage() {
    let path = unique_temp_path("app-floppy-image.kpd");
    let mut emulator = Emulator::default();

    emulator.handle_command(AppCommand::SetFloppyDebugBuffer(true));
    emulator.handle_command(AppCommand::AttachFloppyImage(path.clone()));
    emulator.handle_command(AppCommand::WritePort(0x01, b'F'));
    emulator.bus_mut().floppy.flush().unwrap();
    std::thread::sleep(Duration::from_millis(50));

    let floppy = emulator.snapshot().devices.floppy;
    assert_eq!(floppy.path, Some(path.clone()));
    assert_eq!(floppy.visible_buffer, vec![b'F']);
    assert_eq!(floppy.bytes_queued, 1);
    assert!(!floppy.debug_buffer);
    assert_eq!(std::fs::read(&path).unwrap(), vec![b'F']);
    std::fs::remove_file(path).ok();
}

#[test]
fn floppy_debug_command_allows_buffer_only_writes() {
    let mut emulator = Emulator::default();

    emulator.handle_command(AppCommand::SetFloppyDebugBuffer(true));
    emulator.handle_command(AppCommand::WritePort(0x01, b'D'));

    let floppy = emulator.snapshot().devices.floppy;
    assert_eq!(floppy.path, None);
    assert_eq!(floppy.visible_buffer, vec![b'D']);
    assert_eq!(floppy.bytes_queued, 0);
    assert!(floppy.debug_buffer);
    assert_eq!(floppy.last_error, None);
}

#[test]
fn clear_floppy_buffer_command_clears_visible_device_buffers() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let path = unique_temp_path("app-floppy-clear.kpd");
    let mut emulator = Emulator::default();
    emulator
        .bus_mut()
        .floppy
        .attach_file(&path, runtime.handle());
    emulator.handle_command(AppCommand::WritePort(0x01, b'A'));

    emulator.handle_command(AppCommand::ClearFloppyBuffer);

    let floppy = emulator.snapshot().devices.floppy;
    assert!(floppy.visible_buffer.is_empty());
    assert!(floppy.tail_buffer.is_empty());
    assert_eq!(floppy.bytes_queued, 1);
    runtime.block_on(async { tokio::time::sleep(Duration::from_millis(50)).await });
    std::fs::remove_file(path).ok();
}

#[test]
fn detach_floppy_image_command_disconnects_file_backed_storage() {
    let path = unique_temp_path("app-floppy-detach.kpd");
    let mut emulator = Emulator::default();

    emulator.handle_command(AppCommand::AttachFloppyImage(path.clone()));
    emulator.handle_command(AppCommand::WritePort(0x01, b'F'));
    emulator.bus_mut().floppy.flush().unwrap();

    emulator.handle_command(AppCommand::DetachFloppyImage);
    emulator.handle_command(AppCommand::WritePort(0x01, b'Z'));

    let floppy = emulator.snapshot().devices.floppy;
    assert_eq!(floppy.path, None);
    assert_eq!(floppy.visible_buffer, vec![b'F']);
    assert_eq!(floppy.bytes_queued, 1);
    assert_eq!(floppy.status, DeviceStatus::NotReady);
    assert!(!floppy.worker_alive);
    assert!(!floppy.debug_buffer);
    std::fs::remove_file(path).ok();
}

fn unique_temp_path(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("k580-{nanos}-{name}"))
}
