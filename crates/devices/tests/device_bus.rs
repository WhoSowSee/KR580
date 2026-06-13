use k580_core::{PortBus, PortError};
use k580_devices::{DeviceStatus, IoBus, MonitorPhase, NetworkDevice, NetworkMode, TextCell};
use std::path::PathBuf;
use std::time::Duration;

#[test]
fn monitor_text_command_writes_char_with_colour() {
    let mut bus = IoBus::default();
    // 2-byte text command: bit7=0 + colour 0x40, then char 'A'.
    bus.output(IoBus::MONITOR_PORT, 0x40).unwrap();
    bus.output(IoBus::MONITOR_PORT, b'A').unwrap();
    let snapshot = bus.snapshot();
    assert_eq!(
        snapshot.monitor.text_cells[0],
        TextCell {
            ch: b'A',
            color: 0x40,
        }
    );
    assert_eq!(snapshot.monitor.text_cursor, 1);
    assert_eq!(snapshot.monitor.phase, MonitorPhase::Idle);
    // IN 00h reads device status code (Ready -> 0).
    assert_eq!(bus.input(IoBus::MONITOR_PORT).unwrap(), 0);
}

#[test]
fn monitor_graphics_command_writes_pixel_at_xy() {
    let mut bus = IoBus::default();
    // 3-byte graphics command: bit7=1 + colour 0x7F, X=10, Y=20.
    bus.output(IoBus::MONITOR_PORT, 0xFF).unwrap();
    bus.output(IoBus::MONITOR_PORT, 10).unwrap();
    bus.output(IoBus::MONITOR_PORT, 20).unwrap();
    let snapshot = bus.snapshot();
    assert_eq!(snapshot.monitor.pixels, vec![(10, 20, 0x7F)]);
    assert_eq!(snapshot.monitor.last_command, Some(0xFF));
    assert_eq!(snapshot.monitor.phase, MonitorPhase::Idle);
    // Text layer is untouched by a graphics command.
    assert!(snapshot.monitor.text_cells.iter().all(|c| c.ch == 0));
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
fn storage_not_ready_does_not_update_visible_buffers() {
    let mut bus = IoBus::default();
    assert!(matches!(
        bus.output(IoBus::FLOPPY_PORT, 0xAA),
        Err(PortError::NotReady)
    ));
    let floppy = bus.snapshot().floppy;
    assert!(floppy.visible_buffer.is_empty());
    assert!(floppy.tail_buffer.is_empty());
    assert_eq!(floppy.last_error, Some("device is not ready".to_owned()));
}

#[test]
fn storage_debug_buffer_accepts_bytes_without_attached_file() {
    let mut bus = IoBus::default();
    bus.floppy.set_debug_buffer(true);

    bus.output(IoBus::FLOPPY_PORT, b'D').unwrap();

    let floppy = bus.snapshot().floppy;
    assert_eq!(floppy.visible_buffer, vec![b'D']);
    assert_eq!(floppy.tail_buffer, vec![b'D']);
    assert_eq!(floppy.status, DeviceStatus::Ready);
    assert!(floppy.debug_buffer);
    assert_eq!(floppy.bytes_queued, 0);
    assert_eq!(floppy.last_error, None);
    assert_eq!(bus.input(IoBus::FLOPPY_PORT).unwrap(), 0);
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
fn storage_visible_buffer_can_be_cleared_without_resetting_file_state() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let path = unique_temp_path("floppy-clear.kpd");
    let mut bus = IoBus::default();
    bus.floppy.attach_file(&path, runtime.handle());
    bus.output(IoBus::FLOPPY_PORT, b'A').unwrap();

    bus.floppy.clear_visible_buffer();

    let floppy = bus.snapshot().floppy;
    assert!(floppy.visible_buffer.is_empty());
    assert!(floppy.tail_buffer.is_empty());
    assert_eq!(floppy.bytes_queued, 1);
    assert_eq!(floppy.path, Some(path.clone()));
    assert_eq!(floppy.status, DeviceStatus::Ready);
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
fn network_tx_keeps_only_the_last_output_byte() {
    let mut network = NetworkDevice::default();

    let _ = network.output_byte(0x40);
    let _ = network.output_byte(0x41);

    assert_eq!(network.state().tx_buffer, vec![0x41]);
}

#[test]
fn network_worker_transfers_bytes_over_tcp() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let port = runtime.block_on(async {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        port
    });
    let mut server = NetworkDevice::default();
    server.configure(NetworkMode::Server, "127.0.0.1", port);
    server.start_worker(runtime.handle());
    runtime.block_on(async { tokio::time::sleep(Duration::from_millis(50)).await });

    let mut client = NetworkDevice::default();
    client.configure(NetworkMode::Client, "127.0.0.1", port);
    client.start_worker(runtime.handle());

    runtime.block_on(async {
        for _ in 0..40 {
            tokio::time::sleep(Duration::from_millis(25)).await;
            if server.state().status == DeviceStatus::Connected
                && client.state().status == DeviceStatus::Connected
            {
                break;
            }
        }
    });
    assert_eq!(server.state().status, DeviceStatus::Connected);
    assert_eq!(client.state().status, DeviceStatus::Connected);

    client.output_byte(b'N').unwrap();
    let received = runtime.block_on(async {
        for _ in 0..40 {
            tokio::time::sleep(Duration::from_millis(25)).await;
            let byte = server.input_byte();
            if byte != 0 {
                return byte;
            }
        }
        0
    });
    assert_eq!(received, b'N');
    assert_eq!(server.state().rx_total, 1);
    assert_eq!(client.state().tx_total, 1);
}

#[test]
fn network_buffers_can_be_cleared_without_resetting_connection_settings() {
    let mut network = NetworkDevice::default();
    network.configure(NetworkMode::Server, "0.0.0.0", 5803);
    network.queue_received(0x55);
    let _ = network.output_byte(0x10);

    network.clear_buffers();

    let state = network.state();
    assert_eq!(state.mode, NetworkMode::Server);
    assert_eq!(state.host, "0.0.0.0");
    assert_eq!(state.port, 5803);
    assert!(state.rx_buffer.is_empty());
    assert!(state.tx_buffer.is_empty());
}

#[test]
fn clearing_network_buffers_preserves_the_connection_state() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let occupied = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = occupied.local_addr().unwrap().port();
    let mut network = NetworkDevice::default();
    network.configure(NetworkMode::Server, "127.0.0.1", port);
    network.start_worker(runtime.handle());

    runtime.block_on(async {
        for _ in 0..40 {
            tokio::time::sleep(Duration::from_millis(25)).await;
            if network.state().last_error.is_some() {
                break;
            }
        }
    });
    let before = network.state();
    assert!(before.last_error.is_some());

    network.clear_buffers();

    let after = network.state();
    assert_eq!(after.mode, before.mode);
    assert_eq!(after.host, before.host);
    assert_eq!(after.port, before.port);
    assert_eq!(after.connection, before.connection);
    assert_eq!(after.status, before.status);
    assert_eq!(after.last_error, before.last_error);
}

#[test]
fn reconfiguring_network_aborts_the_previous_worker() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let port = runtime.block_on(async {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        port
    });
    let mut network = NetworkDevice::default();
    network.configure(NetworkMode::Server, "127.0.0.1", port);
    network.start_worker(runtime.handle());
    runtime.block_on(async { tokio::time::sleep(Duration::from_millis(50)).await });

    network.configure(NetworkMode::Client, "127.0.0.1", port + 1);

    runtime.block_on(async { tokio::time::sleep(Duration::from_millis(50)).await });
    let rebound = runtime.block_on(tokio::net::TcpListener::bind(("127.0.0.1", port)));
    assert!(rebound.is_ok());
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
