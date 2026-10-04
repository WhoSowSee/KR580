use crate::devices::{DeviceError, DeviceStatus};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;

mod worker;

const QUEUE_CAP: usize = 65_536;
const CONTROL_RESERVE: usize = 16;
const HISTORY_CAP: usize = 65_536;
const DEBUG_BUFFER_CAP: usize = 1_048_576;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StorageKind {
    Floppy,
    Hdd,
}

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
    tail: VecDeque<u8>,
    visible: VecDeque<u8>,
    tx: Option<mpsc::Sender<StorageCommand>>,
    error_rx: Option<mpsc::Receiver<DeviceError>>,
}

#[derive(Debug)]
enum StorageCommand {
    Write(u8),
    Flush,
    Attach(std::fs::File),
    Detach,
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
            tail: VecDeque::new(),
            visible: VecDeque::new(),
            tx: None,
            error_rx: None,
        }
    }

    pub fn attach_file(
        &mut self,
        path: impl AsRef<Path>,
        handle: &tokio::runtime::Handle,
    ) -> Result<(), DeviceError> {
        let path = path.as_ref().to_path_buf();
        match OpenOptions::new().create(true).append(true).open(&path) {
            Ok(file) => self.attach_open_file(path, file, handle),
            Err(error) => self.record_attachment_failure(path, error.into()),
        }
    }

    /// Installs a prepared file without filesystem work on the calling thread.
    pub fn attach_open_file(
        &mut self,
        path: PathBuf,
        file: std::fs::File,
        handle: &tokio::runtime::Handle,
    ) -> Result<(), DeviceError> {
        if let Some(tx) = &self.tx {
            let result = super::queue::enqueue(tx, StorageCommand::Attach(file));
            self.finish_enqueue(result)?;
        } else {
            let (tx, rx) = mpsc::channel(QUEUE_CAP + CONTROL_RESERVE);
            let (error_tx, error_rx) = mpsc::channel(1);
            handle.spawn(worker::write_file(file, rx, error_tx));
            self.tx = Some(tx);
            self.error_rx = Some(error_rx);
        }
        self.state.path = Some(path);
        self.state.status = DeviceStatus::Ready;
        self.state.last_error = None;
        self.state.worker_alive = true;
        self.state.debug_buffer = false;
        Ok(())
    }

    pub fn detach_file(&mut self) -> Result<(), DeviceError> {
        if let Some(tx) = &self.tx {
            let result = super::queue::enqueue(tx, StorageCommand::Detach);
            self.finish_enqueue(result)?;
        }
        self.state.path = None;
        self.state.status = if self.state.debug_buffer {
            DeviceStatus::Ready
        } else {
            DeviceStatus::NotReady
        };
        self.state.last_error = None;
        self.state.worker_alive = false;
        Ok(())
    }

    pub(crate) fn record_attachment_failure(
        &mut self,
        path: PathBuf,
        error: DeviceError,
    ) -> Result<(), DeviceError> {
        self.detach_file()?;
        self.state.path = Some(path);
        self.state.status = DeviceStatus::Error(error.to_string());
        self.state.last_error = Some(error.to_string());
        self.tx = None;
        self.error_rx = None;
        Err(error)
    }

    pub fn write_byte(&mut self, value: u8) -> Result<(), DeviceError> {
        if let Some(tx) = self.tx.as_ref().filter(|_| self.state.path.is_some()) {
            if tx.capacity() <= CONTROL_RESERVE {
                return self.finish_enqueue(Err(DeviceError::Busy));
            }
            let result = super::queue::enqueue(tx, StorageCommand::Write(value));
            self.finish_enqueue(result)?;
            self.accept_visible_byte(value);
            self.state.bytes_queued += 1;
            self.state.last_error = None;
            return Ok(());
        }

        if self.state.debug_buffer {
            if self.visible.len() == DEBUG_BUFFER_CAP {
                return self.finish_enqueue(Err(DeviceError::Busy));
            }
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
        self.state.status = match (enabled, self.state.path.is_some() && self.tx.is_some()) {
            (true, _) | (false, true) => DeviceStatus::Ready,
            (false, false) => DeviceStatus::NotReady,
        };
        self.state.last_error = None;
    }

    pub(crate) fn debug_buffer_enabled(&self) -> bool {
        self.state.debug_buffer
    }

    pub fn clear_visible_buffer(&mut self) {
        self.visible.clear();
        self.tail.clear();
    }

    pub fn poll(&mut self) -> bool {
        let Some(error_rx) = self.error_rx.as_mut() else {
            return false;
        };
        let mut changed = false;
        while let Ok(error) = error_rx.try_recv() {
            self.state.status = DeviceStatus::Error(error.to_string());
            self.state.last_error = Some(error.to_string());
            self.state.worker_alive = false;
            self.tx = None;
            changed = true;
        }
        changed
    }

    pub fn flush(&mut self) -> Result<(), DeviceError> {
        let Some(tx) = self.tx.as_ref().filter(|_| self.state.path.is_some()) else {
            self.state.status = DeviceStatus::NotReady;
            self.state.last_error = Some(DeviceError::NotReady.to_string());
            return Err(DeviceError::NotReady);
        };
        if tx.capacity() <= CONTROL_RESERVE {
            return self.finish_enqueue(Err(DeviceError::Busy));
        }
        let result = super::queue::enqueue(tx, StorageCommand::Flush);
        self.finish_enqueue(result)
    }

    pub fn close(&mut self) -> Result<(), DeviceError> {
        self.tx = None;
        self.state.status = DeviceStatus::NotReady;
        self.state.worker_alive = false;
        Ok(())
    }

    pub fn input_byte(&self) -> u8 {
        self.state.status.code()
    }

    pub fn state(&self) -> StorageState {
        let mut state = self.state.clone();
        state.visible_buffer = self.visible.iter().copied().collect();
        state.tail_buffer = self.tail.iter().copied().collect();
        state
    }

    fn finish_enqueue(&mut self, result: Result<(), DeviceError>) -> Result<(), DeviceError> {
        match &result {
            Ok(()) => self.state.status = DeviceStatus::Ready,
            Err(error) => {
                self.state.status = if *error == DeviceError::Busy {
                    DeviceStatus::Busy
                } else {
                    self.state.worker_alive = false;
                    DeviceStatus::Disconnected
                };
                self.state.last_error = Some(error.to_string());
            }
        }
        result
    }

    fn accept_visible_byte(&mut self, value: u8) {
        self.visible.push_back(value);
        if self.state.path.is_some() && self.visible.len() > HISTORY_CAP {
            self.visible.pop_front();
        }
        self.tail.push_back(value);
        if self.tail.len() > 4096 {
            self.tail.pop_front();
        }
    }
}
