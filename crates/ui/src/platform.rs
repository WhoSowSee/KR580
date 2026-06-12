//! Windows window helpers for startup cloaking and rounded corners.

#[cfg(windows)]
pub(crate) fn cloak_window(window: &dyn iced::window::Window, cloaked: bool) {
    use iced::window::raw_window_handle::RawWindowHandle;
    use windows_sys::Win32::Foundation::{BOOL, FALSE, HWND, TRUE};
    use windows_sys::Win32::Graphics::Dwm::{DWMWA_CLOAK, DwmSetWindowAttribute};

    let Ok(handle) = window.window_handle() else {
        return;
    };

    let RawWindowHandle::Win32(win32) = handle.as_raw() else {
        return;
    };

    let hwnd = win32.hwnd.get() as HWND;
    let value: BOOL = if cloaked { TRUE } else { FALSE };

    // SAFETY: HWND comes from winit's live window; the attribute is
    // a POD `BOOL`. Older Windows without DWM cloak silently fail
    // back to the launch flash.
    unsafe {
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_CLOAK as u32,
            std::ptr::from_ref(&value).cast(),
            std::mem::size_of::<BOOL>() as u32,
        );
    }
}

#[cfg(windows)]
pub(crate) fn set_rounded_corners(window: &dyn iced::window::Window) {
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

    // SAFETY: HWND from winit; attribute is a POD `i32`; pointer is
    // read-only for the call duration.
    unsafe {
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE as u32,
            std::ptr::from_ref(&value).cast(),
            std::mem::size_of::<DWM_WINDOW_CORNER_PREFERENCE>() as u32,
        );
    }
}

#[cfg(not(windows))]
pub(crate) fn cloak_window(_window: &dyn iced::window::Window, _cloaked: bool) {}

#[cfg(not(windows))]
pub(crate) fn set_rounded_corners(_window: &dyn iced::window::Window) {}

pub(crate) const SUPPORTS_HIDDEN_WINDOW_REUSE: bool = cfg!(windows);

/// `Some(2..=480)` on Windows when the OS reports a believable refresh
/// rate. Callers fall back to 60 Hz on `None`.
#[cfg(windows)]
pub(crate) fn primary_monitor_refresh_hz() -> Option<u32> {
    use windows_sys::Win32::Graphics::Gdi::{
        DEVMODEW, ENUM_CURRENT_SETTINGS, EnumDisplaySettingsW,
    };

    // SAFETY: `DEVMODEW` is a POD struct the Win32 API fills in;
    // zero-init is the documented way to query current settings.
    let mut mode: DEVMODEW = unsafe { std::mem::zeroed() };
    mode.dmSize = std::mem::size_of::<DEVMODEW>() as u16;
    let ok = unsafe { EnumDisplaySettingsW(std::ptr::null(), ENUM_CURRENT_SETTINGS, &mut mode) };
    if ok == 0 {
        return None;
    }
    let hz = mode.dmDisplayFrequency;
    // 0/1 = virtual displays; 480 caps stuck-driver garbage.
    if (2..=480).contains(&hz) {
        Some(hz)
    } else {
        None
    }
}

#[cfg(not(windows))]
pub(crate) fn primary_monitor_refresh_hz() -> Option<u32> {
    None
}
