use super::super::{
    PrinterConfiguration, PrinterInfo, PrinterPropertyChange, PrinterPropertySheet, PrinterSettings,
};
use super::{PrintFailure, list as printer_list, settings, wide_null};
use crate::devices::printer::text;
use std::ptr::{null, null_mut};
use windows_sys::Win32::Foundation::{ERROR_CANCELLED, GetLastError};
use windows_sys::Win32::Graphics::Gdi::{
    CLIP_DEFAULT_PRECIS, CreateDCW, CreateFontW, DEFAULT_CHARSET, DEFAULT_QUALITY, DeleteDC,
    DeleteObject, FF_MODERN, FIXED_PITCH, FW_NORMAL, GetDeviceCaps, GetTextMetricsW, HDC, HGDIOBJ,
    HORZRES, LOGPIXELSX, LOGPIXELSY, OUT_DEFAULT_PRECIS, SelectObject, TEXTMETRICW, TextOutW,
    VERTRES,
};
use windows_sys::Win32::Storage::Xps::{AbortDoc, DOCINFOW, EndDoc, EndPage, StartDocW, StartPage};

const MARGIN_MM: f32 = 18.0;

pub(super) fn configure() -> Result<Option<PrinterSettings>, String> {
    settings::configure()
}

pub(super) fn list() -> Result<Vec<PrinterInfo>, String> {
    printer_list::printers()
}

pub(super) fn configuration(printer: &PrinterInfo) -> Result<PrinterConfiguration, String> {
    settings::configuration(printer)
}

pub(super) fn load_properties(
    printer: &PrinterInfo,
    print_settings: &PrinterSettings,
) -> Result<PrinterPropertySheet, String> {
    settings::load_properties(printer, print_settings)
}

pub(super) fn apply_property(
    printer: &PrinterInfo,
    print_settings: &PrinterSettings,
    change: &PrinterPropertyChange,
) -> Result<PrinterPropertySheet, String> {
    settings::apply_property(printer, print_settings, change)
}

pub(super) fn print(
    print_settings: Option<&PrinterSettings>,
    spool: &[u8],
) -> Result<(), PrintFailure> {
    let printer_name = match print_settings {
        Some(settings) if !settings.printer_name.trim().is_empty() => {
            settings.printer_name.trim().to_owned()
        }
        _ => printer_list::default_printer_name().map_err(PrintFailure::Failed)?,
    };
    let devmode = print_settings
        .map(settings::print_devmode)
        .transpose()
        .map_err(PrintFailure::Failed)?;
    let driver = wide_null("WINSPOOL");
    let device = wide_null(&printer_name);
    let devmode_ptr = devmode.as_ref().map_or(null(), |buffer| buffer.as_ptr());
    // SAFETY: Terminated driver/device names and optional validated aligned DEVMODE outlive the CreateDCW call.
    let hdc = unsafe { CreateDCW(driver.as_ptr(), device.as_ptr(), null(), devmode_ptr) };
    if hdc.is_null() {
        return Err(last_print_error("CreateDCW"));
    }

    let hdc = PrinterDc(hdc);
    print_to_hdc(hdc.0, spool)
}

fn print_to_hdc(hdc: HDC, spool: &[u8]) -> Result<(), PrintFailure> {
    let doc_name = wide_null("KR580 printer output");
    let doc = DOCINFOW {
        cbSize: std::mem::size_of::<DOCINFOW>() as i32,
        lpszDocName: doc_name.as_ptr(),
        lpszOutput: null(),
        lpszDatatype: null(),
        fwType: 0,
    };
    let font_name = wide_null("Consolas");
    let dpi_y = device_cap(hdc, LOGPIXELSY, 96);
    // SAFETY: The font name is terminated UTF-16 and all other arguments are integer font attributes.
    let font = unsafe {
        CreateFontW(
            -((10 * dpi_y) / 72).max(1),
            0,
            0,
            0,
            FW_NORMAL as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET as u32,
            OUT_DEFAULT_PRECIS as u32,
            CLIP_DEFAULT_PRECIS as u32,
            DEFAULT_QUALITY as u32,
            (FIXED_PITCH | FF_MODERN) as u32,
            font_name.as_ptr(),
        )
    };
    let old_font = if font.is_null() {
        null_mut()
    } else {
        // SAFETY: The private caller owns the live DC and the selected font is a successful CreateFontW result.
        unsafe { SelectObject(hdc, font as HGDIOBJ) }
    };
    let _font = SelectedFont {
        hdc,
        font: font as HGDIOBJ,
        previous: old_font,
    };
    let job = PrintJob::start(hdc, &doc)?;
    print_pages(hdc, spool)?;
    job.finish()
}

