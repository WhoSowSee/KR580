mod capabilities;
mod dialog;
mod print_ticket;

use super::super::{
    PrinterConfiguration, PrinterInfo, PrinterOrientation, PrinterPropertyChange,
    PrinterPropertySheet, PrinterSettings,
};
use super::{last_os_error, wide_null};
use std::ptr::{null, null_mut};
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Graphics::Gdi::{
    DEVMODEW, DM_DEFAULTSOURCE, DM_IN_BUFFER, DM_ORIENTATION, DM_OUT_BUFFER, DM_PAPERLENGTH,
    DM_PAPERSIZE, DM_PAPERWIDTH, DMORIENT_LANDSCAPE, DMORIENT_PORTRAIT,
};
use windows_sys::Win32::Graphics::Printing::{ClosePrinter, DocumentPropertiesW, OpenPrinterW};
use windows_sys::Win32::UI::WindowsAndMessaging::IDOK;

pub(super) fn configure() -> Result<Option<PrinterSettings>, String> {
    let Some((printer_name, devmode)) = dialog::configure()? else {
        return Ok(None);
    };
    let printer = super::list::printers()?
        .into_iter()
        .find(|printer| printer.name == printer_name);
    match printer {
        Some(printer) => Ok(Some(
            configuration_from_devmode(&printer, devmode)?.settings,
        )),
        None => Ok(Some(settings_from_devmode(printer_name, devmode, &[], &[]))),
    }
}

pub(super) fn configuration(printer: &PrinterInfo) -> Result<PrinterConfiguration, String> {
    let devmode = default_devmode(&printer.name)?;
    configuration_from_devmode(printer, devmode)
}

pub(super) fn load_properties(
    printer: &PrinterInfo,
    settings: &PrinterSettings,
) -> Result<PrinterPropertySheet, String> {
    print_ticket::load(printer, settings)
}

pub(super) fn apply_property(
    printer: &PrinterInfo,
    settings: &PrinterSettings,
    change: &PrinterPropertyChange,
) -> Result<PrinterPropertySheet, String> {
    print_ticket::apply(printer, settings, change)
}

pub(super) fn print_devmode(settings: &PrinterSettings) -> Result<AlignedDevMode, String> {
    let handle = PrinterHandle::open(&settings.printer_name)?;
    let mut devmode = default_devmode_with_handle(&settings.printer_name, handle.0)?;
    if settings.devmode.is_empty() && settings.paper_id.is_none() && settings.source_id.is_none() {
        return Ok(devmode);
    }
    if !settings.devmode.is_empty()
        && let Ok(saved) = normalize_devmode(&settings.printer_name, handle.0, &settings.devmode)
    {
        devmode = saved;
    }
    apply_standard_settings(&mut devmode, settings);
    normalize_devmode(
        &settings.printer_name,
        handle.0,
        &devmode.clone().into_bytes(),
    )
}

pub(super) fn configuration_from_devmode(
    printer: &PrinterInfo,
    devmode: Vec<u8>,
) -> Result<PrinterConfiguration, String> {
    let aligned = AlignedDevMode::from_bytes(&devmode);
    let (papers, sources) = capabilities::load(printer, aligned.as_ptr())?;
    Ok(PrinterConfiguration {
        settings: settings_from_devmode(printer.name.clone(), devmode, &papers, &sources),
        papers,
        sources,
    })
}

fn settings_from_devmode(
    printer_name: String,
    devmode: Vec<u8>,
    papers: &[super::super::PrinterPaper],
    sources: &[super::super::PrinterSource],
) -> PrinterSettings {
    let Some(mode) = read_devmode(&devmode) else {
        return PrinterSettings::named(printer_name);
    };
    let fields = mode.dmFields;
    let printer = unsafe { mode.Anonymous1.Anonymous1 };
    let paper_id = ((fields & DM_PAPERSIZE) != 0).then_some(printer.dmPaperSize);
    let source_id = ((fields & DM_DEFAULTSOURCE) != 0).then_some(printer.dmDefaultSource);
    let orientation =
        if (fields & DM_ORIENTATION) != 0 && printer.dmOrientation as u32 == DMORIENT_LANDSCAPE {
            PrinterOrientation::Landscape
        } else {
            PrinterOrientation::Portrait
        };
    PrinterSettings {
        printer_name,
        paper_id,
        paper_name: paper_id.and_then(|id| {
            papers
                .iter()
                .find(|paper| paper.id == id)
                .map(|paper| paper.name.clone())
        }),
        source_id,
        source_name: source_id.and_then(|id| {
            sources
                .iter()
                .find(|source| source.id == id)
                .map(|source| source.name.clone())
        }),
        orientation,
        devmode,
    }
}

