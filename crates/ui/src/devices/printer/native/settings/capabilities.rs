use super::super::super::{PrinterInfo, PrinterPaper, PrinterSource};
use super::super::wide_null;
use std::ptr::{null, null_mut};
use windows_sys::Win32::Graphics::Gdi::DEVMODEW;
use windows_sys::Win32::Storage::Xps::{
    DC_BINNAMES, DC_BINS, DC_PAPERNAMES, DC_PAPERS, DeviceCapabilitiesW,
};

const PAPER_NAME_LEN: usize = 64;
const SOURCE_NAME_LEN: usize = 24;

pub(super) fn load(
    printer: &PrinterInfo,
    devmode: *const DEVMODEW,
) -> Result<(Vec<PrinterPaper>, Vec<PrinterSource>), String> {
    let device = wide_null(&printer.name);
    let port = (!printer.port.is_empty()).then(|| wide_null(&printer.port));
    let port_ptr = port.as_ref().map_or(null(), |port| port.as_ptr());
    let papers = query_pairs(
        device.as_ptr(),
        port_ptr,
        devmode,
        DC_PAPERS,
        DC_PAPERNAMES,
        PAPER_NAME_LEN,
    )?
    .into_iter()
    .map(|(id, name)| PrinterPaper { id, name })
    .collect();
    let sources = query_pairs(
        device.as_ptr(),
        port_ptr,
        devmode,
        DC_BINS,
        DC_BINNAMES,
        SOURCE_NAME_LEN,
    )?
    .into_iter()
    .map(|(id, name)| PrinterSource { id, name })
    .collect();
    Ok((papers, sources))
}

fn query_pairs(
    device: *const u16,
    port: *const u16,
    devmode: *const DEVMODEW,
    ids_capability: u16,
    names_capability: u16,
    name_len: usize,
) -> Result<Vec<(i16, String)>, String> {
    // SAFETY: The private caller retains terminated names and a validated aligned DEVMODE during queries.
    let count = unsafe { DeviceCapabilitiesW(device, port, ids_capability, null_mut(), devmode) };
    if count < 0 {
        return Ok(Vec::new());
    }
    if count == 0 {
        return Ok(Vec::new());
    }
    if count > 65_536 {
        return Err("printer capability count exceeds the buffer limit".to_owned());
    }
    let mut ids = vec![0u16; count as usize];
    let ids_count =
        // SAFETY: Windows' count query sizes this initialized output for the same capability/DEVMODE.
        unsafe { DeviceCapabilitiesW(device, port, ids_capability, ids.as_mut_ptr(), devmode) };
    if ids_count < 0 {
        return Err("DeviceCapabilitiesW failed while reading identifiers".to_owned());
    }
    if ids_count as usize > ids.len() {
        return Err("printer identifier count changed during enumeration".to_owned());
    }
    ids.truncate(ids_count as usize);

    // SAFETY: Live terminated names and aligned DEVMODE are retained; null output queries name count.
    let count = unsafe { DeviceCapabilitiesW(device, port, names_capability, null_mut(), devmode) };
    if count > 65_536 {
        return Err("printer name count exceeds the buffer limit".to_owned());
    }
    let mut names = vec![0u16; count.max(0) as usize * name_len];
    if count > 0 {
        // SAFETY: The same name-capability query sized fixed-width UTF-16 entries in initialized storage.
        let names_count = unsafe {
            DeviceCapabilitiesW(device, port, names_capability, names.as_mut_ptr(), devmode)
        };
        if names_count > count {
            return Err("printer name count changed during enumeration".to_owned());
        }
        names.truncate(names_count.max(0) as usize * name_len);
    }
    Ok(ids
        .into_iter()
        .enumerate()
        .map(|(index, id)| {
            let start = index * name_len;
            let end = start + name_len;
            let name = names
                .get(start..end)
                .map(fixed_wide_string)
                .unwrap_or_default();
            let name = if name.is_empty() {
                id.to_string()
            } else {
                name
            };
            (id as i16, name)
        })
        .collect())
}

fn fixed_wide_string(buffer: &[u16]) -> String {
    let end = buffer
        .iter()
        .position(|value| *value == 0)
        .unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..end]).trim().to_owned()
}
