mod keyboard;

use super::tasks::load_native_printer_configuration_blocking;
use super::{PrinterSetupDialog, PrinterSetupTarget};
use crate::app::{DesktopApp, Message};
use iced::Task;
use k580_ui::devices::printer::{PrinterConfiguration, PrinterInfo, PrinterSettings};

impl DesktopApp {
    pub(crate) fn route_printer_setup_message(
        &mut self,
        message: &Message,
    ) -> Option<Task<Message>> {
        self.printer_setup_dialog.as_ref()?;
        if self
            .printer_setup_dialog
            .as_ref()
            .is_some_and(|dialog| dialog.properties.is_some())
        {
            return self.route_printer_properties_message(message);
        }
        match message {
            Message::PrinterSetupLoaded(result) => {
                Some(self.finish_printer_setup_load(result.clone()))
            }
            Message::PrinterSetupSelected(name) => {
                if let Some(dialog) = self.printer_setup_dialog.as_mut() {
                    dialog.selected_name = Some(name.clone());
                    dialog.configuration = None;
                    dialog.error = None;
                    dialog.open_dropdown = None;
                    dialog.dropdown_highlight = None;
                }
                Some(self.load_selected_printer_configuration())
            }
            Message::PrinterSetupDropdownToggled(dropdown) => {
                self.toggle_printer_setup_dropdown(*dropdown);
                Some(Task::none())
            }
            Message::PrinterSetupDropdownDismissed(dropdown) => {
                if self
                    .printer_setup_dialog
                    .as_ref()
                    .is_some_and(|dialog| dialog.open_dropdown == Some(*dropdown))
                {
                    self.close_printer_setup_dropdown();
                }
                Some(Task::none())
            }
            Message::PrinterSetupConfigurationLoaded {
                printer_name,
                result,
            } => {
                self.finish_printer_configuration_load(printer_name, result.clone());
                Some(Task::none())
            }
            Message::PrinterSetupPaperSelected(id) => {
                self.close_printer_setup_dropdown();
                self.select_printer_paper(*id);
                Some(Task::none())
            }
            Message::PrinterSetupSourceSelected(id) => {
                self.close_printer_setup_dropdown();
                self.select_printer_source(*id);
                Some(Task::none())
            }
            Message::PrinterSetupOrientationSelected(orientation) => {
                self.close_printer_setup_dropdown();
                if let Some(settings) = self
                    .printer_setup_dialog
                    .as_mut()
                    .and_then(|dialog| dialog.configuration.as_mut())
                    .map(|configuration| &mut configuration.settings)
                {
                    settings.orientation = *orientation;
                }
                Some(Task::none())
            }
            Message::PrinterSetupProperties => Some(self.open_selected_printer_properties()),
            Message::PrinterSetupConfirmed => {
                self.confirm_printer_setup_dialog();
                Some(Task::none())
            }
            Message::ClosePrinterSetup => {
                self.close_printer_setup_dialog();
                Some(Task::none())
            }
            Message::EscPressed => {
                if self
                    .printer_setup_dialog
                    .as_ref()
                    .is_some_and(|dialog| dialog.open_dropdown.is_some())
                {
                    self.close_printer_setup_dropdown();
                } else {
                    self.close_printer_setup_dialog();
                }
                Some(Task::none())
            }
            Message::EnterPressed => Some(self.activate_printer_setup_focus()),
            Message::FocusCycle { backward } => {
                self.cycle_printer_setup_focus(*backward);
                Some(Task::none())
            }
            Message::ArrowKey(direction) => {
                self.move_printer_setup_dropdown_highlight(*direction);
                Some(Task::none())
            }
            Message::MousePressed | Message::MousePressedIgnored => {
                if let Some(dialog) = self.printer_setup_dialog.as_mut() {
                    dialog.focus_visible = false;
                }
                if matches!(message, Message::MousePressedIgnored) {
                    self.close_printer_setup_dropdown();
                }
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

    fn finish_printer_setup_load(
        &mut self,
        result: Result<Vec<PrinterInfo>, String>,
    ) -> Task<Message> {
        let Some(dialog) = self.printer_setup_dialog.as_mut() else {
            return Task::none();
        };
        dialog.loading = false;
        match result {
            Ok(printers) => {
                dialog.error = if printers.is_empty() {
                    Some("No printers found".to_owned())
                } else {
                    None
                };
                dialog.printers = printers;
                if !selected_printer_exists(dialog) {
                    dialog.selected_name = default_printer_name(dialog)
                        .or_else(|| dialog.printers.first().map(|printer| printer.name.clone()));
                }
            }
            Err(error) => {
                dialog.printers.clear();
                dialog.selected_name = None;
                dialog.error = Some(error);
            }
        }
        self.load_selected_printer_configuration()
    }

    fn load_selected_printer_configuration(&mut self) -> Task<Message> {
        let Some(dialog) = self.printer_setup_dialog.as_mut() else {
            return Task::none();
        };
        let Some(printer) = dialog.selected_printer().cloned() else {
            return Task::none();
        };
        dialog.configuration_loading = true;
        let printer_name = printer.name.clone();
        Task::perform(
            load_native_printer_configuration_blocking(printer),
            move |result| Message::PrinterSetupConfigurationLoaded {
                printer_name,
                result,
            },
        )
    }

    fn finish_printer_configuration_load(
        &mut self,
        printer_name: &str,
        result: Result<PrinterConfiguration, String>,
    ) {
        let preferred = self.preferred_printer_settings(printer_name);
        let Some(dialog) = self.printer_setup_dialog.as_mut() else {
            return;
        };
        if dialog.selected_name.as_deref() != Some(printer_name) {
            return;
        }
        dialog.configuration_loading = false;
        match result {
            Ok(mut configuration) => {
                if let Some(settings) = preferred {
                    merge_preferred_settings(&mut configuration, settings);
                }
                dialog.configuration = Some(configuration);
                dialog.error = None;
            }
            Err(error) => {
                dialog.configuration = None;
                dialog.error = Some(error);
            }
        }
    }

    fn preferred_printer_settings(&self, printer_name: &str) -> Option<PrinterSettings> {
        let target = self.printer_setup_dialog.as_ref()?.target;
        match target {
            PrinterSetupTarget::Session => self.active_printer_settings(),
            PrinterSetupTarget::Settings => self
                .settings_dialog
                .as_ref()
                .and_then(|dialog| dialog.draft_printer_settings.as_ref()),
        }
        .filter(|settings| settings.printer_name == printer_name)
        .cloned()
    }

    fn select_printer_paper(&mut self, id: i16) {
        let Some(configuration) = self
            .printer_setup_dialog
            .as_mut()
            .and_then(|dialog| dialog.configuration.as_mut())
        else {
            return;
        };
        configuration.select_paper(id);
    }

    fn select_printer_source(&mut self, id: i16) {
        let Some(configuration) = self
            .printer_setup_dialog
            .as_mut()
            .and_then(|dialog| dialog.configuration.as_mut())
        else {
            return;
        };
        configuration.select_source(id);
    }

    fn confirm_printer_setup_dialog(&mut self) {
        let Some(dialog) = self.printer_setup_dialog.take() else {
            return;
        };
        self.printer_setup_pending = false;
        let Some(name) = dialog.selected_name else {
            return;
        };
        let settings = dialog
            .configuration
            .map(|configuration| configuration.settings)
            .unwrap_or_else(|| PrinterSettings::named(name.clone()));
        match dialog.target {
            PrinterSetupTarget::Session => self.printer_session_settings = Some(settings),
            PrinterSetupTarget::Settings => {
                if let Some(settings_dialog) = self.settings_dialog.as_mut() {
                    settings_dialog.draft_printer_settings = Some(settings);
                }
            }
        }
    }

    fn close_printer_setup_dialog(&mut self) {
        self.printer_setup_dialog = None;
        self.printer_setup_pending = false;
    }
}

fn selected_printer_exists(dialog: &PrinterSetupDialog) -> bool {
    dialog.selected_name.as_deref().is_some_and(|selected| {
        dialog
            .printers
            .iter()
            .any(|printer| printer.name == selected)
    })
}

fn default_printer_name(dialog: &PrinterSetupDialog) -> Option<String> {
    dialog
        .printers
        .iter()
        .find(|printer| printer.is_default)
        .map(|printer| printer.name.clone())
}

fn merge_preferred_settings(
    configuration: &mut PrinterConfiguration,
    mut preferred: PrinterSettings,
) {
    if preferred.devmode.is_empty() {
        return;
    }
    if !preferred
        .paper_id
        .is_some_and(|id| configuration.papers.iter().any(|paper| paper.id == id))
    {
        preferred.paper_id = configuration.settings.paper_id;
        preferred.paper_name = configuration.settings.paper_name.clone();
    }
    if !preferred
        .source_id
        .is_some_and(|id| configuration.sources.iter().any(|source| source.id == id))
    {
        preferred.source_id = configuration.settings.source_id;
        preferred.source_name = configuration.settings.source_name.clone();
    }
    configuration.settings = preferred;
}
