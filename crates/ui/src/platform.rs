//! Platform helpers for native window presentation and pointer tracking.

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

    // SAFETY: HWND is live and the attribute points to a read-only POD `BOOL`.
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

    // SAFETY: HWND is live and the attribute points to a read-only POD `i32`.
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

#[cfg(windows)]
pub(crate) fn cursor_position_in_window(window: &dyn iced::window::Window) -> Option<iced::Point> {
    use iced::window::raw_window_handle::RawWindowHandle;
    use windows_sys::Win32::Foundation::{HWND, POINT};
    use windows_sys::Win32::Graphics::Gdi::ScreenToClient;
    use windows_sys::Win32::UI::HiDpi::GetDpiForWindow;
    use windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos;

    let handle = window.window_handle().ok()?;
    let RawWindowHandle::Win32(win32) = handle.as_raw() else {
        return None;
    };
    let hwnd = win32.hwnd.get() as HWND;
    let mut cursor = POINT { x: 0, y: 0 };

    // SAFETY: HWND comes from iced's live main window; Win32 writes only to `cursor`.
    let dpi = unsafe {
        if GetCursorPos(&mut cursor) == 0 {
            return None;
        }
        if ScreenToClient(hwnd, &mut cursor) == 0 {
            return None;
        }
        GetDpiForWindow(hwnd)
    };

    (dpi != 0).then(|| logical_cursor_position(cursor.x, cursor.y, dpi))
}

#[cfg(windows)]
fn logical_cursor_position(x: i32, y: i32, dpi: u32) -> iced::Point {
    let scale = dpi as f32 / 96.0;
    iced::Point::new(x as f32 / scale, y as f32 / scale)
}

#[cfg(not(windows))]
pub(crate) fn cursor_position_in_window(_window: &dyn iced::window::Window) -> Option<iced::Point> {
    None
}

pub(crate) const SUPPORTS_HIDDEN_WINDOW_REUSE: bool = cfg!(windows);

#[cfg(all(test, windows))]
mod tests {
    use super::logical_cursor_position;
    use iced::Point;

    #[test]
    fn converts_physical_cursor_coordinates_to_window_logical_coordinates() {
        assert_eq!(
            logical_cursor_position(300, 150, 144),
            Point::new(200.0, 100.0)
        );
    }
}
