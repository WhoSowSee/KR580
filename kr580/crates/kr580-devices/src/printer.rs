//! Printer device.
//!
//! Per `prompt/03_peripherals.md` and `prompt/08_peripheral_edge_cases.md`:
//!
//! * the core writes bytes to the device;
//! * the device renders bytes to a *buffer*;
//! * printing is a separate action from buffer accumulation;
//! * failures appear as device state, not as crashes.

use crate::error::DeviceError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Printer status snapshot.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrinterStatus {
    /// Total bytes accumulated in the spool.
    pub bytes_buffered: u64,
    /// Spool contents (UTF-8 lossy view of the byte buffer).
    pub spool_text: String,
    /// Last error.
    pub last_error: Option<String>,
}

/// Synchronous printer device.
///
/// The printer does not need a worker task — it just appends to an in-memory
/// buffer. The expensive operation (`flush_to_file`) is explicit.
#[derive(Debug, Default)]
pub struct PrinterDevice {
    spool: Vec<u8>,
    status: PrinterStatus,
}

impl PrinterDevice {
    /// Build a new empty printer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Snapshot status.
    pub fn snapshot_status(&self) -> PrinterStatus {
        PrinterStatus {
            bytes_buffered: self.status.bytes_buffered,
            spool_text: String::from_utf8_lossy(&self.spool).to_string(),
            last_error: self.status.last_error.clone(),
        }
    }

    /// Accumulate a byte into the spool.
    pub fn write(&mut self, byte: u8) {
        self.spool.push(byte);
        self.status.bytes_buffered += 1;
    }

    /// `IN`: there is no read protocol for the printer.
    pub fn read(&self) -> u8 {
        0xFF
    }

    /// Render the spool as a UTF-8 lossy text snapshot.
    pub fn spool_text(&self) -> String {
        String::from_utf8_lossy(&self.spool).to_string()
    }

    /// Print: flush the spool to a file. Buffer is preserved on success so
    /// the user can see what was printed; clear it explicitly via
    /// [`Self::clear`].
    pub fn flush_to_file(&mut self, path: PathBuf) -> Result<(), DeviceError> {
        match std::fs::write(&path, &self.spool) {
            Ok(()) => {
                self.status.last_error = None;
                Ok(())
            }
            Err(e) => {
                let err = match e.kind() {
                    std::io::ErrorKind::NotFound => {
                        DeviceError::PathNotFound(path.display().to_string())
                    }
                    std::io::ErrorKind::PermissionDenied => DeviceError::PermissionDenied,
                    _ => DeviceError::Io(e.to_string()),
                };
                self.status.last_error = Some(err.to_string());
                Err(err)
            }
        }
    }

    /// Clear the spool buffer.
    pub fn clear(&mut self) {
        self.spool.clear();
        self.status.bytes_buffered = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffers_bytes_then_flushes() {
        let mut p = PrinterDevice::new();
        for b in b"hello\n" {
            p.write(*b);
        }
        assert_eq!(p.snapshot_status().bytes_buffered, 6);
        let mut path = std::env::temp_dir();
        path.push(format!("kr580-printer-{}.txt", std::process::id()));
        p.flush_to_file(path.clone()).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(&bytes, b"hello\n");
        let _ = std::fs::remove_file(path);
    }
}
