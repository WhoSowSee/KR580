#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app;
mod file_assoc;
mod i18n;
mod platform;
mod runtime;
mod settings_storage;
mod view;

use app::DesktopApp;
use iced::{Color, Theme, theme};
use std::path::PathBuf;

/// Matches `TOKYO_BOARD` so the launch flash blends into the UI.
const BOARD_BACKGROUND: Color = Color::from_rgb(
    0x12 as f32 / 255.0,
    0x13 as f32 / 255.0,
    0x20 as f32 / 255.0,
);

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let mut args = std::env::args().skip(1);
    let initial_arg = args.next();

    if let Some(arg) = initial_arg.as_deref() {
        match arg {
            "--register-file-type" => {
                return run_assoc(file_assoc::register, "Ассоциация .580 зарегистрирована");
            }
            "--unregister-file-type" => {
                return run_assoc(file_assoc::unregister, "Ассоциация .580 удалена");
            }
            _ => {}
        }
    }

    let initial_snapshot_path: Option<PathBuf> =
        initial_arg.map(PathBuf::from).filter(|path| path.is_file());
    iced::daemon(
        move || DesktopApp::boot(initial_snapshot_path.clone()),
        DesktopApp::update,
        DesktopApp::view,
    )
    .title(DesktopApp::title)
    .subscription(DesktopApp::subscription)
    .theme(DesktopApp::theme)
    .style(app_style)
    .antialiasing(true)
    .run()
}

fn run_assoc(action: fn() -> Result<(), String>, success: &str) -> iced::Result {
    match action() {
        Ok(()) => {
            println!("{success}");
            Ok(())
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
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
