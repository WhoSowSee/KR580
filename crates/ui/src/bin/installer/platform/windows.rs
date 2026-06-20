mod environment;
mod system;

pub use system::{IntegrationReport, IntegrationRequest};

use k580_ui::install_mode::InstallScope;
use std::path::{Path, PathBuf};

pub fn default_system_install_dir(scope: InstallScope) -> PathBuf {
    match scope {
        InstallScope::User => std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Programs")
            .join("KR580"),
        InstallScope::Machine => std::env::var_os("ProgramFiles")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(r"C:\Program Files"))
            .join("KR580"),
    }
}

pub fn set_rounded_corners(window: &dyn iced::window::Window) {
    use iced::window::raw_window_handle::RawWindowHandle;
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::Graphics::Dwm::{
        DWM_WINDOW_CORNER_PREFERENCE, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND,
        DwmSetWindowAttribute,
    };

    let Ok(handle) = window.window_handle() else {
        return;
    };

    let RawWindowHandle::Win32(win32) = handle.as_raw() else {
        return;
    };

    let hwnd = win32.hwnd.get() as HWND;
    let value: DWM_WINDOW_CORNER_PREFERENCE = DWMWCP_ROUND;

    // SAFETY: HWND comes from winit's live window; the DWM attribute is a POD integer.
    unsafe {
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE as u32,
            std::ptr::from_ref(&value).cast(),
            std::mem::size_of::<DWM_WINDOW_CORNER_PREFERENCE>() as u32,
        );
    }
}

pub fn add_to_path(bin_dir: &Path, scope: InstallScope) -> Result<bool, String> {
    environment::add_to_path(bin_dir, scope)
}

pub fn remove_from_path(bin_dir: &Path, scope: InstallScope) -> Result<bool, String> {
    environment::remove_from_path(bin_dir, scope)
}

pub fn install_system_integration(
    request: &IntegrationRequest<'_>,
) -> Result<IntegrationReport, String> {
    system::install(request)
}

pub fn remove_system_integration(install_dir: &Path, scope: InstallScope) -> Result<(), String> {
    system::remove(install_dir, scope)
}

pub fn schedule_remove_install_dir(install_dir: &Path) -> Result<(), String> {
    system::schedule_remove_install_dir(install_dir)
}
