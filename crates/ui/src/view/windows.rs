use iced::Element;
use iced::widget::Space;

use super::monitor::monitor_window;
use super::network::network_window;
use super::printer::printer_window;
use super::printer_setup::{printer_properties_window_view, printer_setup_window_view};
use super::storage::{floppy_window, hdd_window};
use super::theme;
use crate::app::{DesktopApp, Message};

impl DesktopApp {
    pub(crate) fn view(&self, window: iced::window::Id) -> Element<'_, Message> {
        theme::set_active_color_scheme(self.color_scheme);
        if self.printer_properties_window_id == Some(window) {
            return printer_properties_window_view(self.printer_setup_dialog.as_ref(), self.lang);
        }
        if self.printer_setup_window_id == Some(window) {
            return printer_setup_window_view(self.printer_setup_dialog.as_ref(), self.lang);
        }
        if self.monitor_window.id == Some(window) {
            if !self.monitor_window.detached {
                return Space::new().into();
            }
            return monitor_window(
                &self.snapshot.devices.monitor,
                self.monitor_split,
                self.monitor_hex_popup,
                self.monitor_hex_filter,
                self.monitor_hex_scroll_visible_ticks > 0,
                self.monitor_window.always_on_top,
                self.lang,
            );
        }
        if self.floppy_window.id == Some(window) {
            if !self.floppy_window.detached {
                return Space::new().into();
            }
            return floppy_window(
                &self.snapshot.devices.floppy,
                self.floppy_show_image_contents,
                &self.floppy_image_contents,
                self.floppy_image_error.as_deref(),
                self.floppy_window.always_on_top,
                self.lang,
            );
        }
        if self.hdd_window.id == Some(window) {
            if !self.hdd_window.detached {
                return Space::new().into();
            }
            return hdd_window(
                &self.snapshot.devices.hdd,
                self.hdd_file_exists,
                self.hdd_show_image_contents,
                &self.hdd_image_contents,
                self.hdd_image_error.as_deref(),
                self.hdd_window.always_on_top,
                self.lang,
            );
        }
        if self.network_window.id == Some(window) {
            if !self.network_window.detached {
                return Space::new().into();
            }
            return network_window(self.network_view_state(), self.network_window.always_on_top);
        }
        if self.printer_window.id == Some(window) {
            if !self.printer_window.detached {
                return Space::new().into();
            }
            return printer_window(
                &self.snapshot.devices.printer,
                self.printer_text_view,
                self.printer_target_label(),
                self.printer_window.always_on_top,
                self.lang,
            );
        }
        if self.main_window_id != Some(window) {
            return Space::new().into();
        }
        self.main_view()
    }
}
