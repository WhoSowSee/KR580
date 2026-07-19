use super::super::PrinterInfo;
use super::{last_os_error, read_wide_z};
use std::ptr::{null, null_mut};
use windows_sys::Win32::Graphics::Printing::{
    EnumPrintersW, GetDefaultPrinterW, PRINTER_ATTRIBUTE_DEFAULT, PRINTER_ENUM_CONNECTIONS,
    PRINTER_ENUM_LOCAL, PRINTER_INFO_2W, PRINTER_STATUS_BUSY, PRINTER_STATUS_DOOR_OPEN,
    PRINTER_STATUS_ERROR, PRINTER_STATUS_INITIALIZING, PRINTER_STATUS_MANUAL_FEED,
    PRINTER_STATUS_NO_TONER, PRINTER_STATUS_NOT_AVAILABLE, PRINTER_STATUS_OFFLINE,
    PRINTER_STATUS_OUT_OF_MEMORY, PRINTER_STATUS_OUTPUT_BIN_FULL, PRINTER_STATUS_PAPER_JAM,
    PRINTER_STATUS_PAPER_OUT, PRINTER_STATUS_PAPER_PROBLEM, PRINTER_STATUS_PAUSED,
    PRINTER_STATUS_PENDING_DELETION, PRINTER_STATUS_PRINTING, PRINTER_STATUS_PROCESSING,
    PRINTER_STATUS_TONER_LOW, PRINTER_STATUS_USER_INTERVENTION, PRINTER_STATUS_WAITING,
    PRINTER_STATUS_WARMING_UP,
};

pub(super) fn printers() -> Result<Vec<PrinterInfo>, String> {
    let flags = PRINTER_ENUM_LOCAL | PRINTER_ENUM_CONNECTIONS;
    let mut needed = 0;
    let mut returned = 0;
    let first =
        unsafe { EnumPrintersW(flags, null(), 2, null_mut(), 0, &mut needed, &mut returned) };
    if first == 0 && needed == 0 {
        return Err(last_os_error("EnumPrintersW"));
    }
    if needed == 0 {
        return Ok(Vec::new());
    }

    let mut buffer = vec![0u8; needed as usize];
    let ok = unsafe {
        EnumPrintersW(
            flags,
            null(),
            2,
            buffer.as_mut_ptr(),
            needed,
            &mut needed,
            &mut returned,
        )
    };
    if ok == 0 {
        return Err(last_os_error("EnumPrintersW"));
    }

    let default_name = default_printer_name().ok();
    let item_size = std::mem::size_of::<PRINTER_INFO_2W>();
    Ok((0..returned as usize)
        .filter_map(|index| {
            let ptr = unsafe {
                buffer
                    .as_ptr()
                    .add(index * item_size)
                    .cast::<PRINTER_INFO_2W>()
            };
            let printer = unsafe { ptr.read_unaligned() };
            printer_info(&printer, default_name.as_deref())
        })
        .collect())
}

pub(super) fn default_printer_name() -> Result<String, String> {
    let mut len = 0;
    unsafe {
        GetDefaultPrinterW(null_mut(), &mut len);
    }
    if len == 0 {
        return Err(last_os_error("GetDefaultPrinterW"));
    }
    let mut buffer = vec![0u16; len as usize];
    if unsafe { GetDefaultPrinterW(buffer.as_mut_ptr(), &mut len) } == 0 {
        return Err(last_os_error("GetDefaultPrinterW"));
    }
    buffer.truncate(len as usize);
    while buffer.last() == Some(&0) {
        buffer.pop();
    }
    String::from_utf16(&buffer).map_err(|error| error.to_string())
}

fn printer_info(printer: &PRINTER_INFO_2W, default_name: Option<&str>) -> Option<PrinterInfo> {
    let name = read_wide_ptr(printer.pPrinterName).ok()?;
    let is_default = (printer.Attributes & PRINTER_ATTRIBUTE_DEFAULT) != 0
        || default_name.is_some_and(|default| default.eq_ignore_ascii_case(&name));
    Some(PrinterInfo {
        name,
        driver: read_wide_ptr(printer.pDriverName).unwrap_or_default(),
        port: read_wide_ptr(printer.pPortName).unwrap_or_default(),
        location: read_wide_ptr(printer.pLocation).unwrap_or_default(),
        comment: read_wide_ptr(printer.pComment).unwrap_or_default(),
        status: printer_status_label(printer.Status).to_owned(),
        is_default,
    })
}

fn printer_status_label(status: u32) -> &'static str {
    if status == 0 {
        return "Ready";
    }
    for (flag, label) in [
        (PRINTER_STATUS_PAUSED, "Paused"),
        (PRINTER_STATUS_ERROR, "Error"),
        (PRINTER_STATUS_PENDING_DELETION, "Pending deletion"),
        (PRINTER_STATUS_PAPER_JAM, "Paper jam"),
        (PRINTER_STATUS_PAPER_OUT, "Paper out"),
        (PRINTER_STATUS_MANUAL_FEED, "Manual feed"),
        (PRINTER_STATUS_PAPER_PROBLEM, "Paper problem"),
        (PRINTER_STATUS_OFFLINE, "Offline"),
        (PRINTER_STATUS_BUSY, "Busy"),
        (PRINTER_STATUS_PRINTING, "Printing"),
        (PRINTER_STATUS_OUTPUT_BIN_FULL, "Output bin full"),
        (PRINTER_STATUS_NOT_AVAILABLE, "Not available"),
        (PRINTER_STATUS_WAITING, "Waiting"),
        (PRINTER_STATUS_PROCESSING, "Processing"),
        (PRINTER_STATUS_INITIALIZING, "Initializing"),
        (PRINTER_STATUS_WARMING_UP, "Warming up"),
        (PRINTER_STATUS_TONER_LOW, "Toner low"),
        (PRINTER_STATUS_NO_TONER, "No toner"),
        (PRINTER_STATUS_USER_INTERVENTION, "User intervention"),
        (PRINTER_STATUS_OUT_OF_MEMORY, "Out of memory"),
        (PRINTER_STATUS_DOOR_OPEN, "Door open"),
    ] {
        if (status & flag) != 0 {
            return label;
        }
    }
    "Unknown"
}

fn read_wide_ptr(ptr: *const u16) -> Result<String, String> {
    if ptr.is_null() {
        return Ok(String::new());
    }
    read_wide_z(ptr)
}
