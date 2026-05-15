//! iced-based view layer for the KR580 emulator.
//!
//! Per `prompt/05_ui_and_workflows.md` the UI is a renderer + command
//! dispatcher. It owns *no* emulator state, only the most recent state
//! snapshot it received from the runtime, plus dispatch channels.

#![deny(rust_2018_idioms)]
#![warn(missing_docs)]

pub mod runtime;
pub mod view;

pub use runtime::{run, RuntimeHandles};
pub use view::{EmulatorApp, Message};
