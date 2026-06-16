#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app;
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

    let initial_path = match parse_cli_args(&mut std::env::args().skip(1)) {
        Ok(path) => path,
        Err(message) => {
            eprintln!("error: {message}");
            std::process::exit(1);
        }
    };

    iced::daemon(
        move || DesktopApp::boot(initial_path.clone()),
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

fn parse_cli_args(args: &mut impl Iterator<Item = String>) -> Result<Option<PathBuf>, String> {
    let Some(arg) = args.next() else {
        return Ok(None);
    };
    if arg.starts_with('-') {
        return Err(format!("unknown option: {arg}"));
    }
    if args.next().is_some() {
        return Err("too many arguments".to_owned());
    }
    Ok(Some(PathBuf::from(arg)))
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

#[cfg(test)]
mod tests {
    use super::parse_cli_args;
    use std::path::PathBuf;

    #[test]
    fn no_args_opens_empty_file() {
        assert!(matches!(parse_cli_args(&mut [].into_iter()), Ok(None)));
    }

    #[test]
    fn single_path_arg_opens_file() {
        assert_eq!(
            parse_cli_args(&mut ["snapshot.580".to_owned()].into_iter()).unwrap(),
            Some(PathBuf::from("snapshot.580"))
        );
    }
}
