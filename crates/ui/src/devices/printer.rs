use crate::devices::{DeviceError, DeviceStatus};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;

mod pdf;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrinterState {
    pub spool: Vec<u8>,
    pub target_path: Option<PathBuf>,
    pub status: DeviceStatus,
    pub bytes_buffered: u64,
    pub last_error: Option<String>,
}

#[derive(Debug)]
pub struct PrinterDevice {
    state: PrinterState,
    tx: Option<mpsc::UnboundedSender<Vec<u8>>>,
    completion_tx: mpsc::UnboundedSender<PrintCompletion>,
    completion_rx: mpsc::UnboundedReceiver<PrintCompletion>,
}

#[derive(Debug)]
struct PrintCompletion {
    path: PathBuf,
    result: Result<(), String>,
}

impl Default for PrinterDevice {
    fn default() -> Self {
        let (completion_tx, completion_rx) = mpsc::unbounded_channel();
        Self {
            state: PrinterState {
                spool: Vec::new(),
                target_path: None,
                status: DeviceStatus::Ready,
                bytes_buffered: 0,
                last_error: None,
            },
            tx: None,
            completion_tx,
            completion_rx,
        }
    }
}

impl PrinterDevice {
    pub fn output_byte(&mut self, value: u8) {
        self.state.spool.push(value);
        self.state.bytes_buffered += 1;
    }

    pub fn attach_export_path(&mut self, path: impl AsRef<Path>, handle: &tokio::runtime::Handle) {
        let path = path.as_ref().to_path_buf();
        let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();
        let worker_path = path.clone();
        handle.spawn(async move {
            while let Some(bytes) = rx.recv().await {
                match tokio::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&worker_path)
                    .await
                {
                    Ok(mut file) => {
                        if let Err(err) = file.write_all(&bytes).await {
                            tracing::error!(error = %err, "printer export write failed");
                            continue;
                        }
                        if let Err(err) = file.flush().await {
                            tracing::error!(error = %err, "printer export flush failed");
                        }
                    }
                    Err(err) => tracing::error!(error = %err, "printer export open failed"),
                }
            }
        });
        self.state.target_path = Some(path);
        self.tx = Some(tx);
    }

    pub fn print_spool(&mut self) -> Result<(), DeviceError> {
        let Some(tx) = self.tx.clone() else {
            self.state.status = DeviceStatus::NotReady;
            self.state.last_error = Some(DeviceError::NotReady.to_string());
            return Err(DeviceError::NotReady);
        };
        tx.send(self.state.spool.clone()).map_err(|_| {
            self.state.status = DeviceStatus::Disconnected;
            self.state.last_error = Some(DeviceError::Disconnected.to_string());
            DeviceError::Disconnected
        })?;
        self.state.last_error = None;
        Ok(())
    }

    pub fn print_to_pdf(
        &mut self,
        path: impl AsRef<Path>,
        handle: &tokio::runtime::Handle,
    ) -> Result<(), DeviceError> {
        if self.state.status == DeviceStatus::Busy {
            return Err(DeviceError::Busy);
        }
        let path = path.as_ref().to_path_buf();
        let spool = self.state.spool.clone();
        let completion_tx = self.completion_tx.clone();
        let completion_path = path.clone();
        self.state.target_path = Some(path);
        self.state.status = DeviceStatus::Busy;
        self.state.last_error = None;
        handle.spawn_blocking(move || {
            let result = pdf::write(&completion_path, &spool);
            let _ = completion_tx.send(PrintCompletion {
                path: completion_path,
                result,
            });
        });
        Ok(())
    }

    pub fn poll(&mut self) -> bool {
        let mut changed = false;
        while let Ok(completion) = self.completion_rx.try_recv() {
            changed = true;
            self.state.target_path = Some(completion.path);
            match completion.result {
                Ok(()) => {
                    self.state.status = DeviceStatus::Ready;
                    self.state.last_error = None;
                }
                Err(error) => {
                    self.state.status = DeviceStatus::Error(error.clone());
                    self.state.last_error = Some(error);
                }
            }
        }
        changed
    }

    pub fn clear(&mut self) {
        self.state.spool.clear();
        self.state.bytes_buffered = 0;
    }

    pub fn input_byte(&self) -> u8 {
        self.state.status.code()
    }

    pub fn state(&self) -> PrinterState {
        self.state.clone()
    }
}
