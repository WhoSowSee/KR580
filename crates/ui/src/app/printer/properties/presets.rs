use super::super::tasks::load_native_printer_properties_blocking;
use crate::app::{DesktopApp, Message};
use crate::i18n::Lang;
use crate::persistence::PrinterPreset;
use crate::settings_storage::{load_settings, save_settings};
use iced::Task;

impl DesktopApp {
    pub(super) fn select_printer_preset(&mut self, name: &str) -> Task<Message> {
        let Some(properties) = self.properties() else {
            return Task::none();
        };
        let Some(preset) = properties
            .presets
            .iter()
            .find(|preset| preset.name == name)
            .cloned()
        else {
            return Task::none();
        };
        let Some(printer) = self
            .printer_setup_dialog
            .as_ref()
            .and_then(|dialog| dialog.selected_printer())
            .cloned()
        else {
            return Task::none();
        };
        if let Some(properties) = self.properties_mut() {
            properties.selected_preset = Some(name.to_owned());
            properties.preset_name = name.to_owned();
            properties.applying = true;
        }
        let printer_name = printer.name.clone();
        Task::perform(
            load_native_printer_properties_blocking(printer, preset.settings),
            move |result| Message::PrinterPropertyApplied {
                printer_name,
                result,
            },
        )
    }

    pub(super) fn save_printer_preset(&mut self) {
        let Some(properties) = self.properties() else {
            return;
        };
        let name = properties.preset_name.trim().to_owned();
        let Some(settings) = properties
            .sheet
            .as_ref()
            .map(|sheet| sheet.configuration.settings.clone())
        else {
            return;
        };
        if name.is_empty() {
            let error = match self.lang {
                Lang::Ru => "Введите название профиля".to_owned(),
                Lang::En => "Enter a profile name".to_owned(),
            };
            if let Some(properties) = self.properties_mut() {
                properties.error = Some(error);
            }
            return;
        }
        let mut stored = load_settings();
        stored.general.printer_presets.retain(|preset| {
            preset.name != name || preset.settings.printer_name != settings.printer_name
        });
        stored.general.printer_presets.push(PrinterPreset {
            name: name.clone(),
            settings,
        });
        save_settings(&stored);
        self.reload_property_presets(Some(name));
    }

    pub(super) fn delete_printer_preset(&mut self) {
        let Some(properties) = self.properties() else {
            return;
        };
        let Some(name) = properties.selected_preset.clone() else {
            return;
        };
        let Some(printer_name) = properties
            .sheet
            .as_ref()
            .map(|sheet| sheet.configuration.settings.printer_name.clone())
        else {
            return;
        };
        let mut stored = load_settings();
        stored
            .general
            .printer_presets
            .retain(|preset| preset.name != name || preset.settings.printer_name != printer_name);
        save_settings(&stored);
        self.reload_property_presets(None);
    }

    fn reload_property_presets(&mut self, selected: Option<String>) {
        let Some(printer_name) = self
            .properties()
            .and_then(|properties| properties.sheet.as_ref())
            .map(|sheet| sheet.configuration.settings.printer_name.clone())
        else {
            return;
        };
        let presets = load_settings()
            .general
            .printer_presets
            .into_iter()
            .filter(|preset| preset.settings.printer_name == printer_name)
            .collect();
        if let Some(properties) = self.properties_mut() {
            properties.presets = presets;
            properties.selected_preset = selected.clone();
            properties.preset_name = selected.unwrap_or_default();
            properties.error = None;
        }
    }
}
