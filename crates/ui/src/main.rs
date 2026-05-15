mod app;
mod runtime;
mod view;

use app::DesktopApp;

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    iced::application(DesktopApp::new, DesktopApp::update, DesktopApp::view)
        .title("KR580 Emulator")
        .subscription(DesktopApp::subscription)
        .theme(DesktopApp::theme)
        .run()
}
