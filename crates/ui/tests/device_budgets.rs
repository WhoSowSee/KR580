use k580_ui::devices::{
    DeviceError, MonitorDevice, NetworkDevice, NetworkMode, PrinterDevice, StorageDevice,
};

const HISTORY_CAP: usize = 65_536;
const USER_BUFFER_CAP: usize = 1_048_576;

#[test]
fn parked_workers_refuse_full_queues_without_recording_rejected_bytes() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let path = std::env::temp_dir().join(format!("kr580-budget-{}.kpd", std::process::id()));
    let mut storage = StorageDevice::new("test");
    storage.attach_file(&path, runtime.handle()).unwrap();
    let mut network = NetworkDevice::default();
    network.configure(NetworkMode::Client, "127.0.0.1", 1);
    network.start_worker(runtime.handle());
    for value in (0..HISTORY_CAP).map(|index| index as u8) {
        storage.write_byte(value).unwrap();
        network.output_byte(value).unwrap();
    }
    assert_eq!(storage.write_byte(0xAB), Err(DeviceError::Busy));
    assert_eq!(network.output_byte(0xAB), Err(DeviceError::Busy));
    assert_eq!(storage.state().bytes_queued, HISTORY_CAP as u64);
    assert_eq!(storage.state().visible_buffer.last(), Some(&0xFF));
    assert_eq!(network.state().tx_buffer, [0xFF]);
    assert_eq!(storage.flush(), Err(DeviceError::Busy));
    let next_path = path.with_extension("next.kpd");
    storage.attach_file(&next_path, runtime.handle()).unwrap();
    storage.close().unwrap();
    runtime.block_on(async {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        while std::fs::metadata(&path).unwrap().len() != HISTORY_CAP as u64 {
            assert!(
                std::time::Instant::now() < deadline,
                "accepted storage bytes did not drain"
            );
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
    });
    assert_eq!(
        std::fs::read(&path).unwrap(),
        (0..HISTORY_CAP)
            .map(|index| index as u8)
            .collect::<Vec<_>>()
    );
    std::fs::remove_file(path).unwrap();
    assert!(std::fs::read(&next_path).unwrap().is_empty());
    std::fs::remove_file(next_path).unwrap();
}

#[test]
fn bounded_history_keeps_suffix_but_user_buffers_refuse_overflow() {
    let mut monitor = MonitorDevice::default();
    let mut storage = StorageDevice::new("debug");
    storage.set_debug_buffer(true);
    let mut printer = PrinterDevice::default();
    for value in (0..USER_BUFFER_CAP).map(|index| index as u8) {
        monitor.output_byte(value);
        storage.write_byte(value).unwrap();
        printer.output_byte(value).unwrap();
    }
    let suffix: Vec<_> = (USER_BUFFER_CAP - HISTORY_CAP..USER_BUFFER_CAP)
        .map(|index| index as u8)
        .collect();
    assert_eq!(monitor.state().hex_buffer, suffix);
    assert_eq!(storage.write_byte(0xAB), Err(DeviceError::Busy));
    assert_eq!(printer.output_byte(0xAB), Err(DeviceError::Busy));
    assert_eq!(storage.state().visible_buffer.len(), USER_BUFFER_CAP);
    assert_eq!(printer.state().bytes_buffered, USER_BUFFER_CAP as u64);
    assert_eq!(printer.state().spool.last(), Some(&0xFF));
    storage.clear_visible_buffer();
    printer.clear();
    storage.write_byte(0).unwrap();
    printer.output_byte(0).unwrap();
    assert_eq!(storage.state().visible_buffer, [0]);
    assert_eq!(printer.state().spool, [0]);
}
