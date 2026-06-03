use crate::{DeviceError, DeviceStatus};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageState {
    pub name: String,
    pub path: Option<PathBuf>,
    pub visible_buffer: Vec<u8>,
    pub status: DeviceStatus,
    pub bytes_queued: u64,
    pub tail_buffer: Vec<u8>,
    pub last_error: Option<String>,
    pub worker_alive: bool,
    pub debug_buffer: bool,
}

#[derive(Debug)]
pub struct StorageDevice {
    state: StorageState,
    tx: Option<mpsc::UnboundedSender<StorageCommand>>,
}

#[derive(Debug)]
enum StorageCommand {
    Write(u8),
    Flush,
    Close,
}

impl StorageDevice {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            state: StorageState {
                name: name.into(),
                path: None,
                visible_buffer: Vec::new(),
                status: DeviceStatus::NotReady,
                bytes_queued: 0,
                tail_buffer: Vec::new(),
                last_error: None,
                worker_alive: false,
                debug_buffer: false,
            },
            tx: None,
        }
    }

    pub fn attach_file(&mut self, path: impl AsRef<Path>, handle: &tokio::runtime::Handle) {
        let path = path.as_ref().to_path_buf();
        let (tx, mut rx) = mpsc::unbounded_channel();
        let worker_path = path.clone();
        handle.spawn(async move {
            let mut file = match tokio::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(worker_path)
                .await
            {
                Ok(file) => file,
                Err(err) => {
                    tracing::error!(error = %err, "storage worker failed to open file");
                    return;
                }
            };
            while let Some(command) = rx.recv().await {
                match command {
                    StorageCommand::Write(byte) => {
                        if let Err(err) = file.write_all(&[byte]).await {
                            tracing::error!(error = %err, "storage worker write failed");
                        }
                    }
                    StorageCommand::Flush => {
                        if let Err(err) = file.flush().await {
                            tracing::error!(error = %err, "storage worker flush failed");
                        }
                    }
                    StorageCommand::Close => break,
                }
            }
        });
        self.state.path = Some(path);
        self.state.status = DeviceStatus::Ready;
        self.state.last_error = None;
        self.state.worker_alive = true;
        self.state.debug_buffer = false;
        self.tx = Some(tx);
    }

    pub fn detach_file(&mut self) {
        if let Some(tx) = self.tx.take() {
            let _ = tx.send(StorageCommand::Close);
        }
        self.state.path = None;
        self.state.status = if self.state.debug_buffer {
            DeviceStatus::Ready
        } else {
            DeviceStatus::NotReady
        };
        self.state.last_error = None;
        self.state.worker_alive = false;
    }

    pub fn write_byte(&mut self, value: u8) -> Result<(), DeviceError> {
        if let Some(tx) = self.tx.as_ref() {
            tx.send(StorageCommand::Write(value)).map_err(|_| {
                self.state.status = DeviceStatus::Disconnected;
                self.state.worker_alive = false;
                self.state.last_error = Some(DeviceError::Disconnected.to_string());
                DeviceError::Disconnected
            })?;
            self.accept_visible_byte(value);
            self.state.bytes_queued += 1;
            self.state.last_error = None;
            return Ok(());
        }

        if self.state.debug_buffer {
            self.state.status = DeviceStatus::Ready;
            self.accept_visible_byte(value);
            self.state.last_error = None;
            return Ok(());
        }

        self.state.status = DeviceStatus::NotReady;
        self.state.last_error = Some(DeviceError::NotReady.to_string());
        Err(DeviceError::NotReady)
    }

    pub fn set_debug_buffer(&mut self, enabled: bool) {
        self.state.debug_buffer = enabled;
        self.state.status = match (enabled, self.tx.is_some()) {
            (true, _) | (false, true) => DeviceStatus::Ready,
            (false, false) => DeviceStatus::NotReady,
        };
        self.state.last_error = None;
    }

    pub fn clear_visible_buffer(&mut self) {
        self.state.visible_buffer.clear();
        self.state.tail_buffer.clear();
    }

    pub fn flush(&mut self) -> Result<(), DeviceError> {
        let Some(tx) = self.tx.as_ref() else {
            self.state.status = DeviceStatus::NotReady;
            self.state.last_error = Some(DeviceError::NotReady.to_string());
            return Err(DeviceError::NotReady);
        };
        tx.send(StorageCommand::Flush).map_err(|_| {
            self.state.status = DeviceStatus::Disconnected;
            self.state.worker_alive = false;
            self.state.last_error = Some(DeviceError::Disconnected.to_string());
            DeviceError::Disconnected
        })
    }

    pub fn close(&mut self) -> Result<(), DeviceError> {
        if let Some(tx) = self.tx.take() {
            tx.send(StorageCommand::Close)
                .map_err(|_| DeviceError::Disconnected)?;
        }
        self.state.status = DeviceStatus::NotReady;
        self.state.worker_alive = false;
        Ok(())
    }

    pub fn input_byte(&self) -> u8 {
        self.state.status.code()
    }

    pub fn state(&self) -> StorageState {
        self.state.clone()
    }

    fn accept_visible_byte(&mut self, value: u8) {
        self.state.visible_buffer.push(value);
        self.state.tail_buffer.push(value);
        if self.state.tail_buffer.len() > 4096 {
            let drop_count = self.state.tail_buffer.len() - 4096;
            self.state.tail_buffer.drain(0..drop_count);
        }
    }
}
