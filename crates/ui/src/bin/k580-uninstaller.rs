#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod installer;

fn main() -> iced::Result {
    installer::entry::run()
}
