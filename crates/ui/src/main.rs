#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app;
mod i18n;
use k580_ui::{backend, persistence};
mod platform;
mod runtime;
mod settings_storage;
mod system_locale;
mod view;

use app::DesktopApp;
use iced::{Theme, theme};
use std::path::PathBuf;

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

fn app_style(state: &DesktopApp, _theme: &Theme) -> theme::Style {
    view::theme::app_base_style(state.color_scheme)
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
