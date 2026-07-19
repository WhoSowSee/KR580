use crate::devices::{DeviceError, DeviceStatus};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;

mod native;
mod properties;
mod text;

pub use properties::{
    PrinterFeature, PrinterFeatureGroup, PrinterFeatureOption, PrinterParameter,
    PrinterPropertyChange, PrinterPropertySheet,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrinterState {
    pub spool: Vec<u8>,
    pub target_path: Option<PathBuf>,
    pub status: DeviceStatus,
    pub bytes_buffered: u64,
    pub last_error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrinterInfo {
    pub name: String,
    pub driver: String,
    pub port: String,
    pub location: String,
    pub comment: String,
    pub status: String,
    pub is_default: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PrinterOrientation {
    #[default]
    Portrait,
    Landscape,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrinterPaper {
    pub id: i16,
    pub name: String,
}

impl std::fmt::Display for PrinterPaper {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrinterSource {
    pub id: i16,
    pub name: String,
}

impl std::fmt::Display for PrinterSource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrinterSettings {
    pub printer_name: String,
    pub paper_id: Option<i16>,
    pub paper_name: Option<String>,
    pub source_id: Option<i16>,
    pub source_name: Option<String>,
    pub orientation: PrinterOrientation,
    #[serde(default)]
    pub devmode: Vec<u8>,
}

impl PrinterSettings {
    pub fn named(printer_name: String) -> Self {
        Self {
            printer_name,
            paper_id: None,
            paper_name: None,
            source_id: None,
            source_name: None,
            orientation: PrinterOrientation::Portrait,
            devmode: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrinterConfiguration {
    pub settings: PrinterSettings,
    pub papers: Vec<PrinterPaper>,
    pub sources: Vec<PrinterSource>,
}

impl PrinterConfiguration {
    pub fn selected_paper(&self) -> Option<&PrinterPaper> {
        let id = self.settings.paper_id?;
        self.papers.iter().find(|paper| paper.id == id)
    }

    pub fn selected_source(&self) -> Option<&PrinterSource> {
        let id = self.settings.source_id?;
        self.sources.iter().find(|source| source.id == id)
    }

    pub fn select_paper(&mut self, id: i16) {
        let Some(paper) = self.papers.iter().find(|paper| paper.id == id) else {
            return;
        };
        self.settings.paper_id = Some(id);
        self.settings.paper_name = Some(paper.name.clone());
    }

    pub fn select_source(&mut self, id: i16) {
        let Some(source) = self.sources.iter().find(|source| source.id == id) else {
            return;
        };
        self.settings.source_id = Some(id);
        self.settings.source_name = Some(source.name.clone());
    }
}

#[derive(Debug)]
pub struct PrinterDevice {
    state: PrinterState,
    tx: Option<mpsc::UnboundedSender<Vec<u8>>>,
    completion_tx: mpsc::UnboundedSender<PrintCompletion>,
    completion_rx: mpsc::UnboundedReceiver<PrintCompletion>,
}

#[derive(Debug)]
struct PrintCompletion {
    result: Result<(), native::PrintFailure>,
}

impl Default for PrinterDevice {
    fn default() -> Self {
        let (completion_tx, completion_rx) = mpsc::unbounded_channel();
        Self {
            state: PrinterState {
                spool: Vec::new(),
                target_path: None,
                status: DeviceStatus::Ready,
                bytes_buffered: 0,
                last_error: None,
            },
            tx: None,
            completion_tx,
            completion_rx,
        }
    }
}

impl PrinterDevice {
    pub fn output_byte(&mut self, value: u8) {
        self.state.spool.push(value);
        self.state.bytes_buffered += 1;
    }

    pub fn attach_export_path(&mut self, path: impl AsRef<Path>, handle: &tokio::runtime::Handle) {
        let path = path.as_ref().to_path_buf();
        let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();
        let worker_path = path.clone();
        handle.spawn(async move {
            while let Some(bytes) = rx.recv().await {
                match tokio::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&worker_path)
                    .await
                {
                    Ok(mut file) => {
                        if let Err(err) = file.write_all(&bytes).await {
                            tracing::error!(error = %err, "printer export write failed");
                            continue;
                        }
                        if let Err(err) = file.flush().await {
                            tracing::error!(error = %err, "printer export flush failed");
                        }
                    }
                    Err(err) => tracing::error!(error = %err, "printer export open failed"),
                }
            }
        });
        self.state.target_path = Some(path);
        self.tx = Some(tx);
    }

    pub fn print_spool(&mut self) -> Result<(), DeviceError> {
        let Some(tx) = self.tx.clone() else {
            self.state.status = DeviceStatus::NotReady;
            self.state.last_error = Some(DeviceError::NotReady.to_string());
            return Err(DeviceError::NotReady);
        };
        tx.send(self.state.spool.clone()).map_err(|_| {
            self.state.status = DeviceStatus::Disconnected;
            self.state.last_error = Some(DeviceError::Disconnected.to_string());
            DeviceError::Disconnected
        })?;
        self.state.last_error = None;
        Ok(())
    }

    pub fn print_native(
        &mut self,
        settings: Option<PrinterSettings>,
        handle: &tokio::runtime::Handle,
    ) -> Result<(), DeviceError> {
        if self.state.status == DeviceStatus::Busy {
            return Err(DeviceError::Busy);
        }
        let spool = self.state.spool.clone();
        let completion_tx = self.completion_tx.clone();
        self.state.status = DeviceStatus::Busy;
        self.state.last_error = None;
        handle.spawn_blocking(move || {
            let result = native::print(settings.as_ref(), &spool);
            let _ = completion_tx.send(PrintCompletion { result });
        });
        Ok(())
    }

    pub fn poll(&mut self) -> bool {
        let mut changed = false;
        while let Ok(completion) = self.completion_rx.try_recv() {
            changed = true;
            match completion.result {
                Ok(()) => {
                    self.state.status = DeviceStatus::Ready;
                    self.state.last_error = None;
                }
                Err(native::PrintFailure::Cancelled) => {
                    self.state.status = DeviceStatus::Ready;
                    self.state.last_error = None;
                }
                Err(native::PrintFailure::Failed(error)) => {
                    self.state.status = DeviceStatus::Error(error.clone());
                    self.state.last_error = Some(error);
                }
            }
        }
        changed
    }

    pub fn clear(&mut self) {
        self.state.spool.clear();
        self.state.bytes_buffered = 0;
    }

    pub fn input_byte(&self) -> u8 {
        self.state.status.code()
    }

    pub fn state(&self) -> PrinterState {
        self.state.clone()
    }
}

pub fn configure_native_printer() -> Result<Option<PrinterSettings>, String> {
    native::configure()
}

pub fn list_native_printers() -> Result<Vec<PrinterInfo>, String> {
    native::list()
}

pub fn load_native_printer_configuration(
    printer: &PrinterInfo,
) -> Result<PrinterConfiguration, String> {
    native::configuration(printer)
}

pub fn load_native_printer_properties(
    printer: &PrinterInfo,
    settings: &PrinterSettings,
) -> Result<PrinterPropertySheet, String> {
    native::load_properties(printer, settings)
}

pub fn apply_native_printer_property(
    printer: &PrinterInfo,
    settings: &PrinterSettings,
    change: &PrinterPropertyChange,
) -> Result<PrinterPropertySheet, String> {
    native::apply_property(printer, settings, change)
}

#[cfg(test)]
mod tests {
    use super::{
        PrintCompletion, PrinterConfiguration, PrinterDevice, PrinterPaper, PrinterSettings,
        PrinterSource, native,
    };
    use crate::devices::DeviceStatus;

    #[test]
    fn cancelled_native_print_returns_to_ready_without_an_error() {
        let mut printer = PrinterDevice::default();
        printer.state.status = DeviceStatus::Busy;
        printer
            .completion_tx
            .send(PrintCompletion {
                result: Err(native::PrintFailure::Cancelled),
            })
            .unwrap();

        assert!(printer.poll());
        assert_eq!(printer.state.status, DeviceStatus::Ready);
        assert_eq!(printer.state.last_error, None);
    }

    #[test]
    fn configuration_keeps_selected_ids_and_names_in_sync() {
        let mut configuration = PrinterConfiguration {
            settings: PrinterSettings::named("Printer".to_owned()),
            papers: vec![PrinterPaper {
                id: 9,
                name: "A4".to_owned(),
            }],
            sources: vec![PrinterSource {
                id: 7,
                name: "Auto".to_owned(),
            }],
        };

        configuration.select_paper(9);
        configuration.select_source(7);

        assert_eq!(configuration.selected_paper().unwrap().name, "A4");
        assert_eq!(configuration.selected_source().unwrap().name, "Auto");
        assert_eq!(configuration.settings.paper_name.as_deref(), Some("A4"));
        assert_eq!(configuration.settings.source_name.as_deref(), Some("Auto"));
    }
}
