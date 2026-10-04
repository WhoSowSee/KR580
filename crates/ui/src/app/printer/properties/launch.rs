use super::super::tasks::load_native_printer_properties_blocking;
use super::{DesktopApp, Message, PrinterPropertiesDialog};
use crate::backend::decode_oem_text;
use iced::Task;

impl DesktopApp {
    pub(in crate::app::printer) fn open_selected_printer_properties(&mut self) -> Task<Message> {
        let Some(dialog) = self.printer_setup.printer_setup_dialog.as_ref() else {
            return Task::none();
        };
        let Some(printer) = dialog.selected_printer().cloned() else {
            return Task::none();
        };
        let Some(settings) = dialog
            .configuration
            .as_ref()
            .map(|configuration| configuration.settings.clone())
        else {
            return Task::none();
        };
        let presets = self
            .preferences
            .stored
            .general
            .printer_presets
            .iter()
            .filter(|preset| preset.settings.printer_name == printer.name)
            .cloned()
            .collect();
        let preview_text = decode_oem_text(&self.snapshot.devices.printer.spool);
        let detached_surface = self.printer_setup_uses_detached_window();
        if let Some(dialog) = self.printer_setup.printer_setup_dialog.as_mut() {
            dialog.properties_pending = true;
            dialog.properties = Some(PrinterPropertiesDialog::new(preview_text, presets));
            dialog.properties_surface_ready = !detached_surface;
        }
        let printer_name = printer.name.clone();
        Task::batch([
            self.open_detached_printer_properties_window(),
            Task::perform(
                load_native_printer_properties_blocking(printer, settings),
                move |result| Message::PrinterPropertiesLoaded {
                    printer_name,
                    result,
                },
            ),
        ])
    }
}
