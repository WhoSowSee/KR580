mod dialog;
mod properties;
mod tasks;
#[cfg(test)]
mod tests;
mod window;

use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::DesktopApp;
use crate::i18n::{Key, PrinterKey};
use crate::persistence::PrinterDialogMode;
use crate::persistence::PrinterPreset;
use iced::Task;
use k580_ui::devices::printer::{
    PrinterConfiguration, PrinterInfo, PrinterPropertySheet, PrinterSettings,
};
use tasks::{configure_native_printer_blocking, list_native_printers_blocking};

const PRINTER_PROPERTIES_ATTENTION_DURATION: Duration = Duration::from_millis(520);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PrinterSetupTarget {
    Session,
    Settings,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum PrinterPropertiesTab {
    #[default]
    Favorites,
    General,
    Paper,
    Graphics,
    Advanced,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PrinterPropertyDropdown {
    Feature(String),
    Paper,
    Source,
    Preset,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PrinterPropertiesFocus {
    Close,
    Tab(PrinterPropertiesTab),
    Dropdown(PrinterPropertyDropdown),
    Portrait,
    Landscape,
    ParameterInput(String),
    ParameterApply(String),
    PresetName,
    PresetDelete,
    PresetSave,
    Cancel,
    Ok,
}

impl Default for PrinterPropertiesFocus {
    fn default() -> Self {
        Self::Tab(PrinterPropertiesTab::Favorites)
    }
}

pub(crate) const PRINTER_PROPERTIES_PRESET_INPUT_ID: &str = "printer-properties-preset-name-input";

pub(crate) fn printer_property_parameter_input_id(name: &str) -> String {
    format!("printer-properties-parameter-input:{name}")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PrinterSetupDropdown {
    Printer,
    Paper,
    Source,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum PrinterSetupFocus {
    Close,
    #[default]
    Printer,
    Properties,
    Paper,
    Source,
    Portrait,
    Landscape,
    Cancel,
    Ok,
}

impl PrinterSetupFocus {
    pub(crate) const ALL: [Self; 9] = [
        Self::Close,
        Self::Printer,
        Self::Properties,
        Self::Paper,
        Self::Source,
        Self::Portrait,
        Self::Landscape,
        Self::Cancel,
        Self::Ok,
    ];
}

#[derive(Clone, Debug)]
pub(crate) struct PrinterPropertiesDialog {
    pub(crate) sheet: Option<PrinterPropertySheet>,
    pub(crate) tab: PrinterPropertiesTab,
    pub(crate) loading: bool,
    pub(crate) applying: bool,
    pub(crate) parameter_values: HashMap<String, String>,
    pub(crate) presets: Vec<PrinterPreset>,
    pub(crate) selected_preset: Option<String>,
    pub(crate) preset_name: String,
    pub(crate) open_dropdown: Option<PrinterPropertyDropdown>,
    pub(crate) dropdown_highlight: Option<usize>,
    pub(crate) focus: PrinterPropertiesFocus,
    pub(crate) focus_visible: bool,
    pub(crate) preview_text: String,
    pub(crate) error: Option<String>,
    attention_started_at: Option<Instant>,
}

impl PrinterPropertiesDialog {
    fn new(preview_text: String, presets: Vec<PrinterPreset>) -> Self {
        Self {
            sheet: None,
            tab: PrinterPropertiesTab::default(),
            loading: true,
            applying: false,
            parameter_values: HashMap::new(),
            presets,
            selected_preset: None,
            preset_name: String::new(),
            open_dropdown: None,
            dropdown_highlight: None,
            focus: PrinterPropertiesFocus::default(),
            focus_visible: false,
            preview_text,
            error: None,
            attention_started_at: None,
        }
    }

    fn restart_attention(&mut self, now: Instant) {
        self.attention_started_at = Some(now);
    }

    fn expire_attention(&mut self, now: Instant) {
        if self.attention_started_at.is_some_and(|started_at| {
            now.saturating_duration_since(started_at) >= PRINTER_PROPERTIES_ATTENTION_DURATION
        }) {
            self.attention_started_at = None;
        }
    }

    pub(crate) fn attention_strength(&self, now: Instant) -> f32 {
        let Some(started_at) = self.attention_started_at else {
            return 0.0;
        };
        let progress = (now.saturating_duration_since(started_at).as_secs_f32()
            / PRINTER_PROPERTIES_ATTENTION_DURATION.as_secs_f32())
        .clamp(0.0, 1.0);
        (std::f32::consts::PI * progress).sin()
    }

    pub(crate) fn sync_parameter_values(&mut self) {
        let Some(sheet) = self.sheet.as_ref() else {
            return;
        };
        self.parameter_values = sheet
            .parameters
            .iter()
            .map(|parameter| (parameter.name.clone(), parameter.value.clone()))
            .collect();
    }

    pub(crate) fn focus_is_visible(&self, focus: PrinterPropertiesFocus) -> bool {
        self.focus_visible && self.focus == focus
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PrinterSetupDialog {
    pub(crate) target: PrinterSetupTarget,
    pub(crate) owner_ready: bool,
    pub(crate) owner_position: Option<iced::Point>,
    pub(crate) properties_surface_ready: bool,
    pub(crate) printers: Vec<PrinterInfo>,
    pub(crate) selected_name: Option<String>,
    pub(crate) configuration: Option<PrinterConfiguration>,
    pub(crate) loading: bool,
    pub(crate) configuration_loading: bool,
    pub(crate) properties_pending: bool,
    pub(crate) properties: Option<PrinterPropertiesDialog>,
    pub(crate) focus: PrinterSetupFocus,
    pub(crate) focus_visible: bool,
    pub(crate) open_dropdown: Option<PrinterSetupDropdown>,
    pub(crate) dropdown_highlight: Option<usize>,
    pub(crate) error: Option<String>,
}

impl PrinterSetupDialog {
    fn new(target: PrinterSetupTarget, selected_name: Option<String>) -> Self {
        Self {
            target,
            owner_ready: true,
            owner_position: None,
            properties_surface_ready: true,
            printers: Vec::new(),
            selected_name,
            configuration: None,
            loading: true,
            configuration_loading: false,
            properties_pending: false,
            properties: None,
            focus: PrinterSetupFocus::default(),
            focus_visible: false,
            open_dropdown: None,
            dropdown_highlight: None,
            error: None,
        }
    }

    pub(crate) fn selected_printer(&self) -> Option<&PrinterInfo> {
        let selected = self.selected_name.as_deref()?;
        self.printers
            .iter()
            .find(|printer| printer.name == selected)
    }

    pub(crate) fn focus_is_visible(&self, focus: PrinterSetupFocus) -> bool {
        self.focus_visible && self.focus == focus
    }
}

impl DesktopApp {
    pub(crate) fn printer_setup_uses_detached_window(&self) -> bool {
        self.printer_open
            && self.printer_window.detached
            && self
                .printer_setup_dialog
                .as_ref()
                .is_some_and(|dialog| dialog.target == PrinterSetupTarget::Session)
    }

    pub(crate) fn print_printer_native(&mut self) {
        self.dispatch_sync(crate::backend::AppCommand::PrintPrinterNative(
            self.active_printer_settings().cloned(),
        ));
    }

    pub(crate) fn configure_printer_session(&mut self) -> Task<crate::app::Message> {
        if self.printer_setup_pending {
            return Task::none();
        }
        match self.printer_dialog_mode {
            PrinterDialogMode::Custom => {
                self.open_printer_setup_dialog(PrinterSetupTarget::Session)
            }
            PrinterDialogMode::System => self.configure_printer_session_system(),
        }
    }

    pub(crate) fn finish_printer_session_setup(
        &mut self,
        result: Result<Option<PrinterSettings>, String>,
    ) {
        self.printer_setup_pending = false;
        match result {
            Ok(Some(settings)) => self.printer_session_settings = Some(settings),
            Ok(None) => {}
            Err(error) => self.show_printer_error(error),
        }
    }

    pub(crate) fn configure_printer_settings(&mut self) -> Task<crate::app::Message> {
        if self.settings_dialog.is_none() || self.printer_setup_pending {
            return Task::none();
        }
        let mode = self
            .settings_dialog
            .as_ref()
            .map(|dialog| dialog.draft_printer_dialog_mode)
            .unwrap_or(self.printer_dialog_mode);
        match mode {
            PrinterDialogMode::Custom => {
                self.open_printer_setup_dialog(PrinterSetupTarget::Settings)
            }
            PrinterDialogMode::System => self.configure_printer_settings_system(),
        }
    }

    pub(crate) fn finish_printer_settings_setup(
        &mut self,
        result: Result<Option<PrinterSettings>, String>,
    ) {
        self.printer_setup_pending = false;
        match result {
            Ok(Some(settings)) => {
                if let Some(dialog) = self.settings_dialog.as_mut() {
                    dialog.draft_printer_settings = Some(settings);
                }
            }
            Ok(None) => {}
            Err(error) => self.show_printer_error(error),
        }
    }

    pub(crate) fn active_printer_name(&self) -> Option<&str> {
        self.active_printer_settings()
            .map(|settings| settings.printer_name.as_str())
    }

    pub(crate) fn active_printer_settings(&self) -> Option<&PrinterSettings> {
        self.printer_session_settings
            .as_ref()
            .or(self.printer_default_settings.as_ref())
    }

    pub(crate) fn printer_target_label(&self) -> String {
        self.active_printer_name()
            .map(str::to_owned)
            .unwrap_or_else(|| {
                self.lang
                    .t(Key::Printer(PrinterKey::SystemDefault))
                    .to_owned()
            })
    }

    fn show_printer_error(&mut self, error: String) {
        self.error_notice = Some(format!("{}: {error}", self.lang.t(Key::ErrorPrefix)));
        self.error_notice_dismiss_at = Some(Instant::now() + Duration::from_secs(8));
    }

    fn configure_printer_session_system(&mut self) -> Task<crate::app::Message> {
        self.printer_setup_pending = true;
        Task::perform(
            configure_native_printer_blocking(),
            crate::app::Message::PrinterSessionSetupFinished,
        )
    }

    fn configure_printer_settings_system(&mut self) -> Task<crate::app::Message> {
        self.printer_setup_pending = true;
        Task::perform(
            configure_native_printer_blocking(),
            crate::app::Message::SettingsPrinterSetupFinished,
        )
    }

    fn open_printer_setup_dialog(
        &mut self,
        target: PrinterSetupTarget,
    ) -> Task<crate::app::Message> {
        let selected_name = match target {
            PrinterSetupTarget::Session => self.active_printer_name().map(str::to_owned),
            PrinterSetupTarget::Settings => self
                .settings_dialog
                .as_ref()
                .and_then(|dialog| dialog.draft_printer_settings.as_ref())
                .map(|settings| settings.printer_name.clone()),
        };
        self.printer_setup_pending = true;
        self.printer_setup_dialog = Some(PrinterSetupDialog::new(target, selected_name));
        if self.printer_setup_uses_detached_window()
            && let Some(dialog) = self.printer_setup_dialog.as_mut()
        {
            dialog.owner_ready = false;
        }
        Task::batch([
            self.prepare_detached_printer_dialog(),
            Task::perform(
                list_native_printers_blocking(),
                crate::app::Message::PrinterSetupLoaded,
            ),
        ])
    }
}
