//! IoBus trait used by `IN` and `OUT` opcodes.
//!
//! Per `prompt/03_peripherals.md` the core *only* talks to the bus, never to
//! UI controls. Devices observe `IN` and `OUT` at instruction boundaries.

/// Boundary-level port bus used by `IN` / `OUT`.
///
/// Implementations should be cheap and non-blocking. Heavy work belongs in
/// device worker tasks; the bus itself just enqueues / drains a frontier.
pub trait IoBus: Send {
    /// Read a byte from `port`. Implementations that have no pending byte
    /// must return `0xFF` (open bus) per typical 8080 systems.
    fn read(&mut self, port: u8) -> u8;

    /// Write `value` to `port`.
    fn write(&mut self, port: u8, value: u8);
}

/// IoBus that swallows writes and returns 0xFF on every read.
/// Useful for unit tests that exercise CPU behavior in isolation.
#[derive(Debug, Default, Clone, Copy)]
pub struct NullIoBus;

impl IoBus for NullIoBus {
    fn read(&mut self, _port: u8) -> u8 {
        0xFF
    }
    fn write(&mut self, _port: u8, _value: u8) {}
}

/// In-memory recording bus, useful for testing.
#[derive(Debug)]
pub struct RecordingIoBus {
    /// Bytes the device side will hand back on the next `IN`, per port.
    pub in_queue: Vec<Vec<u8>>,
    /// All `OUT` operations seen, in order: `(port, value)`.
    pub out_log: Vec<(u8, u8)>,
}

impl Default for RecordingIoBus {
    fn default() -> Self {
        Self {
            in_queue: (0..256).map(|_| Vec::new()).collect(),
            out_log: Vec::new(),
        }
    }
}

impl IoBus for RecordingIoBus {
    fn read(&mut self, port: u8) -> u8 {
        let q = &mut self.in_queue[port as usize];
        if q.is_empty() {
            0xFF
        } else {
            q.remove(0)
        }
    }
    fn write(&mut self, port: u8, value: u8) {
        self.out_log.push((port, value));
    }
}
