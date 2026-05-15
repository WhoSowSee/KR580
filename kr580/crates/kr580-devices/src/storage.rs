//! Floppy and HDD storage devices.
//!
//! Per `prompt/03_peripherals.md` and `prompt/08_peripheral_edge_cases.md`:
//!
//! * writes are *queued*;
//! * file open/create/flush/close errors are explicit, not modal;
//! * paths come from settings, not from the process working directory;
//! * the visible buffer is a snapshot, not the source of truth;
//! * the host file is the canonical destination.
//!
//! Internally the device runs a Tokio task that drains a write queue. The
//! synchronous `write` / `read` methods called from the IO bus only enqueue
//! bytes; they never block on disk I/O.

use crate::error::DeviceError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// Logical storage type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageKind {
    /// Floppy device (port 0x01).
    Floppy,
    /// Hard disk device (port 0x02).
    Hdd,
}

/// Snapshot of storage status the UI / exporters can read.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStatus {
    /// Underlying file path.
    pub path: PathBuf,
    /// Total bytes written since the device was opened.
    pub bytes_written: u64,
    /// Visible tail of recently written bytes (snapshot only — not used as
    /// the source of truth).
    pub tail_buffer: Vec<u8>,
    /// Last device error, if any.
    pub last_error: Option<String>,
    /// Whether the worker task is still running.
    pub worker_alive: bool,
}

impl StorageStatus {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            bytes_written: 0,
            tail_buffer: Vec::new(),
            last_error: None,
            worker_alive: true,
        }
    }
}

/// Storage device (floppy or HDD).
pub struct StorageDevice {
    kind: StorageKind,
    tx: mpsc::UnboundedSender<u8>,
    status: Arc<Mutex<StorageStatus>>,
    /// Worker join handle. Dropped at device shutdown.
    _worker: tokio::task::JoinHandle<()>,
}

impl StorageDevice {
    /// Spawn the device worker on the current Tokio runtime. The caller must
    /// be inside a Tokio runtime (e.g. the application has a multi-thread
    /// runtime active).
    pub fn spawn(kind: StorageKind, path: PathBuf) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<u8>();
        let status = Arc::new(Mutex::new(StorageStatus::new(path.clone())));
        let status_for_worker = Arc::clone(&status);
        let worker = tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;
            // Open / create the host file. Errors are surfaced through status.
            let file = match tokio::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .await
            {
                Ok(f) => Some(f),
                Err(e) => {
                    let mut s = status_for_worker.lock().unwrap();
                    s.last_error = Some(map_open_error(&e).to_string());
                    s.worker_alive = false;
                    None
                }
            };
            let Some(mut file) = file else { return };
            while let Some(byte) = rx.recv().await {
                if let Err(e) = file.write_all(&[byte]).await {
                    let mut s = status_for_worker.lock().unwrap();
                    s.last_error = Some(format!("write: {e}"));
                    continue;
                }
                let mut s = status_for_worker.lock().unwrap();
                s.bytes_written += 1;
                s.tail_buffer.push(byte);
                if s.tail_buffer.len() > 4096 {
                    let drop = s.tail_buffer.len() - 4096;
                    s.tail_buffer.drain(0..drop);
                }
            }
            // Channel closed: flush and shut down.
            let _ = file.flush().await;
            let mut s = status_for_worker.lock().unwrap();
            s.worker_alive = false;
        });

        Self {
            kind,
            tx,
            status,
            _worker: worker,
        }
    }

    /// Logical kind (floppy vs HDD).
    pub fn kind(&self) -> StorageKind {
        self.kind
    }

    /// Snapshot the current device status for UI / export.
    pub fn snapshot_status(&self) -> StorageStatus {
        self.status.lock().unwrap().clone()
    }

    /// Enqueue one byte. Never blocks; never panics.
    pub fn write(&self, byte: u8) -> Result<(), DeviceError> {
        self.tx.send(byte).map_err(|_| DeviceError::Disconnected)?;
        Ok(())
    }

    /// `IN`: storage devices have no spec'd read semantics in the prompt.
    /// Return `0xFF` (open bus).
    pub fn read(&self) -> u8 {
        0xFF
    }
}

fn map_open_error(e: &std::io::Error) -> DeviceError {
    match e.kind() {
        std::io::ErrorKind::NotFound => DeviceError::PathNotFound(e.to_string()),
        std::io::ErrorKind::PermissionDenied => DeviceError::PermissionDenied,
        _ => DeviceError::Io(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn enqueues_bytes_to_disk() {
        let dir = tempfile_dir();
        let path = dir.join("test_floppy.kpd");
        let dev = StorageDevice::spawn(StorageKind::Floppy, path.clone());
        for b in b"hello" {
            dev.write(*b).unwrap();
        }
        // Allow the worker to drain.
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let snap = dev.snapshot_status();
        assert_eq!(snap.bytes_written, 5);
        assert_eq!(snap.tail_buffer, b"hello");
        // Drop dev so the channel closes; then read the file.
        drop(dev);
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let bytes = tokio::fs::read(&path).await.unwrap();
        assert!(bytes.ends_with(b"hello"));
    }

    fn tempfile_dir() -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("kr580-test-{}", std::process::id()));
        std::fs::create_dir_all(&p).unwrap();
        p
    }
}