fn print_pages(hdc: HDC, spool: &[u8]) -> Result<(), PrintFailure> {
    let lines = text::printer_lines(spool);
    let dpi_x = device_cap(hdc, LOGPIXELSX, 96);
    let dpi_y = device_cap(hdc, LOGPIXELSY, 96);
    let margin_x = mm_to_px(MARGIN_MM, dpi_x);
    let margin_y = mm_to_px(MARGIN_MM, dpi_y);
    let line_height = line_height(hdc, dpi_y);
    let page_height = device_cap(hdc, VERTRES, 1100);
    let usable_height = (page_height - margin_y * 2).max(line_height);
    let lines_per_page = (usable_height / line_height).max(1) as usize;
    let x = margin_x.min(device_cap(hdc, HORZRES, 800).saturating_sub(1));

    for page_lines in lines.chunks(lines_per_page) {
        // SAFETY: The private caller keeps the live DC and successful document job until all pages finish.
        if unsafe { StartPage(hdc) } <= 0 {
            return Err(last_print_error("StartPage"));
        }
        let mut y = margin_y;
        for line in page_lines {
            if !line.is_empty() {
                let text = wide(line);
                // SAFETY: The initialized UTF-16 slice covers the text count and the owned DC has an active page.
                if unsafe { TextOutW(hdc, x, y, text.as_ptr(), text.len() as i32) } == 0 {
                    return Err(last_print_error("TextOutW"));
                }
            }
            y += line_height;
        }
        // SAFETY: The owned DC has a page opened successfully above.
        if unsafe { EndPage(hdc) } <= 0 {
            return Err(last_print_error("EndPage"));
        }
    }
    Ok(())
}

fn line_height(hdc: HDC, dpi_y: i32) -> i32 {
    let mut metrics = TEXTMETRICW::default();
    // SAFETY: The owned DC is live and metrics is an initialized correctly sized writable POD output.
    if unsafe { GetTextMetricsW(hdc, &mut metrics) } != 0 {
        return (metrics.tmHeight + metrics.tmExternalLeading).max(1);
    }
    ((12 * dpi_y) / 72).max(1)
}

fn device_cap(hdc: HDC, index: u32, fallback: i32) -> i32 {
    // SAFETY: The private caller keeps the owned printer DC live and passes a documented capability index.
    let value = unsafe { GetDeviceCaps(hdc, index as i32) };
    if value > 0 { value } else { fallback }
}

fn mm_to_px(mm: f32, dpi: i32) -> i32 {
    ((mm / 25.4) * dpi.max(1) as f32).round().max(1.0) as i32
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().collect()
}

fn last_print_error(api: &str) -> PrintFailure {
    // SAFETY: GetLastError only reads thread-local Win32 error state immediately after the failing call.
    let code = unsafe { GetLastError() };
    if code == ERROR_CANCELLED {
        PrintFailure::Cancelled
    } else {
        let error = std::io::Error::from_raw_os_error(code as i32);
        PrintFailure::Failed(format!("{api} failed ({code}): {error}"))
    }
}

struct PrinterDc(HDC);

impl Drop for PrinterDc {
    fn drop(&mut self) {
        // SAFETY: This guard owns CreateDCW's successful result and all fonts/jobs were dropped first.
        unsafe { DeleteDC(self.0) };
    }
}

struct SelectedFont {
    hdc: HDC,
    font: HGDIOBJ,
    previous: HGDIOBJ,
}

impl Drop for SelectedFont {
    fn drop(&mut self) {
        if !self.previous.is_null() && self.previous as isize != -1 {
            // SAFETY: The live DC returned this prior selected font, which we restore before deletion.
            unsafe { SelectObject(self.hdc, self.previous) };
        }
        if !self.font.is_null() {
            // SAFETY: This is the owned CreateFontW object, no longer selected into the live DC.
            unsafe { DeleteObject(self.font) };
        }
    }
}

struct PrintJob {
    hdc: HDC,
    finished: bool,
}

impl PrintJob {
    fn start(hdc: HDC, doc: &DOCINFOW) -> Result<Self, PrintFailure> {
        // SAFETY: The private caller owns the live printer DC and all DOCINFOW strings outlive this call.
        if unsafe { StartDocW(hdc, doc) } <= 0 {
            return Err(last_print_error("StartDocW"));
        }
        Ok(Self {
            hdc,
            finished: false,
        })
    }

    fn finish(mut self) -> Result<(), PrintFailure> {
        // SAFETY: StartDocW succeeded for this live DC and every printed page has been ended.
        if unsafe { EndDoc(self.hdc) } <= 0 {
            return Err(last_print_error("EndDoc"));
        }
        self.finished = true;
        Ok(())
    }
}

impl Drop for PrintJob {
    fn drop(&mut self) {
        if !self.finished {
            // SAFETY: This guard owns an unfinished StartDocW job on a still-live printer DC.
            unsafe { AbortDoc(self.hdc) };
        }
    }
}
