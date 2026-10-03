#[cfg(any(windows, test))]
mod buffer;
#[cfg(windows)]
mod list;
#[cfg(windows)]
mod settings;

#[cfg(windows)]
fn wide_null(value: &str) -> Vec<u16> {
    value.encode_utf16().chain([0]).collect()
}

#[cfg(windows)]
fn last_os_error(api: &str) -> String {
    // SAFETY: GetLastError only reads thread-local Win32 error state.
    let code = unsafe { windows_sys::Win32::Foundation::GetLastError() };
    let error = std::io::Error::from_raw_os_error(code as i32);
    format!("{api} failed ({code}): {error}")
}

#[derive(Debug)]
pub(super) enum PrintFailure {
    #[cfg(windows)]
    Cancelled,
    Failed(String),
}

#[cfg(windows)]
mod printing;
#[cfg(windows)]
use printing as imp;

#[cfg(not(windows))]
mod imp {
    use super::super::{
        PrinterConfiguration, PrinterInfo, PrinterPropertyChange, PrinterPropertySheet,
        PrinterSettings,
    };

    pub(super) fn configure() -> Result<Option<PrinterSettings>, String> {
        Err("system printer setup is only available on Windows".to_owned())
    }

    pub(super) fn list() -> Result<Vec<PrinterInfo>, String> {
        Err("system printer list is only available on Windows".to_owned())
    }

    pub(super) fn configuration(_printer: &PrinterInfo) -> Result<PrinterConfiguration, String> {
        Err("system printer capabilities are only available on Windows".to_owned())
    }

    pub(super) fn load_properties(
        _printer: &PrinterInfo,
        _settings: &PrinterSettings,
    ) -> Result<PrinterPropertySheet, String> {
        Err("printer properties are only available on Windows".to_owned())
    }

    pub(super) fn apply_property(
        _printer: &PrinterInfo,
        _settings: &PrinterSettings,
        _change: &PrinterPropertyChange,
    ) -> Result<PrinterPropertySheet, String> {
        Err("printer properties are only available on Windows".to_owned())
    }

    pub(super) fn print(
        _settings: Option<&PrinterSettings>,
        _spool: &[u8],
    ) -> Result<(), super::PrintFailure> {
        Err(super::PrintFailure::Failed(
            "system printing is only available on Windows".to_owned(),
        ))
    }
}

pub(super) fn configure() -> Result<Option<super::PrinterSettings>, String> {
    imp::configure()
}

pub(super) fn list() -> Result<Vec<super::PrinterInfo>, String> {
    imp::list()
}

pub(super) fn configuration(
    printer: &super::PrinterInfo,
) -> Result<super::PrinterConfiguration, String> {
    imp::configuration(printer)
}

pub(super) fn load_properties(
    printer: &super::PrinterInfo,
    settings: &super::PrinterSettings,
) -> Result<super::PrinterPropertySheet, String> {
    imp::load_properties(printer, settings)
}

pub(super) fn apply_property(
    printer: &super::PrinterInfo,
    settings: &super::PrinterSettings,
    change: &super::PrinterPropertyChange,
) -> Result<super::PrinterPropertySheet, String> {
    imp::apply_property(printer, settings, change)
}

pub(super) fn print(
    settings: Option<&super::PrinterSettings>,
    spool: &[u8],
) -> Result<(), PrintFailure> {
    imp::print(settings, spool)
}
