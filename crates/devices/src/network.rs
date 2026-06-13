use crate::{DeviceError, DeviceStatus};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::task::AbortHandle;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NetworkMode {
    Client,
    Server,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Listening,
    Refused,
    TimedOut,
    Error(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkState {
    pub mode: NetworkMode,
    pub host: String,
    pub port: u16,
    pub connection: ConnectionState,
    pub rx_buffer: Vec<u8>,
    pub tx_buffer: Vec<u8>,
    pub rx_total: u64,
    pub tx_total: u64,
    pub last_error: Option<String>,
    pub status: DeviceStatus,
}

#[derive(Clone, Debug)]
pub struct NetworkDevice {
    state: NetworkState,
    tx: Option<mpsc::UnboundedSender<u8>>,
    worker_rx: Arc<Mutex<VecDeque<u8>>>,
    worker_status: Arc<Mutex<NetworkWorkerStatus>>,
    worker_abort: Option<AbortHandle>,
}

#[derive(Clone, Debug)]
struct NetworkWorkerStatus {
    connection: ConnectionState,
    status: DeviceStatus,
    rx_total: u64,
    tx_total: u64,
    last_error: Option<String>,
}

impl Default for NetworkWorkerStatus {
    fn default() -> Self {
        Self {
            connection: ConnectionState::Disconnected,
            status: DeviceStatus::Disconnected,
            rx_total: 0,
            tx_total: 0,
            last_error: None,
        }
    }
}

impl Default for NetworkDevice {
    fn default() -> Self {
        Self {
            state: NetworkState {
                mode: NetworkMode::Client,
                host: "127.0.0.1".to_owned(),
                port: 5800,
                connection: ConnectionState::Disconnected,
                rx_buffer: Vec::new(),
                tx_buffer: Vec::new(),
                rx_total: 0,
                tx_total: 0,
                last_error: None,
                status: DeviceStatus::Disconnected,
            },
            tx: None,
            worker_rx: Arc::new(Mutex::new(VecDeque::new())),
            worker_status: Arc::new(Mutex::new(NetworkWorkerStatus::default())),
            worker_abort: None,
        }
    }
}

impl NetworkDevice {
    pub fn configure(&mut self, mode: NetworkMode, host: impl Into<String>, port: u16) {
        self.stop_worker();
        self.state.mode = mode;
        self.state.host = host.into();
        self.state.port = port;
        self.state.connection = ConnectionState::Disconnected;
        self.state.status = DeviceStatus::Disconnected;
        self.state.last_error = None;
        self.tx = None;
        self.worker_rx.lock().unwrap().clear();
        *self.worker_status.lock().unwrap() = NetworkWorkerStatus::default();
    }

    pub fn clear_buffers(&mut self) {
        self.state.rx_buffer.clear();
        self.state.tx_buffer.clear();
        self.worker_rx.lock().unwrap().clear();
    }

    pub fn start_worker(&mut self, handle: &tokio::runtime::Handle) {
        self.stop_worker();
        let (tx, rx_out) = mpsc::unbounded_channel();
        self.worker_rx.lock().unwrap().clear();
        self.worker_status = Arc::new(Mutex::new(NetworkWorkerStatus {
            connection: match self.state.mode {
                NetworkMode::Client => ConnectionState::Connecting,
                NetworkMode::Server => ConnectionState::Listening,
            },
            status: match self.state.mode {
                NetworkMode::Client => DeviceStatus::Busy,
                NetworkMode::Server => DeviceStatus::Listening,
            },
            rx_total: 0,
            tx_total: 0,
            last_error: None,
        }));
        let rx_in = Arc::clone(&self.worker_rx);
        let status = Arc::clone(&self.worker_status);
        let mode = self.state.mode;
        let host = self.state.host.clone();
        let port = self.state.port;
        let task = handle.spawn(async move {
            run_worker(mode, host, port, rx_out, rx_in, status).await;
        });
        self.worker_abort = Some(task.abort_handle());
        self.state.connection = self.worker_status.lock().unwrap().connection.clone();
        self.state.status = self.worker_status.lock().unwrap().status.clone();
        self.state.last_error = None;
        self.tx = Some(tx);
    }

    pub fn queue_received(&mut self, value: u8) {
        self.state.rx_buffer.push(value);
        if matches!(self.state.status, DeviceStatus::NoData) {
            self.state.status = DeviceStatus::Connected;
        }
    }

    pub fn output_byte(&mut self, value: u8) -> Result<(), DeviceError> {
        self.state.tx_buffer.clear();
        self.state.tx_buffer.push(value);
        if let Some(tx) = &self.tx {
            tx.send(value).map_err(|_| {
                self.state.status = DeviceStatus::Disconnected;
                self.state.connection = ConnectionState::Disconnected;
                self.state.last_error = Some(DeviceError::Disconnected.to_string());
                DeviceError::Disconnected
            })?;
            self.state.tx_total += 1;
            self.apply_worker_status();
            return Ok(());
        }
        match self.state.status {
            DeviceStatus::Connected | DeviceStatus::Listening | DeviceStatus::Ready => Ok(()),
            _ => Err(DeviceError::Disconnected),
        }
    }

    pub fn input_byte(&mut self) -> u8 {
        self.apply_worker_status();
        let value = self
            .worker_rx
            .lock()
            .unwrap()
            .pop_front()
            .or_else(|| {
                let mut rx = VecDeque::from(std::mem::take(&mut self.state.rx_buffer));
                let value = rx.pop_front();
                self.state.rx_buffer = rx.into();
                value
            })
            .unwrap_or(0);
        if value == 0 {
            self.state.status = DeviceStatus::NoData;
        }
        value
    }

    pub fn state(&self) -> NetworkState {
        let mut state = self.state.clone();
        let worker = self.worker_status.lock().unwrap();
        if !matches!(worker.status, DeviceStatus::Disconnected) || worker.last_error.is_some() {
            state.connection = worker.connection.clone();
            state.status = worker.status.clone();
            state.rx_total = worker.rx_total;
            state.tx_total = worker.tx_total;
            state.last_error = worker.last_error.clone();
        }
        let worker_rx = self.worker_rx.lock().unwrap();
        if !worker_rx.is_empty() {
            state.rx_buffer.extend(worker_rx.iter().copied());
        }
        state
    }

    fn apply_worker_status(&mut self) {
        let worker = self.worker_status.lock().unwrap().clone();
        if !matches!(worker.status, DeviceStatus::Disconnected) || worker.last_error.is_some() {
            self.state.connection = worker.connection;
            self.state.status = worker.status;
            self.state.rx_total = worker.rx_total;
            self.state.tx_total = worker.tx_total;
            self.state.last_error = worker.last_error;
        }
    }

    fn stop_worker(&mut self) {
        if let Some(worker) = self.worker_abort.take() {
            worker.abort();
        }
        self.tx = None;
    }
}

async fn run_worker(
    mode: NetworkMode,
    host: String,
    port: u16,
    mut rx_out: mpsc::UnboundedReceiver<u8>,
    rx_in: Arc<Mutex<VecDeque<u8>>>,
    status: Arc<Mutex<NetworkWorkerStatus>>,
) {
    let address = format!("{host}:{port}");
    let socket = match mode {
        NetworkMode::Client => match TcpStream::connect(&address).await {
            Ok(socket) => socket,
            Err(error) => {
                set_network_error(&status, error);
                return;
            }
        },
        NetworkMode::Server => {
            let listener = match TcpListener::bind(&address).await {
                Ok(listener) => listener,
                Err(error) => {
                    set_network_error(&status, error);
                    return;
                }
            };
            {
                let mut worker = status.lock().unwrap();
                worker.connection = ConnectionState::Listening;
                worker.status = DeviceStatus::Listening;
                worker.last_error = None;
            }
            match listener.accept().await {
                Ok((socket, _)) => socket,
                Err(error) => {
                    set_network_error(&status, error);
                    return;
                }
            }
        }
    };

    {
        let mut worker = status.lock().unwrap();
        worker.connection = ConnectionState::Connected;
        worker.status = DeviceStatus::Connected;
        worker.last_error = None;
    }

    let (mut read_half, mut write_half) = socket.into_split();
    let read_rx = Arc::clone(&rx_in);
    let read_status = Arc::clone(&status);
    let read_task = tokio::spawn(async move {
        let mut buf = [0u8; 256];
        loop {
            match read_half.read(&mut buf).await {
                Ok(0) => break,
                Ok(count) => {
                    {
                        let mut queue = read_rx.lock().unwrap();
                        queue.extend(buf[..count].iter().copied());
                    }
                    let mut worker = read_status.lock().unwrap();
                    worker.rx_total += count as u64;
                    worker.status = DeviceStatus::Connected;
                }
                Err(error) => {
                    let mut worker = read_status.lock().unwrap();
                    worker.connection = ConnectionState::Error(error.to_string());
                    worker.status = DeviceStatus::Error(error.to_string());
                    worker.last_error = Some(error.to_string());
                    break;
                }
            }
        }
        let mut worker = read_status.lock().unwrap();
        if worker.last_error.is_none() {
            worker.connection = ConnectionState::Disconnected;
            worker.status = DeviceStatus::Disconnected;
        }
    });

    while let Some(byte) = rx_out.recv().await {
        if let Err(error) = write_half.write_all(&[byte]).await {
            let mut worker = status.lock().unwrap();
            worker.connection = ConnectionState::Error(error.to_string());
            worker.status = DeviceStatus::Error(error.to_string());
            worker.last_error = Some(error.to_string());
            break;
        }
        let mut worker = status.lock().unwrap();
        worker.tx_total += 1;
        worker.status = DeviceStatus::Connected;
    }

    let _ = read_task.await;
}

fn set_network_error(status: &Arc<Mutex<NetworkWorkerStatus>>, error: std::io::Error) {
    let mut worker = status.lock().unwrap();
    worker.connection = match error.kind() {
        std::io::ErrorKind::ConnectionRefused => ConnectionState::Refused,
        std::io::ErrorKind::TimedOut => ConnectionState::TimedOut,
        _ => ConnectionState::Error(error.to_string()),
    };
    worker.status = DeviceStatus::Error(error.to_string());
    worker.last_error = Some(error.to_string());
}
