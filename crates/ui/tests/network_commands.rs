use k580_ui::backend::{AppCommand, AppEvent, Emulator, NetworkMode, spawn_emulator};
use std::io::{Read, Write};
use std::time::{Duration, Instant};

#[test]
fn configure_network_command_applies_settings_and_starts_worker() {
    let mut emulator = Emulator::default();

    emulator.handle_command(AppCommand::ConfigureNetwork {
        mode: NetworkMode::Server,
        host: "127.0.0.1".to_owned(),
        port: 0,
    });

    let network = emulator.snapshot().devices.network;
    assert_eq!(network.mode, NetworkMode::Server);
    assert_eq!(network.host, "127.0.0.1");
    assert_eq!(network.port, 0);
}

#[test]
fn clear_network_buffers_command_preserves_endpoint() {
    let mut emulator = Emulator::default();
    emulator.handle_command(AppCommand::ConfigureNetwork {
        mode: NetworkMode::Client,
        host: "127.0.0.1".to_owned(),
        port: 5800,
    });
    emulator.bus_mut().network.queue_received(0x55);
    emulator.handle_command(AppCommand::WritePort(0x03, 0x10));

    emulator.handle_command(AppCommand::ClearNetworkBuffers);

    let network = emulator.snapshot().devices.network;
    assert_eq!(network.host, "127.0.0.1");
    assert_eq!(network.port, 5800);
    assert!(network.rx_buffer.is_empty());
    assert!(network.tx_buffer.is_empty());
}

#[test]
fn clearing_empty_network_buffers_emits_no_events() {
    let mut emulator = Emulator::default();

    let events = emulator.handle_command(AppCommand::ClearNetworkBuffers);

    assert!(events.is_empty());
}

#[test]
fn clearing_empty_network_buffers_does_not_publish_a_worker_error() {
    let occupied = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = occupied.local_addr().unwrap().port();
    let mut emulator = Emulator::default();
    emulator.handle_command(AppCommand::ConfigureNetwork {
        mode: NetworkMode::Server,
        host: "127.0.0.1".to_owned(),
        port,
    });

    for _ in 0..40 {
        std::thread::sleep(Duration::from_millis(25));
        if emulator.snapshot().devices.network.last_error.is_some() {
            break;
        }
    }
    assert!(emulator.snapshot().devices.network.last_error.is_some());

    let events = emulator.handle_command(AppCommand::ClearNetworkBuffers);

    assert!(events.is_empty());
}

#[test]
fn actor_publishes_network_rx_and_tx_without_cpu_ticks() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let handle = spawn_emulator();
    handle.drain_events();
    handle
        .send(AppCommand::ConfigureNetwork {
            mode: NetworkMode::Server,
            host: "127.0.0.1".to_owned(),
            port,
        })
        .unwrap();

    let deadline = Instant::now() + Duration::from_secs(2);
    let mut client = loop {
        match std::net::TcpStream::connect(("127.0.0.1", port)) {
            Ok(client) => break client,
            Err(error) if Instant::now() < deadline => {
                let _ = error;
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(error) => panic!("network worker did not start listening: {error}"),
        }
    };
    client.write_all(b"ABC").unwrap();

    let mut received = None;
    while Instant::now() < deadline {
        for event in handle.drain_events() {
            if let AppEvent::StateChanged(snapshot) = event
                && snapshot.devices.network.rx_buffer == b"ABC"
            {
                received = Some(snapshot.devices.network);
                break;
            }
        }
        if received.is_some() {
            break;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    let received = received.expect("actor did not publish bytes received by TCP worker");
    assert_eq!(received.rx_total, 3);

    handle.send(AppCommand::WritePort(0x03, b'Z')).unwrap();
    client
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    let mut sent = [0_u8; 1];
    client.read_exact(&mut sent).unwrap();
    assert_eq!(sent, [b'Z']);

    let deadline = Instant::now() + Duration::from_secs(2);
    let mut transmitted = false;
    while Instant::now() < deadline {
        for event in handle.drain_events() {
            if let AppEvent::StateChanged(snapshot) = event
                && snapshot.devices.network.tx_buffer == b"Z"
                && snapshot.devices.network.tx_total == 1
            {
                transmitted = true;
                break;
            }
        }
        if transmitted {
            break;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    handle.send(AppCommand::Shutdown).unwrap();

    assert!(
        transmitted,
        "actor did not publish bytes sent by TCP worker"
    );
}
