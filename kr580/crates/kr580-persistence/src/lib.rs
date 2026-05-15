//! Persistence layer: `.580` snapshots, `.krs` subprograms, JSON settings,
//! plain text exports.

#![deny(rust_2018_idioms)]
#![warn(missing_docs)]

pub mod error;
pub mod export;
pub mod settings;
pub mod snapshot;
pub mod subprogram;

pub use error::{ExportError, PersistenceError, SnapshotError};
pub use export::TxtExporter;
pub use settings::{
    ExportSettings, NetworkMode, NetworkSettings, Settings, SettingsStore, StorageSettings,
    UiSettings,
};
pub use snapshot::{Snapshot580Serializer, SNAPSHOT_VERSION};
pub use subprogram::{SubprogramFile, SubprogramSerializer};
