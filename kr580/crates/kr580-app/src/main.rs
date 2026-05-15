//! Top-level KR580 emulator binary.
//!
//! The binary owns the orchestration:
//!
//! * starts a Tokio runtime for device workers;
//! * builds the routing [`DeviceBus`];
//! * spawns the deterministic core actor on a dedicated thread (in
//!   `kr580_ui::runtime::run`);
//! * hands the resulting channel handles to the iced application.

use anyhow::Result;
use iced::{Application, Settings as IcedSettings};
use kr580_devices::DeviceBus;
use kr580_ui::{run as run_runtime, EmulatorApp};
use std::sync::{Arc, Mutex};
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {
    init_tracing();

    // Tokio runtime drives device workers (storage / network / etc.).
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let _guard = runtime.enter();

    // Routing IoBus shared between the core actor and device workers.
    let bus = Arc::new(Mutex::new(DeviceBus::new()));
    let handles = run_runtime(Arc::clone(&bus));

    // Hand off to iced. The iced runtime will block this thread.
    EmulatorApp::run(IcedSettings::with_flags(handles))
        .map_err(|e| anyhow::anyhow!("iced error: {e}"))?;

    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .compact()
        .init();
}
