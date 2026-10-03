use k580_ui::devices::{ConnectionState, DeviceStatus, NetworkDevice, NetworkMode};
use std::io::Write;
use std::net::{Shutdown, TcpListener};
use std::time::{Duration, Instant};

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
