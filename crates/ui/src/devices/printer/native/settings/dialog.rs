use super::super::{last_os_error, read_wide_z};
use std::ptr::{null, null_mut};
use windows_sys::Win32::Foundation::{HGLOBAL, HWND};
use windows_sys::Win32::Graphics::Gdi::DEVMODEW;
use windows_sys::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize};
use windows_sys::Win32::System::Memory::{GlobalLock, GlobalUnlock};
use windows_sys::Win32::UI::Controls::Dialogs::{
    CommDlgExtendedError, DEVNAMES, PD_PRINTSETUP, PD_USEDEVMODECOPIESANDCOLLATE, PRINTDLGW,
    PrintDlgW,
};

pub(super) fn configure() -> Result<Option<(String, Vec<u8>)>, String> {
    let _com = ComApartment::init()?;
    let mut dialog = print_dialog();
    let accepted = unsafe { PrintDlgW(&mut dialog) };
    if accepted == 0 {
        let code = unsafe { CommDlgExtendedError() };
        free_dialog_handles(&mut dialog);
        return if code == 0 {
            Ok(None)
        } else {
            Err(format!("printer setup failed: 0x{code:04X}"))
        };
    }
    let result = (|| {
        let printer_name = device_name(dialog.hDevNames)?;
        let devmode = devmode(dialog.hDevMode)?;
        Ok((printer_name, devmode))
    })();
    free_dialog_handles(&mut dialog);
    result.map(Some)
}

fn print_dialog() -> PRINTDLGW {
    PRINTDLGW {
        lStructSize: std::mem::size_of::<PRINTDLGW>() as u32,
        hwndOwner: null_mut::<std::ffi::c_void>() as HWND,
        hDevMode: null_mut(),
        hDevNames: null_mut(),
        hDC: null_mut(),
        Flags: PD_PRINTSETUP | PD_USEDEVMODECOPIESANDCOLLATE,
        nFromPage: 0,
        nToPage: 0,
        nMinPage: 0,
        nMaxPage: 0,
        nCopies: 1,
        hInstance: null_mut(),
        lCustData: 0,
        lpfnPrintHook: None,
        lpfnSetupHook: None,
        lpPrintTemplateName: null(),
        lpSetupTemplateName: null(),
        hPrintTemplate: null_mut(),
        hSetupTemplate: null_mut(),
    }
}

fn device_name(hdevnames: HGLOBAL) -> Result<String, String> {
    let ptr = unsafe { GlobalLock(hdevnames) as *const DEVNAMES };
    if ptr.is_null() {
        return Err(last_os_error("GlobalLock"));
    }
    let result = unsafe {
        let names = *ptr;
        let base = ptr.cast::<u16>();
        read_wide_z(base.add(names.wDeviceOffset as usize))
    };
    unsafe {
        GlobalUnlock(hdevnames);
    }
    result
}

fn devmode(hdevmode: HGLOBAL) -> Result<Vec<u8>, String> {
    let ptr = unsafe { GlobalLock(hdevmode) as *const DEVMODEW };
    if ptr.is_null() {
        return Err(last_os_error("GlobalLock"));
    }
    let mode = unsafe { ptr.read_unaligned() };
    let len = mode.dmSize as usize + mode.dmDriverExtra as usize;
    let bytes = unsafe { std::slice::from_raw_parts(ptr.cast::<u8>(), len).to_vec() };
    unsafe {
        GlobalUnlock(hdevmode);
    }
    Ok(bytes)
}

fn free_dialog_handles(dialog: &mut PRINTDLGW) {
    if !dialog.hDC.is_null() {
        unsafe {
            windows_sys::Win32::Graphics::Gdi::DeleteDC(dialog.hDC);
        }
    }
    unsafe {
        if !dialog.hDevMode.is_null() {
            windows_sys::Win32::Foundation::GlobalFree(dialog.hDevMode);
        }
        if !dialog.hDevNames.is_null() {
            windows_sys::Win32::Foundation::GlobalFree(dialog.hDevNames);
        }
    }
}

pub(super) struct ComApartment;

impl ComApartment {
    pub(super) fn init() -> Result<Self, String> {
        let result = unsafe { CoInitializeEx(null_mut(), COINIT_APARTMENTTHREADED as u32) };
        if result < 0 {
            return Err(format!(
                "printer setup COM initialization failed: 0x{:08X}",
                result as u32
            ));
        }
        Ok(Self)
    }
}

impl Drop for ComApartment {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}
