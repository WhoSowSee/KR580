//! Platform-specific helpers for taming the OS-level launch flash.
//!
//! On Windows the desktop window manager paints the client area with the
//! white system brush between window creation and the first GPU-presented
//! frame. To suppress that flash we ask DWM to cloak the window (keeping it
//! laid out and rendered, just invisible to the compositor) and uncloak it
//! once iced has produced its first frame.

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

#[cfg(not(windows))]
pub(crate) fn cloak_window(_window: &dyn iced::window::Window, _cloaked: bool) {}
