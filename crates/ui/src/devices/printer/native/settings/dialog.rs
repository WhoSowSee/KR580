mod global;

use super::super::last_os_error;
use std::ptr::{null, null_mut};
use windows_sys::Win32::Foundation::{HGLOBAL, HWND};
use windows_sys::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize};
use windows_sys::Win32::UI::Controls::Dialogs::{
    CommDlgExtendedError, DEVNAMES, PD_PRINTSETUP, PD_USEDEVMODECOPIESANDCOLLATE, PRINTDLGW,
    PrintDlgW,
};

pub(super) fn configure() -> Result<Option<(String, Vec<u8>)>, String> {
    let _com = ComApartment::init()?;
    let mut dialog = DialogHandles(print_dialog());
    // SAFETY: The initialized structure has the correct size and no borrowed hook/template pointers.
    let accepted = unsafe { PrintDlgW(&mut dialog.0) };
    if accepted == 0 {
        // SAFETY: CommDlgExtendedError reads the failure state of PrintDlgW on this same thread.
        let code = unsafe { CommDlgExtendedError() };
        return if code == 0 {
            Ok(None)
        } else {
            Err(format!("printer setup failed: 0x{code:04X}"))
        };
    }
    let printer_name = device_name(dialog.0.hDevNames)?;
    let devmode = devmode(dialog.0.hDevMode)?;
    Ok(Some((printer_name, devmode)))
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

fn device_name(handle: HGLOBAL) -> Result<String, String> {
    let memory = global::LockedGlobal::lock(handle)?;
    let bytes = memory.bytes();
    if bytes.len() < std::mem::size_of::<DEVNAMES>() {
        return Err("printer dialog DEVNAMES header is truncated".to_owned());
    }
    // SAFETY: The byte view contains the complete POD header; no alignment is assumed.
    let names = unsafe { bytes.as_ptr().cast::<DEVNAMES>().read_unaligned() };
    super::super::buffer::wide_string(bytes, names.wDeviceOffset as usize * 2)
}

fn devmode(handle: HGLOBAL) -> Result<Vec<u8>, String> {
    let memory = global::LockedGlobal::lock(handle)?;
    let bytes = memory.bytes();
    let mode = super::read_devmode(bytes)
        .ok_or_else(|| "printer dialog DEVMODE buffer is invalid".to_owned())?;
    Ok(bytes[..mode.dmSize as usize + mode.dmDriverExtra as usize].to_vec())
}

struct DialogHandles(PRINTDLGW);

impl Drop for DialogHandles {
    fn drop(&mut self) {
        if !self.0.hDC.is_null() {
            // SAFETY: PrintDlgW returned this owned DC and all dialog reads have finished.
            unsafe { windows_sys::Win32::Graphics::Gdi::DeleteDC(self.0.hDC) };
        }
        // SAFETY: These non-null handles are owned PrintDlgW outputs and no locks remain alive.
        unsafe {
            if !self.0.hDevMode.is_null() {
                windows_sys::Win32::Foundation::GlobalFree(self.0.hDevMode);
            }
            if !self.0.hDevNames.is_null() {
                windows_sys::Win32::Foundation::GlobalFree(self.0.hDevNames);
            }
        }
    }
}

struct ComApartment(std::marker::PhantomData<std::rc::Rc<()>>);

impl ComApartment {
    fn init() -> Result<Self, String> {
        // SAFETY: The reserved pointer is null; a successful initialization is paired on this thread with Drop.
        let result = unsafe { CoInitializeEx(null_mut(), COINIT_APARTMENTTHREADED as u32) };
        if result < 0 {
            return Err(format!(
                "printer setup COM initialization failed: 0x{:08X}",
                result as u32
            ));
        }
        Ok(Self(std::marker::PhantomData))
    }
}

impl Drop for ComApartment {
    fn drop(&mut self) {
        // SAFETY: The non-Send guard pairs one successful CoInitializeEx on this same thread.
        unsafe {
            CoUninitialize();
        }
    }
}
