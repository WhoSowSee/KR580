//! Asynchronous network device.
//!
//! Per `prompt/03_peripherals.md` and `prompt/08_peripheral_edge_cases.md`:
//!
//! * mode is *explicit*: `client` or `server` — never auto-detected;
//! * non-blocking sockets / async tasks;
//! * no-data conditions are normal and non-fatal;
//! * RX and TX buffers are separate from connection state;
//! * disconnect / timeout / refused must be visible.

use crate::error::DeviceError;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;

/// Network operating mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkMode {
    /// Connect to a remote peer.
    Client,
    /// Listen for an inbound connection.
    Server,
}

/// Visible network status snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatus {
    /// Mode in effect.
    pub mode: NetworkMode,
    /// Configured address (`host:port`).
    pub address: String,
    /// Whether the peer side is connected.
    pub connected: bool,
    /// Bytes pending in the RX buffer.
    pub rx_pending: usize,
    /// Total bytes received since start.
    pub rx_total: u64,
    /// Total bytes transmitted since start.
    pub tx_total: u64,
    /// Last error.
    pub last_error: Option<String>,
}

/// Async network device.
pub struct NetworkDevice {
    tx_out: mpsc::UnboundedSender<u8>,
    rx_in: Arc<Mutex<std::collections::VecDeque<u8>>>,
    status: Arc<Mutex<NetworkStatus>>,
    _worker: tokio::task::JoinHandle<()>,
}

impl NetworkDevice {
    /// Spawn a network device worker. Must run inside a Tokio runtime.
    pub fn spawn(mode: NetworkMode, host: String, port: u16) -> Self {
        let address = format!("{host}:{port}");
        let (tx_out, rx_out) = mpsc::unbounded_channel::<u8>();
        let rx_in = Arc::new(Mutex::new(std::collections::VecDeque::<u8>::new()));
        let status = Arc::new(Mutex::new(NetworkStatus {
            mode,
            address: address.clone(),
            connected: false,
            rx_pending: 0,
            rx_total: 0,
            tx_total: 0,
            last_error: None,
        }));
        let rx_in_for_worker = Arc::clone(&rx_in);
        let status_for_worker = Arc::clone(&status);

        let worker = tokio::spawn(async move {
            run_worker(mode, address, rx_out, rx_in_for_worker, status_for_worker).await;
        });

        Self {
            tx_out,
            rx_in,
            status,
            _worker: worker,
        }
    }

    /// Snapshot the current status.
    pub fn snapshot_status(&self) -> NetworkStatus {
        self.status.lock().unwrap().clone()
    }

    /// Enqueue a byte to send. Never blocks the caller.
    pub fn write(&self, byte: u8) -> Result<(), DeviceError> {
        self.tx_out
            .send(byte)
            .map_err(|_| DeviceError::Disconnected)?;
        Ok(())
    }

    /// Read one received byte if available. Returns `0xFF` if the queue is
    /// empty (open-bus convention; the prompt allows non-fatal no-data).
    pub fn read(&self) -> u8 {
        let mut q = self.rx_in.lock().unwrap();
        let b = q.pop_front().unwrap_or(0xFF);
        let mut s = self.status.lock().unwrap();
        s.rx_pending = q.len();
        b
    }
}

async fn run_worker(
    mode: NetworkMode,
    address: String,
    mut rx_out: mpsc::UnboundedReceiver<u8>,
    rx_in: Arc<Mutex<std::collections::VecDeque<u8>>>,
    status: Arc<Mutex<NetworkStatus>>,
) {
    let conn = match mode {
        NetworkMode::Client => match TcpStream::connect(&address).await {
            Ok(s) => Some(s),
            Err(e) => {
                let mut st = status.lock().unwrap();
                st.last_error = Some(format!("connect: {e}"));
                None
            }
        },
        NetworkMode::Server => {
            let listener = match TcpListener::bind(&address).await {
                Ok(l) => l,
                Err(e) => {
                    let mut st = status.lock().unwrap();
                    st.last_error = Some(format!("bind: {e}"));
                    return;
                }
            };
            match listener.accept().await {
                Ok((s, _)) => Some(s),
                Err(e) => {
                    let mut st = status.lock().unwrap();
                    st.last_error = Some(format!("accept: {e}"));
                    None
                }
            }
        }
    };

    let Some(socket) = conn else { return };

    {
        let mut st = status.lock().unwrap();
        st.connected = true;
        st.last_error = None;
    }

    let (mut read_half, mut write_half) = socket.into_split();
    let rx_in_clone = Arc::clone(&rx_in);
    let status_clone = Arc::clone(&status);
    let read_task = tokio::spawn(async move {
        let mut buf = [0u8; 256];
        loop {
            match read_half.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    let mut q = rx_in_clone.lock().unwrap();
                    for b in &buf[..n] {
                        q.push_back(*b);
                    }
                    let mut st = status_clone.lock().unwrap();
                    st.rx_pending = q.len();
                    st.rx_total += n as u64;
                }
                Err(e) => {
                    let mut st = status_clone.lock().unwrap();
                    st.last_error = Some(format!("read: {e}"));
                    break;
                }
            }
        }
        let mut st = status_clone.lock().unwrap();
        st.connected = false;
    });

    while let Some(byte) = rx_out.recv().await {
        if let Err(e) = write_half.write_all(&[byte]).await {
            let mut st = status.lock().unwrap();
            st.last_error = Some(format!("write: {e}"));
            st.connected = false;
            break;
        }
        let mut st = status.lock().unwrap();
        st.tx_total += 1;
    }
    let _ = read_task.await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn server_client_roundtrip() {
        // Pick a free port.
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        drop(listener);

        let server =
            NetworkDevice::spawn(NetworkMode::Server, "127.0.0.1".to_string(), addr.port());
        // Tiny delay so the server is bound before client connects.
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let client =
            NetworkDevice::spawn(NetworkMode::Client, "127.0.0.1".to_string(), addr.port());

        // Wait for the connection to settle.
        for _ in 0..40 {
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
            let s = server.snapshot_status();
            let c = client.snapshot_status();
            if s.connected && c.connected {
                break;
            }
        }

        client.write(b'H').unwrap();
        client.write(b'I').unwrap();

        // Wait until server sees both bytes.
        for _ in 0..40 {
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
            if server.snapshot_status().rx_pending >= 2 {
                break;
            }
        }
        assert_eq!(server.read(), b'H');
        assert_eq!(server.read(), b'I');
    }
}
