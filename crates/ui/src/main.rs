#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app;
mod platform;
mod runtime;
mod view;

use app::DesktopApp;
use iced::{Color, Size, Theme, theme, window};
use std::path::PathBuf;

/// Matches `TOKYO_BOARD` so the launch flash blends into the UI.
const BOARD_BACKGROUND: Color = Color::from_rgb(
    0x12 as f32 / 255.0,
    0x13 as f32 / 255.0,
    0x20 as f32 / 255.0,
);

const ICON_PNG: &[u8] = include_bytes!("../../../assets/icons/icon-64.png");

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    // OS-launched `k580 path/to/snap.580` arrives as `argv[1]`.
    // Filter to existing files so a stale shortcut leaves us empty.
    let initial_snapshot_path: Option<PathBuf> = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .filter(|path| path.is_file());
    iced::application(
        move || DesktopApp::with_initial_path(initial_snapshot_path.clone()),
        DesktopApp::update,
        DesktopApp::view,
    )
    .title("KR580 Emulator")
    .subscription(DesktopApp::subscription)
    .theme(DesktopApp::theme)
    .style(app_style)
    .window(window::Settings {
        size: Size::new(1180.0, 720.0),
        maximized: false,
        min_size: Some(Size::new(1180.0, 720.0)),
        icon: window::icon::from_file_data(ICON_PNG, None).ok(),
        decorations: false,
        // Hidden until the first iced paint; uncloaked from `update`.
        visible: false,
        ..window::Settings::default()
    })
    .centered()
    .antialiasing(true)
    // Route OS close requests through the dirty gate.
    .exit_on_close_request(false)
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
