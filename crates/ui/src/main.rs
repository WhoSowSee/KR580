// Suppress the auxiliary console window on release builds for Windows. Debug
// builds intentionally keep the console so `tracing` output stays visible
// during development (`RUST_LOG=debug cargo run`). On non-Windows targets the
// attribute is a no-op anyway, but we gate it to keep the source obvious.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app;
mod platform;
mod runtime;
mod view;

use app::DesktopApp;
use iced::{Color, Size, Theme, theme, window};

/// Initial background painted by the OS before iced renders its first frame.
/// Matches `TOKYO_BOARD` in `view.rs` so the launch flash blends into the UI.
const BOARD_BACKGROUND: Color = Color::from_rgb(
    0x12 as f32 / 255.0,
    0x13 as f32 / 255.0,
    0x20 as f32 / 255.0,
);

/// Pre-rendered window icon. We embed the 64×64 PNG: it is the size Windows
/// actually consumes for the title bar and Alt+Tab thumbnail at typical DPI,
/// and the file is small enough that there is no point shipping multiple
/// resolutions. The other sizes in `assets/icons/` are kept for installers,
/// taskbar pinning, and future platform-specific bundling.
const ICON_PNG: &[u8] = include_bytes!("../../../assets/icons/icon-64.png");

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    iced::application(DesktopApp::new, DesktopApp::update, DesktopApp::view)
        .title("KR580 Emulator")
        .subscription(DesktopApp::subscription)
        .theme(DesktopApp::theme)
        .style(app_style)
        .window(window::Settings {
            size: Size::new(1180.0, 720.0),
            maximized: false,
            min_size: Some(Size::new(1180.0, 720.0)),
            icon: window::icon::from_file_data(ICON_PNG, None).ok(),
            // Start hidden so the OS does not flash a white frame before the
            // first iced paint. `DesktopApp::update` re-shows the window via
            // `window::set_mode(_, Windowed)` after the first `Tick`.
            visible: false,
            ..window::Settings::default()
        })
        .centered()
        .antialiasing(true)
        .run()
}

fn app_style(_state: &DesktopApp, _theme: &Theme) -> theme::Style {
    theme::Style {
        background_color: BOARD_BACKGROUND,
        text_color: Color::from_rgb(
            0xC0 as f32 / 255.0,
            0xCA as f32 / 255.0,
            0xF5 as f32 / 255.0,
        ),
    }
}
