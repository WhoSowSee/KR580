mod focus;
mod keyboard;
mod presets;
mod validation;

use super::PrinterPropertiesDialog;
use super::tasks::{
    apply_native_printer_property_blocking, load_native_printer_properties_blocking,
};
use crate::app::{DesktopApp, Message};
use crate::backend::decode_oem_text;
use crate::settings_storage::load_settings;
use iced::Task;
use k580_ui::devices::printer::{PrinterPropertyChange, PrinterPropertySheet};

impl DesktopApp {
    pub(super) fn open_selected_printer_properties(&mut self) -> Task<Message> {
        let Some(dialog) = self.printer_setup_dialog.as_ref() else {
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
        let presets = load_settings()
            .general
            .printer_presets
            .into_iter()
            .filter(|preset| preset.settings.printer_name == printer.name)
            .collect();
        let preview_text = decode_oem_text(&self.snapshot.devices.printer.spool);
        if let Some(dialog) = self.printer_setup_dialog.as_mut() {
            dialog.properties_pending = true;
            dialog.properties = Some(PrinterPropertiesDialog::new(preview_text, presets));
        }
        let printer_name = printer.name.clone();
        Task::perform(
            load_native_printer_properties_blocking(printer, settings),
            move |result| Message::PrinterPropertiesLoaded {
                printer_name,
                result,
            },
        )
    }

    pub(super) fn route_printer_properties_message(
        &mut self,
        message: &Message,
    ) -> Option<Task<Message>> {
        match message {
            Message::PrinterPropertiesLoaded {
                printer_name,
                result,
            }
            | Message::PrinterPropertyApplied {
                printer_name,
                result,
            } => {
                self.finish_property_sheet_load(printer_name, result.clone());
                Some(Task::none())
            }
            Message::PrinterPropertiesTabSelected(tab) => {
                if let Some(properties) = self.properties_mut() {
                    properties.tab = *tab;
                    properties.focus = super::PrinterPropertiesFocus::Tab(*tab);
                    properties.tab_focus_visible = false;
                    properties.open_dropdown = None;
                    properties.dropdown_highlight = None;
                }
                Some(Task::none())
            }
            Message::PrinterPropertiesFocusResolved { focused, backward } => {
                Some(self.resolve_printer_properties_focus(focused.clone(), *backward))
            }
            Message::PrinterPropertyDropdownToggled(dropdown) => {
                if let Some(properties) = self.properties_mut() {
                    properties.focus = super::PrinterPropertiesFocus::Dropdown(dropdown.clone());
                }
                self.toggle_property_dropdown(dropdown.clone());
                Some(Task::none())
            }
            Message::PrinterPropertyDropdownDismissed(dropdown) => {
                if self
                    .properties()
                    .is_some_and(|properties| properties.open_dropdown.as_ref() == Some(dropdown))
                {
                    self.close_property_dropdown();
                }
                Some(Task::none())
            }
            Message::PrinterPropertyFeatureSelected(change) => {
                self.close_property_dropdown();
                Some(self.apply_printer_property(change.clone()))
            }
            Message::PrinterPropertyParameterChanged { name, value } => {
                if let Some(properties) = self.properties_mut() {
                    properties.focus = super::PrinterPropertiesFocus::ParameterInput(name.clone());
                    properties
                        .parameter_values
                        .insert(name.clone(), value.clone());
                }
                Some(Task::none())
            }
            Message::PrinterPropertyParameterApply(name) => {
                if let Some(properties) = self.properties_mut() {
                    properties.focus = super::PrinterPropertiesFocus::ParameterApply(name.clone());
                }
                Some(self.apply_printer_parameter(name))
            }
            Message::PrinterPropertyPaperSelected(id) => {
                self.close_property_dropdown();
                self.select_property_paper(*id);
                Some(Task::none())
            }
            Message::PrinterPropertySourceSelected(id) => {
                self.close_property_dropdown();
                self.select_property_source(*id);
                Some(Task::none())
            }
            Message::PrinterPropertyOrientationSelected(orientation) => {
                self.close_property_dropdown();
                if let Some(properties) = self.properties_mut() {
                    properties.focus = match orientation {
                        k580_ui::devices::printer::PrinterOrientation::Portrait => {
                            super::PrinterPropertiesFocus::Portrait
                        }
                        k580_ui::devices::printer::PrinterOrientation::Landscape => {
                            super::PrinterPropertiesFocus::Landscape
                        }
                    };
                }
                if let Some(settings) = self.property_settings_mut() {
                    settings.orientation = *orientation;
                }
                Some(Task::none())
            }
            Message::PrinterPropertyPresetSelected(name) => {
                self.close_property_dropdown();
                Some(self.select_printer_preset(name))
            }
            Message::PrinterPropertyPresetNameChanged(name) => {
                if let Some(properties) = self.properties_mut() {
                    properties.focus = super::PrinterPropertiesFocus::PresetName;
                    properties.preset_name = name.clone();
                }
                Some(Task::none())
            }
            Message::PrinterPropertyPresetSave => {
                self.save_printer_preset();
                Some(Task::none())
            }
            Message::PrinterPropertyPresetDelete => {
                self.delete_printer_preset();
                Some(Task::none())
            }
            Message::PrinterPropertyConfirmed => {
                self.confirm_printer_properties();
                Some(Task::none())
            }
            Message::ClosePrinterProperties => {
                self.close_printer_properties();
                Some(Task::none())
            }
            Message::EscPressed => {
                if self
                    .properties()
                    .is_some_and(|properties| properties.open_dropdown.is_some())
                {
                    self.close_property_dropdown();
                } else {
                    self.close_printer_properties();
                }
                Some(Task::none())
            }
            Message::EnterPressed => Some(self.activate_printer_properties_focus()),
            Message::FocusCycle { backward } => {
                Some(self.begin_printer_properties_focus_cycle(*backward))
            }
            Message::ArrowKey(direction) => {
                self.move_property_dropdown_highlight(*direction);
                Some(Task::none())
            }
            Message::MousePressedIgnored => {
                self.close_property_dropdown();
                Some(Task::none())
            }
            Message::Tick
            | Message::CursorMoved(_)
            | Message::ModifiersChanged(_)
            | Message::FocusReconciled { .. }
            | Message::ResolveFocusedTracker(_) => None,
            _ => Some(Task::none()),
        }
    }

    fn finish_property_sheet_load(
        &mut self,
        printer_name: &str,
        result: Result<PrinterPropertySheet, String>,
    ) {
        if self
            .printer_setup_dialog
            .as_ref()
            .and_then(|dialog| dialog.selected_name.as_deref())
            != Some(printer_name)
        {
            return;
        }
        let Some(properties) = self.properties_mut() else {
            return;
        };
        properties.loading = false;
        properties.applying = false;
        match result {
            Ok(sheet) => {
                properties.sheet = Some(sheet);
                properties.error = None;
                properties.sync_parameter_values();
            }
            Err(error) => properties.error = Some(error),
        }
    }

    fn apply_printer_property(&mut self, change: PrinterPropertyChange) -> Task<Message> {
        let Some((printer, settings)) = self.property_context() else {
            return Task::none();
        };
        if let Some(properties) = self.properties_mut() {
            if properties.applying {
                return Task::none();
            }
            properties.applying = true;
            properties.error = None;
        }
        let printer_name = printer.name.clone();
        Task::perform(
            apply_native_printer_property_blocking(printer, settings, change),
            move |result| Message::PrinterPropertyApplied {
                printer_name,
                result,
            },
        )
    }

    fn apply_printer_parameter(&mut self, name: &str) -> Task<Message> {
        let lang = self.lang;
        let Some(properties) = self.properties() else {
            return Task::none();
        };
        let Some(sheet) = properties.sheet.as_ref() else {
            return Task::none();
        };
        let Some(parameter) = sheet
            .parameters
            .iter()
            .find(|parameter| parameter.name == name)
        else {
            return Task::none();
        };
        let value = properties
            .parameter_values
            .get(name)
            .cloned()
            .unwrap_or_else(|| parameter.value.clone());
        if let Err(error) =
            validation::validate_parameter(parameter.minimum, parameter.maximum, &value, lang)
        {
            if let Some(properties) = self.properties_mut() {
                properties.error = Some(error);
            }
            return Task::none();
        }
        self.apply_printer_property(PrinterPropertyChange::Parameter {
            parameter_name: parameter.name.clone(),
            value_type: parameter.value_type.clone(),
            value,
        })
    }

    fn select_property_paper(&mut self, id: i16) {
        let Some(sheet) = self
            .properties_mut()
            .and_then(|dialog| dialog.sheet.as_mut())
        else {
            return;
        };
        sheet.configuration.select_paper(id);
    }

    fn select_property_source(&mut self, id: i16) {
        let Some(sheet) = self
            .properties_mut()
            .and_then(|dialog| dialog.sheet.as_mut())
        else {
            return;
        };
        sheet.configuration.select_source(id);
    }

    fn confirm_printer_properties(&mut self) {
        let configuration = self
            .properties()
            .and_then(|properties| properties.sheet.as_ref())
            .map(|sheet| sheet.configuration.clone());
        if let Some(dialog) = self.printer_setup_dialog.as_mut() {
            if let Some(configuration) = configuration {
                dialog.configuration = Some(configuration);
            }
            dialog.properties = None;
            dialog.properties_pending = false;
        }
    }

    fn close_printer_properties(&mut self) {
        if let Some(dialog) = self.printer_setup_dialog.as_mut() {
            dialog.properties = None;
            dialog.properties_pending = false;
        }
    }

    fn property_context(
        &self,
    ) -> Option<(
        k580_ui::devices::printer::PrinterInfo,
        k580_ui::devices::printer::PrinterSettings,
    )> {
        let dialog = self.printer_setup_dialog.as_ref()?;
        let printer = dialog.selected_printer()?.clone();
        let settings = dialog
            .properties
            .as_ref()?
            .sheet
            .as_ref()?
            .configuration
            .settings
            .clone();
        Some((printer, settings))
    }

    fn properties(&self) -> Option<&PrinterPropertiesDialog> {
        self.printer_setup_dialog.as_ref()?.properties.as_ref()
    }

    fn properties_mut(&mut self) -> Option<&mut PrinterPropertiesDialog> {
        self.printer_setup_dialog.as_mut()?.properties.as_mut()
    }

    fn close_property_dropdown(&mut self) {
        if let Some(properties) = self.properties_mut() {
            properties.open_dropdown = None;
            properties.dropdown_highlight = None;
        }
    }

    fn property_settings_mut(&mut self) -> Option<&mut k580_ui::devices::printer::PrinterSettings> {
        Some(
            &mut self
                .properties_mut()?
                .sheet
                .as_mut()?
                .configuration
                .settings,
        )
    }
}
