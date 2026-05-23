//! Platform-specific helpers for taming the OS-level launch flash.
//!
//! On Windows the desktop window manager paints the client area with the
//! white system brush between window creation and the first GPU-presented
//! frame. To suppress that flash we ask DWM to cloak the window (keeping it
//! laid out and rendered, just invisible to the compositor) and uncloak it
//! once iced has produced its first frame.
//!
//! The same module also opts the borderless window into Windows 11's DWM
//! rounded-corner treatment. Without it a `decorations: false` window has
//! sharp 90° corners on every Windows version, because winit only draws
//! the client area and there is no native frame around it. On Windows 11
//! the DWM happily clips and shadows our window for us; on Windows 10 the
//! attribute call is a documented no-op.

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

    // SAFETY: HWND originates from winit's live window and the attribute is a
    // POD `BOOL`. Failure (e.g. older Windows without DWM cloak support) is
    // ignored because the worst case is the original launch flash.
    unsafe {
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_CLOAK as u32,
            std::ptr::from_ref(&value).cast(),
            std::mem::size_of::<BOOL>() as u32,
        );
    }
}

/// Asks DWM to render rounded corners on our borderless window. Without
/// the call winit hands us a 90° client area, so a `decorations: false`
/// window looks unfinished — the user reported "у окна нет скруглений".
/// `DWMWCP_ROUND` is the same preference Windows 11's native chrome uses
/// for everything that is not a tooltip / context menu, so the radius
/// matches the visual rhythm of the rest of the OS. On Windows 10 the
/// attribute is unknown and `DwmSetWindowAttribute` returns an error,
/// which we deliberately swallow — the window stays sharp-cornered there
/// (matching the OS convention) instead of triggering a runtime crash.
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

    // SAFETY: HWND comes from winit's live window, the attribute payload
    // is a POD `i32`, and `DwmSetWindowAttribute` returns `S_OK` on
    // success / a documented HRESULT we don't need to react to on
    // failure (older Windows). The pointer is read-only for the
    // duration of the call.
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

/// Best-effort guess at the primary display's refresh rate, used as the
/// "Высоко" tier on the speed switch. The "High" tier promises "as fast
/// as the screen can paint", and a 60 Hz fallback is the floor every
/// modern desktop guarantees — Windows 7 onward, every macOS, and the
/// vast majority of Linux compositors. Numbers above 60 (120 / 144 /
/// 240 …) only matter when we can confirm them; without confirmation
/// we'd rather under-promise than waste CPU on UI ticks the panel
/// can't actually display.
///
/// Returns `Some(refresh_hz)` when the platform query reports a
/// believable value (1…480 Hz; iced/winit don't expose this directly
/// in 0.14, so we go straight to the OS), or `None` if the query was
/// unavailable. Callers should fall back to 60 Hz on `None`.
#[cfg(windows)]
pub(crate) fn primary_monitor_refresh_hz() -> Option<u32> {
    use windows_sys::Win32::Graphics::Gdi::{
        DEVMODEW, ENUM_CURRENT_SETTINGS, EnumDisplaySettingsW,
    };

    // SAFETY: `DEVMODEW` is a POD struct that the Win32 API fills in
    // for us; zero-initialising it is the documented way to ask for
    // current settings (the API only requires `dmSize` to be set
    // correctly, which we do below). Passing a null pointer for the
    // device name asks for the primary display, which is the one we
    // care about for "what refresh rate is the user looking at".
    let mut mode: DEVMODEW = unsafe { std::mem::zeroed() };
    mode.dmSize = std::mem::size_of::<DEVMODEW>() as u16;
    let ok = unsafe { EnumDisplaySettingsW(std::ptr::null(), ENUM_CURRENT_SETTINGS, &mut mode) };
    if ok == 0 {
        return None;
    }
    let hz = mode.dmDisplayFrequency;
    // Windows reports 0 or 1 for "default / unknown" on virtual /
    // headless displays — neither is a refresh rate, both should fall
    // through to the 60 Hz default. Cap at 480 to swat any obviously
    // garbage value (the field is a `u32`, so a stuck driver could in
    // principle return something nonsensical).
    if (2..=480).contains(&hz) {
        Some(hz)
    } else {
        None
    }
}

#[cfg(not(windows))]
pub(crate) fn primary_monitor_refresh_hz() -> Option<u32> {
    // No platform-specific path implemented yet — callers fall back to
    // 60 Hz, which is correct on every desktop monitor manufactured in
    // the last twenty years and any virtual display that defaults to
    // the VESA baseline. A future macOS / X11 / Wayland implementation
    // can plug into this signature without churning callers.
    None
}
