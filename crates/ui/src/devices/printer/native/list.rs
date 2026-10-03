use super::super::{PrinterInfo, PrinterStatus};
use super::last_os_error;
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
        // SAFETY: Null output with zero capacity queries the required byte count into live outputs.
        unsafe { EnumPrintersW(flags, null(), 2, null_mut(), 0, &mut needed, &mut returned) };
    if first == 0 && needed == 0 {
        return Err(last_os_error("EnumPrintersW"));
    }
    if needed == 0 {
        return Ok(Vec::new());
    }

    let mut buffer = vec![0usize; (needed as usize).div_ceil(std::mem::size_of::<usize>())];
    // SAFETY: The aligned initialized allocation covers the requested byte count and outputs remain live.
    let ok = unsafe {
        EnumPrintersW(
            flags,
            null(),
            2,
            buffer.as_mut_ptr().cast(),
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
    let size = buffer.len() * std::mem::size_of::<usize>();
    if returned as usize > size / item_size {
        return Err("printer enumeration count exceeds its buffer".to_owned());
    }
    // SAFETY: The initialized word allocation covers size bytes and outlives the borrowed view.
    let bytes = unsafe { std::slice::from_raw_parts(buffer.as_ptr().cast::<u8>(), size) };
    Ok(bytes
        .chunks_exact(item_size)
        .take(returned as usize)
        .filter_map(|item| {
            // SAFETY: Each chunk contains a complete POD PRINTER_INFO_2W; alignment is not assumed.
            let printer = unsafe { item.as_ptr().cast::<PRINTER_INFO_2W>().read_unaligned() };
            printer_info(&printer, bytes, default_name.as_deref())
        })
        .collect())
}

pub(super) fn default_printer_name() -> Result<String, String> {
    let mut len = 0;
    // SAFETY: Null output queries the required UTF-16 length into a live output.
    unsafe {
        GetDefaultPrinterW(null_mut(), &mut len);
    }
    if len == 0 {
        return Err(last_os_error("GetDefaultPrinterW"));
    }
    let mut buffer = vec![0u16; len as usize];
    // SAFETY: The initialized UTF-16 allocation has the queried character capacity and remains live.
    if unsafe { GetDefaultPrinterW(buffer.as_mut_ptr(), &mut len) } == 0 {
        return Err(last_os_error("GetDefaultPrinterW"));
    }
    buffer.truncate(len as usize);
    while buffer.last() == Some(&0) {
        buffer.pop();
    }
    String::from_utf16(&buffer).map_err(|error| error.to_string())
}

fn printer_info(
    printer: &PRINTER_INFO_2W,
    bytes: &[u8],
    default_name: Option<&str>,
) -> Option<PrinterInfo> {
    let name = read_wide_ptr(bytes, printer.pPrinterName).ok()?;
    let is_default = (printer.Attributes & PRINTER_ATTRIBUTE_DEFAULT) != 0
        || default_name.is_some_and(|default| default.eq_ignore_ascii_case(&name));
    Some(PrinterInfo {
        name,
        driver: read_wide_ptr(bytes, printer.pDriverName).unwrap_or_default(),
        port: read_wide_ptr(bytes, printer.pPortName).unwrap_or_default(),
        location: read_wide_ptr(bytes, printer.pLocation).unwrap_or_default(),
        comment: read_wide_ptr(bytes, printer.pComment).unwrap_or_default(),
        status: printer_status(printer.Status),
        is_default,
    })
}

fn printer_status(status: u32) -> PrinterStatus {
    if status == 0 {
        return PrinterStatus::Ready;
    }
    for (flag, reported) in [
        (PRINTER_STATUS_PAUSED, PrinterStatus::Paused),
        (PRINTER_STATUS_ERROR, PrinterStatus::Error),
        (
            PRINTER_STATUS_PENDING_DELETION,
            PrinterStatus::PendingDeletion,
        ),
        (PRINTER_STATUS_PAPER_JAM, PrinterStatus::PaperJam),
        (PRINTER_STATUS_PAPER_OUT, PrinterStatus::PaperOut),
        (PRINTER_STATUS_MANUAL_FEED, PrinterStatus::ManualFeed),
        (PRINTER_STATUS_PAPER_PROBLEM, PrinterStatus::PaperProblem),
        (PRINTER_STATUS_OFFLINE, PrinterStatus::Offline),
        (PRINTER_STATUS_BUSY, PrinterStatus::Busy),
        (PRINTER_STATUS_PRINTING, PrinterStatus::Printing),
        (PRINTER_STATUS_OUTPUT_BIN_FULL, PrinterStatus::OutputBinFull),
        (PRINTER_STATUS_NOT_AVAILABLE, PrinterStatus::NotAvailable),
        (PRINTER_STATUS_WAITING, PrinterStatus::Waiting),
        (PRINTER_STATUS_PROCESSING, PrinterStatus::Processing),
        (PRINTER_STATUS_INITIALIZING, PrinterStatus::Initializing),
        (PRINTER_STATUS_WARMING_UP, PrinterStatus::WarmingUp),
        (PRINTER_STATUS_TONER_LOW, PrinterStatus::TonerLow),
        (PRINTER_STATUS_NO_TONER, PrinterStatus::NoToner),
        (
            PRINTER_STATUS_USER_INTERVENTION,
            PrinterStatus::UserIntervention,
        ),
        (PRINTER_STATUS_OUT_OF_MEMORY, PrinterStatus::OutOfMemory),
        (PRINTER_STATUS_DOOR_OPEN, PrinterStatus::DoorOpen),
    ] {
        if (status & flag) != 0 {
            return reported;
        }
    }
    PrinterStatus::Unknown
}

fn read_wide_ptr(bytes: &[u8], ptr: *const u16) -> Result<String, String> {
    if ptr.is_null() {
        return Ok(String::new());
    }
    let offset = (ptr as usize)
        .checked_sub(bytes.as_ptr() as usize)
        .ok_or_else(|| "printer string precedes its buffer".to_owned())?;
    super::buffer::wide_string(bytes, offset)
}
