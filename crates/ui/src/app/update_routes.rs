use iced::Task;

use super::{DesktopApp, Message};

impl DesktopApp {
    pub(crate) fn route_blocking_ui_message(&mut self, message: &Message) -> Option<Task<Message>> {
        if let Some(task) = self.dispatch_window_message(message) {
            return Some(task);
        }
        if let Some(task) = self.route_discard_modal_message(message) {
            return Some(task);
        }
        if let Some(task) = self.route_import_modal_message(message) {
            return Some(task);
        }
        if let Some(task) = self.route_export_modal_message(message) {
            return Some(task);
        }
        if let Some(task) = self.route_help_dialog_message(message) {
            return Some(task);
        }
        if let Some(task) = self.route_printer_setup_message(message) {
            return Some(task);
        }
        if let Some(task) = self.route_settings_modal_message(message) {
            return Some(task);
        }
        self.route_open_menu_message(message)
    }
}

pub(super) fn open_device_message(port: u8) -> Option<Message> {
    use crate::backend::IoBus;
    match port {
        IoBus::MONITOR_PORT => Some(Message::OpenMonitor),
        IoBus::FLOPPY_PORT => Some(Message::OpenFloppy),
        IoBus::HDD_PORT => Some(Message::OpenHdd),
        IoBus::NETWORK_PORT => Some(Message::OpenNetwork),
        IoBus::PRINTER_PORT => Some(Message::OpenPrinter),
        _ => None,
    }
}
