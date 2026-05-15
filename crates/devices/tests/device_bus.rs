use k580_core::{PortBus, PortError};
use k580_devices::{DeviceStatus, IoBus, NetworkMode};
use std::path::PathBuf;
use std::time::Duration;

#[test]
fn monitor_port_routes_text_and_status() {
    let mut bus = IoBus::default();
    bus.output(IoBus::MONITOR_PORT, b'A').unwrap();
    let snapshot = bus.snapshot();
    assert_eq!(snapshot.monitor.text, "A");
    assert_eq!(snapshot.monitor.text_cells[0].ch, b'A');
    assert_eq!(snapshot.monitor.text_cells[0].attr, 0x07);
    assert_eq!(bus.input(IoBus::MONITOR_PORT).unwrap(), 0);
}

#[test]
fn monitor_tracks_mode_attributes_and_graphics_layer() {
    let mut bus = IoBus::default();
    bus.output(IoBus::MONITOR_PORT, 0x9A).unwrap();
    bus.output(IoBus::MONITOR_PORT, b'X').unwrap();
    let snapshot = bus.snapshot();
    assert_eq!(snapshot.monitor.current_attr, 0x0A);
    assert_eq!(snapshot.monitor.text_cells[0].attr, 0x0A);

    bus.output(IoBus::MONITOR_PORT, 0x81).unwrap();
    bus.output(IoBus::MONITOR_PORT, 0x3C).unwrap();
    let snapshot = bus.snapshot();
    assert_eq!(snapshot.monitor.mode, k580_devices::MonitorMode::Graphics);
    assert_eq!(snapshot.monitor.pixels.last(), Some(&(1, 0, 0x3C)));
    assert_eq!(snapshot.monitor.last_command, Some(0x3C));
}

#[test]
fn invalid_ports_return_typed_error() {
    let mut bus = IoBus::default();
    assert!(matches!(bus.input(0xFF), Err(PortError::InvalidPort(0xFF))));
    assert!(matches!(
        bus.output(0xFF, 1),
        Err(PortError::InvalidPort(0xFF))
    ));
}

#[test]
fn storage_not_ready_is_explicit_but_visible_buffer_is_updated() {
    let mut bus = IoBus::default();
    assert!(matches!(
        bus.output(IoBus::FLOPPY_PORT, 0xAA),
        Err(PortError::NotReady)
    ));
    let floppy = bus.snapshot().floppy;
    assert_eq!(floppy.visible_buffer, vec![0xAA]);
    assert_eq!(floppy.tail_buffer, vec![0xAA]);
    assert_eq!(floppy.last_error, Some("device is not ready".to_owned()));
}

#[test]
fn storage_worker_writes_to_configured_file() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let path = unique_temp_path("floppy.kpd");
    let mut bus = IoBus::default();
    bus.floppy.attach_file(&path, runtime.handle());
    bus.output(IoBus::FLOPPY_PORT, 0x41).unwrap();
    bus.floppy.flush().unwrap();
    runtime.block_on(async { tokio::time::sleep(Duration::from_millis(50)).await });
    assert_eq!(std::fs::read(&path).unwrap(), vec![0x41]);
    let floppy = bus.snapshot().floppy;
    assert_eq!(floppy.bytes_queued, 1);
    assert!(floppy.worker_alive);
    std::fs::remove_file(path).ok();
}

#[test]
fn network_no_data_is_non_fatal_and_buffers_are_separate() {
    let mut bus = IoBus::default();
    bus.network
        .configure(NetworkMode::Client, "127.0.0.1", 5800);
    assert_eq!(bus.input(IoBus::NETWORK_PORT).unwrap(), 0);
    assert_eq!(bus.snapshot().network.status, DeviceStatus::NoData);

    assert!(matches!(
        bus.output(IoBus::NETWORK_PORT, 0x10),
        Err(PortError::Disconnected)
    ));
    bus.network.queue_received(0x55);
    assert_eq!(bus.input(IoBus::NETWORK_PORT).unwrap(), 0x55);
    assert_eq!(bus.snapshot().network.tx_buffer, vec![0x10]);
}

#[test]
fn printer_buffers_then_exports_as_separate_action() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let path = unique_temp_path("printer.txt");
    let mut bus = IoBus::default();
    bus.printer.attach_export_path(&path, runtime.handle());
    bus.output(IoBus::PRINTER_PORT, b'P').unwrap();
    let printer = bus.snapshot().printer;
    assert_eq!(printer.spool, vec![b'P']);
    assert_eq!(printer.bytes_buffered, 1);
    bus.printer.print_spool().unwrap();
    runtime.block_on(async { tokio::time::sleep(Duration::from_millis(50)).await });
    assert_eq!(std::fs::read(&path).unwrap(), vec![b'P']);
    std::fs::remove_file(path).ok();
}

fn unique_temp_path(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("k580-{nanos}-{name}"))
}
