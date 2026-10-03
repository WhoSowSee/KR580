use crate::devices::{DeviceError, DeviceStatus};
use k580_core::Memory64K;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tokio::sync::{Notify, mpsc};
use tokio::task::AbortHandle;

mod worker;

use worker::run_worker;

const TX_QUEUE_CAP: usize = 65_536;
const RX_BUFFER_CAP: usize = Memory64K::SIZE;

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
    tx: Option<mpsc::Sender<u8>>,
    worker_rx: Arc<Mutex<VecDeque<u8>>>,
    worker_status: Arc<Mutex<NetworkWorkerStatus>>,
    worker_abort: Option<AbortHandle>,
    rx_space: Arc<Notify>,
    generation: u64,
}

#[derive(Clone, Debug)]
struct NetworkWorkerStatus {
    revision: u64,
    connection: ConnectionState,
    status: DeviceStatus,
    rx_total: u64,
    tx_total: u64,
    last_error: Option<String>,
}

impl Default for NetworkWorkerStatus {
    fn default() -> Self {
        Self {
            revision: 0,
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
            rx_space: Arc::new(Notify::new()),
            generation: 0,
        }
    }
}

impl NetworkDevice {
    pub fn configure(&mut self, mode: NetworkMode, host: impl Into<String>, port: u16) {
        self.generation = self.generation.wrapping_add(1);
        self.stop_worker();
        self.state.mode = mode;
        self.state.host = host.into();
        self.state.port = port;
        self.state.connection = ConnectionState::Disconnected;
        self.state.status = DeviceStatus::Disconnected;
        self.state.last_error = None;
        self.tx = None;
        self.worker_rx = Arc::new(Mutex::new(VecDeque::new()));
        self.worker_status = Arc::new(Mutex::new(NetworkWorkerStatus::default()));
        self.rx_space = Arc::new(Notify::new());
    }

    pub fn clear_buffers(&mut self) {
        self.state.rx_buffer.clear();
        self.state.tx_buffer.clear();
        self.worker_rx.lock().unwrap().clear();
        self.rx_space.notify_one();
    }

    pub fn start_worker(&mut self, handle: &tokio::runtime::Handle) {
        self.generation = self.generation.wrapping_add(1);
        self.stop_worker();
        let (tx, rx_out) = mpsc::channel(TX_QUEUE_CAP);
        self.worker_rx = Arc::new(Mutex::new(VecDeque::new()));
        self.rx_space = Arc::new(Notify::new());
        self.worker_status = Arc::new(Mutex::new(NetworkWorkerStatus {
            revision: 0,
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
        let rx_space = Arc::clone(&self.rx_space);
        let task = handle.spawn(async move {
            run_worker(mode, host, port, rx_out, rx_in, status, rx_space).await;
        });
        self.worker_abort = Some(task.abort_handle());
        self.state.connection = self.worker_status.lock().unwrap().connection.clone();
        self.state.status = self.worker_status.lock().unwrap().status.clone();
        self.state.last_error = None;
        self.tx = Some(tx);
    }

    pub fn queue_received(&mut self, value: u8) -> Result<(), DeviceError> {
        let mut queue = self.worker_rx.lock().unwrap();
        if queue.len() == RX_BUFFER_CAP {
            return Err(DeviceError::Busy);
        }
        queue.push_back(value);
        if self.state.status == DeviceStatus::NoData {
            self.state.status = DeviceStatus::Connected;
        }
        Ok(())
    }

    pub fn output_byte(&mut self, value: u8) -> Result<(), DeviceError> {
        if let Some(tx) = &self.tx {
            let result = super::queue::enqueue(tx, value);
            if let Err(error) = result {
                let mut worker = self.worker_status.lock().unwrap();
                worker.revision = worker.revision.wrapping_add(1);
                worker.status = if error == DeviceError::Busy {
                    DeviceStatus::Busy
                } else {
                    worker.connection = ConnectionState::Disconnected;
                    DeviceStatus::Disconnected
                };
                worker.last_error = Some(error.to_string());
                return Err(error);
            }
            self.state.tx_buffer.clear();
            self.state.tx_buffer.push(value);
            self.apply_worker_status();
            return Ok(());
        }
        self.state.tx_buffer.clear();
        self.state.tx_buffer.push(value);
        match self.state.status {
            DeviceStatus::Connected | DeviceStatus::Listening | DeviceStatus::Ready => Ok(()),
            _ => Err(DeviceError::Disconnected),
        }
    }

    pub fn input_byte(&mut self) -> u8 {
        self.apply_worker_status();
        let value = self.worker_rx.lock().unwrap().pop_front();
        self.rx_space.notify_one();
        if value.is_none() {
            self.state.status = DeviceStatus::NoData;
        }
        value.unwrap_or(0)
    }

    pub(crate) fn revision(&self) -> (u64, u64) {
        (self.generation, self.worker_status.lock().unwrap().revision)
    }

    pub fn state(&self) -> NetworkState {
        let mut state = self.state.clone();
        let worker = self.worker_status.lock().unwrap();
        if self.worker_abort.is_some() {
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
        if self.worker_abort.is_some() {
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
