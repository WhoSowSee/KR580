use k580_ui::devices::{ConnectionState, DeviceStatus, NetworkDevice, NetworkMode};
use std::io::Write;
use std::net::{Shutdown, TcpListener};
use std::time::{Duration, Instant};

#[test]
fn dropping_device_releases_a_waiting_listener_with_runtime_alive() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let reservation = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = reservation.local_addr().unwrap();
    drop(reservation);
    let mut device = NetworkDevice::default();
    device.configure(NetworkMode::Server, "127.0.0.1", address.port());
    device.start_worker(runtime.handle());
    runtime.block_on(async {
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            tokio::task::yield_now().await;
            match TcpListener::bind(address) {
                Err(error) if error.kind() == std::io::ErrorKind::AddrInUse => break,
                Ok(probe) => drop(probe),
                Err(error) => panic!("listener readiness probe failed: {error}"),
            }
            assert!(Instant::now() < deadline, "network listener did not bind");
        }
    });
    drop(device);
    runtime.block_on(async {
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            tokio::task::yield_now().await;
            match TcpListener::bind(address) {
                Ok(_) => break,
                Err(error) => {
                    assert_eq!(error.kind(), std::io::ErrorKind::AddrInUse);
                    assert!(Instant::now() < deadline, "listener survived device drop");
                }
            }
        }
    });
}

#[test]
fn network_worker_lives_until_the_last_device_clone_is_dropped() {
    use tokio::io::AsyncReadExt;

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let mut device = NetworkDevice::default();
        device.configure(
            NetworkMode::Client,
            "127.0.0.1",
            listener.local_addr().unwrap().port(),
        );
        device.start_worker(runtime.handle());
        let (mut peer, _) = tokio::time::timeout(Duration::from_secs(2), listener.accept())
            .await
            .unwrap()
            .unwrap();
        let mut surviving = device.clone();
        drop(device);
        surviving.output_byte(0xA5).unwrap();
        let mut byte = [0];
        tokio::time::timeout(Duration::from_secs(2), peer.read_exact(&mut byte))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(byte, [0xA5]);
        drop(surviving);
        let read = tokio::time::timeout(Duration::from_secs(2), peer.read(&mut byte))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(read, 0);
    });
}

#[test]
fn saturated_rx_delivers_every_byte_in_order_after_cpu_reads() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let mut device = NetworkDevice::default();
    device.configure(
        NetworkMode::Client,
        "127.0.0.1",
        listener.local_addr().unwrap().port(),
    );
    device.start_worker(runtime.handle());
    let (mut peer, _) = listener.accept().unwrap();
    let expected: Vec<_> = (0..131_072).map(|index| (index % 251) as u8).collect();
    peer.write_all(&expected).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while device.state().rx_buffer.len() != 65_536 {
        let state = device.state();
        assert!(
            Instant::now() < deadline,
            "RX {}, total {}, connection {:?}, error {:?}",
            state.rx_buffer.len(),
            state.rx_total,
            state.connection,
            state.last_error
        );
        std::thread::sleep(Duration::from_millis(1));
    }
    let mut received = Vec::with_capacity(expected.len());
    while received.len() != expected.len() {
        assert!(Instant::now() < deadline);
        let available = device.state().rx_buffer.len();
        for _ in 0..available {
            received.push(device.input_byte());
        }
        std::thread::yield_now();
    }
    assert_eq!(received, expected);
    assert_eq!(device.state().rx_total, expected.len() as u64);
    assert_eq!(device.state().last_error, None);
}

#[test]
fn clean_peer_eof_is_published_without_an_error() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let mut device = NetworkDevice::default();
    device.configure(
        NetworkMode::Client,
        "127.0.0.1",
        listener.local_addr().unwrap().port(),
    );
    device.start_worker(runtime.handle());
    let (mut peer, _) = listener.accept().unwrap();
    peer.write_all(&[0x41]).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    while device.state().rx_total != 1 {
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
    peer.shutdown(Shutdown::Both).unwrap();
    while device.state().connection != ConnectionState::Disconnected {
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
    let state = device.state();
    assert_eq!(state.status, DeviceStatus::Disconnected);
    assert_eq!(state.last_error, None);
    assert_eq!(state.rx_buffer, [0x41]);
}
