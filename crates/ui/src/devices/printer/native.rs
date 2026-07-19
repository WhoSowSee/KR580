#[cfg(windows)]
mod list;
#[cfg(windows)]
mod settings;

#[cfg(windows)]
fn wide_null(value: &str) -> Vec<u16> {
    value.encode_utf16().chain([0]).collect()
}

#[cfg(windows)]
fn read_wide_z(mut ptr: *const u16) -> Result<String, String> {
    let mut data = Vec::new();
    unsafe {
        while !ptr.is_null() && *ptr != 0 {
            data.push(*ptr);
            ptr = ptr.add(1);
        }
    }
    String::from_utf16(&data).map_err(|error| error.to_string())
}

#[cfg(windows)]
fn last_os_error(api: &str) -> String {
    let code = unsafe { windows_sys::Win32::Foundation::GetLastError() };
    let error = std::io::Error::from_raw_os_error(code as i32);
    format!("{api} failed ({code}): {error}")
}

#[derive(Debug)]
pub(super) enum PrintFailure {
    Cancelled,
    Failed(String),
}

#[cfg(windows)]
mod imp {
    use super::super::{
        PrinterConfiguration, PrinterInfo, PrinterPropertyChange, PrinterPropertySheet,
        PrinterSettings,
    };
    use super::{PrintFailure, list as printer_list, settings, wide_null};
    use crate::devices::printer::text;
    use std::ptr::{null, null_mut};
    use windows_sys::Win32::Foundation::{ERROR_CANCELLED, GetLastError};
    use windows_sys::Win32::Graphics::Gdi::{
        CLIP_DEFAULT_PRECIS, CreateDCW, CreateFontW, DEFAULT_CHARSET, DEFAULT_QUALITY, DeleteDC,
        DeleteObject, FF_MODERN, FIXED_PITCH, FW_NORMAL, GetDeviceCaps, GetTextMetricsW, HDC,
        HGDIOBJ, HORZRES, LOGPIXELSX, LOGPIXELSY, OUT_DEFAULT_PRECIS, SelectObject, TEXTMETRICW,
        TextOutW, VERTRES,
    };
    use windows_sys::Win32::Storage::Xps::{
        AbortDoc, DOCINFOW, EndDoc, EndPage, StartDocW, StartPage,
    };

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
        let hdc = unsafe { CreateDCW(driver.as_ptr(), device.as_ptr(), null(), devmode_ptr) };
        if hdc.is_null() {
            return Err(last_print_error("CreateDCW"));
        }

        let result = print_to_hdc(hdc, spool);
        unsafe {
            DeleteDC(hdc);
        }
        result
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
            unsafe { SelectObject(hdc, font as HGDIOBJ) }
        };
        let result = unsafe {
            if StartDocW(hdc, &doc) <= 0 {
                Err(last_print_error("StartDocW"))
            } else {
                match print_pages(hdc, spool) {
                    Ok(()) if EndDoc(hdc) > 0 => Ok(()),
                    Ok(()) => Err(last_print_error("EndDoc")),
                    Err(error) => {
                        AbortDoc(hdc);
                        Err(error)
                    }
                }
            }
        };
        if !old_font.is_null() {
            unsafe {
                SelectObject(hdc, old_font);
            }
        }
        if !font.is_null() {
            unsafe {
                DeleteObject(font as HGDIOBJ);
            }
        }
        result
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
            if unsafe { StartPage(hdc) } <= 0 {
                return Err(last_print_error("StartPage"));
            }
            let mut y = margin_y;
            for line in page_lines {
                if !line.is_empty() {
                    let text = wide(line);
                    if unsafe { TextOutW(hdc, x, y, text.as_ptr(), text.len() as i32) } == 0 {
                        return Err(last_print_error("TextOutW"));
                    }
                }
                y += line_height;
            }
            if unsafe { EndPage(hdc) } <= 0 {
                return Err(last_print_error("EndPage"));
            }
        }
        Ok(())
    }

    fn line_height(hdc: HDC, dpi_y: i32) -> i32 {
        let mut metrics = unsafe { std::mem::zeroed::<TEXTMETRICW>() };
        if unsafe { GetTextMetricsW(hdc, &mut metrics) } != 0 {
            return (metrics.tmHeight + metrics.tmExternalLeading).max(1);
        }
        ((12 * dpi_y) / 72).max(1)
    }

    fn device_cap(hdc: HDC, index: u32, fallback: i32) -> i32 {
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
        let code = unsafe { GetLastError() };
        if code == ERROR_CANCELLED {
            PrintFailure::Cancelled
        } else {
            let error = std::io::Error::from_raw_os_error(code as i32);
            PrintFailure::Failed(format!("{api} failed ({code}): {error}"))
        }
    }
}

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
