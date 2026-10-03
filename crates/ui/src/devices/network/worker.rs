use super::{ConnectionState, DeviceStatus, NetworkMode, NetworkWorkerStatus, RX_BUFFER_CAP};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Notify, mpsc};

pub(super) async fn run_worker(
    mode: NetworkMode,
    host: String,
    port: u16,
    mut rx_out: mpsc::Receiver<u8>,
    rx_in: Arc<Mutex<VecDeque<u8>>>,
    status: Arc<Mutex<NetworkWorkerStatus>>,
    rx_space: Arc<Notify>,
) {
    let address = format!("{host}:{port}");
    let connected = match mode {
        NetworkMode::Client => TcpStream::connect(&address).await,
        NetworkMode::Server => match TcpListener::bind(&address).await {
            Ok(listener) => listener.accept().await.map(|(socket, _)| socket),
            Err(error) => Err(error),
        },
    };
    let mut socket = match connected {
        Ok(socket) => socket,
        Err(error) => {
            set_network_error(&status, error);
            return;
        }
    };
    {
        let mut worker = status.lock().unwrap();
        worker.revision = worker.revision.wrapping_add(1);
        worker.connection = ConnectionState::Connected;
        worker.status = DeviceStatus::Connected;
        worker.last_error = None;
    }
    let (mut read_half, mut write_half) = socket.split();
    let reader = async {
        let mut buf = [0u8; 256];
        loop {
            let remaining = RX_BUFFER_CAP.saturating_sub(rx_in.lock().unwrap().len());
            if remaining == 0 {
                rx_space.notified().await;
                continue;
            }
            let limit = remaining.min(buf.len());
            let count = read_half.read(&mut buf[..limit]).await?;
            if count == 0 {
                return Ok::<(), std::io::Error>(());
            }
            let mut offset = 0;
            while offset < count {
                {
                    let mut queue = rx_in.lock().unwrap();
                    let free = RX_BUFFER_CAP.saturating_sub(queue.len());
                    let end = count.min(offset + free);
                    queue.extend(buf[offset..end].iter().copied());
                    offset = end;
                }
                if offset < count {
                    rx_space.notified().await;
                }
            }
            let mut worker = status.lock().unwrap();
            worker.revision = worker.revision.wrapping_add(1);
            worker.rx_total += count as u64;
            worker.status = DeviceStatus::Connected;
            worker.last_error = None;
        }
    };
    let writer = async {
        let mut bytes = Vec::with_capacity(4096);
        while rx_out.recv_many(&mut bytes, 4096).await != 0 {
            write_half.write_all(&bytes).await?;
            let mut worker = status.lock().unwrap();
            worker.revision = worker.revision.wrapping_add(1);
            worker.tx_total += bytes.len() as u64;
            bytes.clear();
            worker.status = DeviceStatus::Connected;
            worker.last_error = None;
        }
        Ok::<(), std::io::Error>(())
    };
    let result = tokio::select! {
        result = reader => result,
        result = writer => result,
    };
    match result {
        Err(error) => set_network_error(&status, error),
        Ok(()) => {
            let mut worker = status.lock().unwrap();
            worker.revision = worker.revision.wrapping_add(1);
            worker.connection = ConnectionState::Disconnected;
            worker.status = DeviceStatus::Disconnected;
        }
    }
}

fn set_network_error(status: &Arc<Mutex<NetworkWorkerStatus>>, error: std::io::Error) {
    let mut worker = status.lock().unwrap();
    worker.revision = worker.revision.wrapping_add(1);
    worker.connection = match error.kind() {
        std::io::ErrorKind::ConnectionRefused => ConnectionState::Refused,
        std::io::ErrorKind::TimedOut => ConnectionState::TimedOut,
        _ => ConnectionState::Error(error.to_string()),
    };
    worker.status = DeviceStatus::Error(error.to_string());
    worker.last_error = Some(error.to_string());
}
