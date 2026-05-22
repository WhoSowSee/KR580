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
