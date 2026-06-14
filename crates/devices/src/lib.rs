pub mod error;
pub mod iobus;
pub mod monitor;
pub mod network;
mod oem;
pub mod printer;
pub mod status;
pub mod storage;

pub use error::DeviceError;
pub use iobus::{DeviceSnapshot, IoBus};
pub use monitor::{
    GRAPHICS_HEIGHT, GRAPHICS_WIDTH, MonitorDevice, MonitorPhase, MonitorState, TEXT_CELL_COUNT,
    TEXT_COLS, TEXT_ROWS, TextCell,
};
pub use network::{ConnectionState, NetworkDevice, NetworkMode, NetworkState};
pub use oem::decode_oem_text;
pub use printer::{PrinterDevice, PrinterState};
pub use status::DeviceStatus;
pub use storage::{StorageDevice, StorageState};
