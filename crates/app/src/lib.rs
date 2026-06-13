pub mod actor;
pub mod command;
pub mod emulator;
pub mod error;

pub use actor::{EmulatorHandle, MIN_STEP_INTERVAL, initial_snapshot, spawn_emulator};
pub use command::{AppCommand, AppEvent, AppSnapshot, RunMode};
pub use emulator::{DEFAULT_STEP_INTERVAL, Emulator};
pub use error::AppError;
pub use k580_devices::{
    ConnectionState, DeviceSnapshot, DeviceStatus, GRAPHICS_HEIGHT, GRAPHICS_WIDTH, MonitorPhase,
    MonitorState, NetworkMode, NetworkState, StorageState, TEXT_COLS, TEXT_ROWS, TextCell,
};
