//! Asynchronous peripheral devices and the routing IoBus.
//!
//! Devices are addressed as documented in `prompt/03_peripherals.md`:
//!
//! | Port | Device          |
//! |-----:|-----------------|
//! | 00h  | Monitor         |
//! | 01h  | Floppy storage  |
//! | 02h  | HDD storage     |
//! | 03h  | Network adapter |
//! | 04h  | Printer         |
//!
//! Each device has its own non-blocking worker. The `IoBus` impl here is the
//! synchronous boundary the CPU calls into; it forwards work onto worker
//! channels so the core thread never blocks on device I/O.

#![deny(rust_2018_idioms)]
#![warn(missing_docs)]

pub mod bus;
pub mod error;
pub mod monitor;
pub mod network;
pub mod printer;
pub mod storage;

pub use bus::{DeviceBus, DeviceStatus};
pub use error::DeviceError;
pub use monitor::{MonitorDevice, MonitorMode, MonitorState};
pub use network::{NetworkDevice, NetworkMode, NetworkStatus};
pub use printer::{PrinterDevice, PrinterStatus};
pub use storage::{StorageDevice, StorageKind, StorageStatus};
