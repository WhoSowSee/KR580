use crate::{DeviceError, DeviceStatus};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use tokio::net::{TcpListener, TcpStream};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
    pub status: DeviceStatus,
}

#[derive(Clone, Debug)]
pub struct NetworkDevice {
    state: NetworkState,
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
                status: DeviceStatus::Disconnected,
            },
        }
    }
}

impl NetworkDevice {
    pub fn configure(&mut self, mode: NetworkMode, host: impl Into<String>, port: u16) {
        self.state.mode = mode;
        self.state.host = host.into();
        self.state.port = port;
        self.state.connection = ConnectionState::Disconnected;
        self.state.status = DeviceStatus::Disconnected;
    }

    pub async fn connect_client(&mut self) -> Result<(), DeviceError> {
        self.state.mode = NetworkMode::Client;
        self.state.connection = ConnectionState::Connecting;
        match TcpStream::connect((self.state.host.as_str(), self.state.port)).await {
            Ok(_) => {
                self.state.connection = ConnectionState::Connected;
                self.state.status = DeviceStatus::Connected;
                Ok(())
            }
            Err(err) => {
                self.state.connection = ConnectionState::Error(err.to_string());
                self.state.status = DeviceStatus::Error(err.to_string());
                Err(err.into())
            }
        }
    }

    pub async fn bind_server(&mut self) -> Result<(), DeviceError> {
        self.state.mode = NetworkMode::Server;
        match TcpListener::bind((self.state.host.as_str(), self.state.port)).await {
            Ok(_) => {
                self.state.connection = ConnectionState::Listening;
                self.state.status = DeviceStatus::Listening;
                Ok(())
            }
            Err(err) => {
                self.state.connection = ConnectionState::Error(err.to_string());
                self.state.status = DeviceStatus::Error(err.to_string());
                Err(err.into())
            }
        }
    }

    pub fn queue_received(&mut self, value: u8) {
        self.state.rx_buffer.push(value);
        if matches!(self.state.status, DeviceStatus::NoData) {
            self.state.status = DeviceStatus::Connected;
        }
    }

    pub fn output_byte(&mut self, value: u8) -> Result<(), DeviceError> {
        self.state.tx_buffer.push(value);
        match self.state.status {
            DeviceStatus::Connected | DeviceStatus::Listening | DeviceStatus::Ready => Ok(()),
            _ => Err(DeviceError::Disconnected),
        }
    }

    pub fn input_byte(&mut self) -> u8 {
        let mut rx = VecDeque::from(std::mem::take(&mut self.state.rx_buffer));
        let value = rx.pop_front().unwrap_or(0);
        self.state.rx_buffer = rx.into();
        if value == 0 {
            self.state.status = DeviceStatus::NoData;
        }
        value
    }

    pub fn state(&self) -> NetworkState {
        self.state.clone()
    }
}