fn apply_standard_settings(devmode: &mut AlignedDevMode, settings: &PrinterSettings) {
    let mode = unsafe { &mut *devmode.as_mut_ptr() };
    let mut printer = unsafe { mode.Anonymous1.Anonymous1 };
    mode.dmFields |= DM_ORIENTATION;
    printer.dmOrientation = match settings.orientation {
        PrinterOrientation::Portrait => DMORIENT_PORTRAIT as i16,
        PrinterOrientation::Landscape => DMORIENT_LANDSCAPE as i16,
    };
    if let Some(paper_id) = settings.paper_id {
        mode.dmFields &= !(DM_PAPERLENGTH | DM_PAPERWIDTH);
        mode.dmFields |= DM_PAPERSIZE;
        printer.dmPaperSize = paper_id;
    }
    if let Some(source_id) = settings.source_id {
        mode.dmFields |= DM_DEFAULTSOURCE;
        printer.dmDefaultSource = source_id;
    }
    mode.Anonymous1.Anonymous1 = printer;
}

fn default_devmode(printer_name: &str) -> Result<Vec<u8>, String> {
    let handle = PrinterHandle::open(printer_name)?;
    Ok(default_devmode_with_handle(printer_name, handle.0)?.into_bytes())
}

fn default_devmode_with_handle(
    printer_name: &str,
    handle: HANDLE,
) -> Result<AlignedDevMode, String> {
    let name = wide_null(printer_name);
    let size =
        unsafe { DocumentPropertiesW(null_mut(), handle, name.as_ptr(), null_mut(), null(), 0) };
    if size <= 0 {
        return Err(last_os_error("DocumentPropertiesW"));
    }
    let mut devmode = AlignedDevMode::zeroed(size as usize);
    let result = unsafe {
        DocumentPropertiesW(
            null_mut(),
            handle,
            name.as_ptr(),
            devmode.as_mut_ptr(),
            null(),
            DM_OUT_BUFFER,
        )
    };
    if result != IDOK {
        return Err(last_os_error("DocumentPropertiesW"));
    }
    Ok(devmode)
}

fn normalize_devmode(
    printer_name: &str,
    handle: HANDLE,
    input: &[u8],
) -> Result<AlignedDevMode, String> {
    if read_devmode(input).is_none() {
        return Err("saved printer settings are invalid".to_owned());
    }
    let input = AlignedDevMode::from_bytes(input);
    let mut output = default_devmode_with_handle(printer_name, handle)?;
    let name = wide_null(printer_name);
    let result = unsafe {
        DocumentPropertiesW(
            null_mut(),
            handle,
            name.as_ptr(),
            output.as_mut_ptr(),
            input.as_ptr(),
            DM_IN_BUFFER | DM_OUT_BUFFER,
        )
    };
    if result != IDOK {
        return Err(last_os_error("DocumentPropertiesW"));
    }
    Ok(output)
}

fn read_devmode(bytes: &[u8]) -> Option<DEVMODEW> {
    if bytes.len() < std::mem::size_of::<DEVMODEW>() {
        return None;
    }
    let mode = unsafe { bytes.as_ptr().cast::<DEVMODEW>().read_unaligned() };
    let required = mode.dmSize as usize + mode.dmDriverExtra as usize;
    (mode.dmSize as usize >= std::mem::size_of::<DEVMODEW>() && required <= bytes.len())
        .then_some(mode)
}

struct PrinterHandle(HANDLE);

impl PrinterHandle {
    fn open(name: &str) -> Result<Self, String> {
        let name = wide_null(name);
        let mut handle = null_mut();
        if unsafe { OpenPrinterW(name.as_ptr(), &mut handle, null()) } == 0 {
            return Err(last_os_error("OpenPrinterW"));
        }
        Ok(Self(handle))
    }
}

impl Drop for PrinterHandle {
    fn drop(&mut self) {
        unsafe {
            ClosePrinter(self.0);
        }
    }
}

#[derive(Clone)]
pub(super) struct AlignedDevMode {
    words: Vec<usize>,
    len: usize,
}

impl AlignedDevMode {
    fn zeroed(len: usize) -> Self {
        Self {
            words: vec![0; len.div_ceil(std::mem::size_of::<usize>())],
            len,
        }
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let mut buffer = Self::zeroed(bytes.len());
        unsafe {
            std::ptr::copy_nonoverlapping(
                bytes.as_ptr(),
                buffer.words.as_mut_ptr().cast::<u8>(),
                bytes.len(),
            );
        }
        buffer
    }

    pub(super) fn as_ptr(&self) -> *const DEVMODEW {
        self.words.as_ptr().cast()
    }

    fn as_mut_ptr(&mut self) -> *mut DEVMODEW {
        self.words.as_mut_ptr().cast()
    }

    fn into_bytes(self) -> Vec<u8> {
        unsafe { std::slice::from_raw_parts(self.words.as_ptr().cast::<u8>(), self.len).to_vec() }
    }
}
