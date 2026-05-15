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
        self.tx = Some(tx);
    }

    pub fn write_byte(&mut self, value: u8) -> Result<(), DeviceError> {
        self.state.visible_buffer.push(value);
        let tx = self.tx.as_ref().ok_or(DeviceError::NotReady)?;
        tx.send(StorageCommand::Write(value))
            .map_err(|_| DeviceError::Disconnected)
    }

    pub fn flush(&mut self) -> Result<(), DeviceError> {
        let tx = self.tx.as_ref().ok_or(DeviceError::NotReady)?;
        tx.send(StorageCommand::Flush)
            .map_err(|_| DeviceError::Disconnected)
    }

    pub fn close(&mut self) -> Result<(), DeviceError> {
        if let Some(tx) = self.tx.take() {
            tx.send(StorageCommand::Close)
                .map_err(|_| DeviceError::Disconnected)?;
        }
        self.state.status = DeviceStatus::NotReady;
        Ok(())
    }

    pub fn input_byte(&self) -> u8 {
        self.state.status.code()
    }

    pub fn state(&self) -> StorageState {
        self.state.clone()
    }
}
