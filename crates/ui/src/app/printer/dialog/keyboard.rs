use super::super::{PrinterSetupDialog, PrinterSetupDropdown, PrinterSetupFocus};
use crate::app::{DesktopApp, Message};
use iced::Task;
use k580_ui::devices::printer::PrinterOrientation;

impl DesktopApp {
    pub(super) fn toggle_printer_setup_dropdown(&mut self, dropdown: PrinterSetupDropdown) {
        let Some(dialog) = self.printer_setup_dialog.as_mut() else {
            return;
        };
        dialog.focus = dropdown_focus(dropdown);
        if dialog.open_dropdown == Some(dropdown) {
            close_dropdown(dialog);
            return;
        }
        dialog.dropdown_highlight = selected_dropdown_index(dialog, dropdown);
        dialog.open_dropdown = Some(dropdown);
    }

    pub(super) fn close_printer_setup_dropdown(&mut self) {
        if let Some(dialog) = self.printer_setup_dialog.as_mut() {
            close_dropdown(dialog);
        }
    }

    pub(super) fn move_printer_setup_dropdown_highlight(&mut self, direction: i32) {
        let Some(dialog) = self.printer_setup_dialog.as_mut() else {
            return;
        };
        let Some(dropdown) = dialog.open_dropdown else {
            return;
        };
        let count = dropdown_option_count(dialog, dropdown);
        if count == 0 {
            return;
        }
        let current = dialog
            .dropdown_highlight
            .or_else(|| selected_dropdown_index(dialog, dropdown))
            .unwrap_or(0) as i32;
        dialog.dropdown_highlight = Some((current - direction).clamp(0, count as i32 - 1) as usize);
    }

    pub(super) fn cycle_printer_setup_focus(&mut self, backward: bool) {
        let Some(dialog) = self.printer_setup_dialog.as_mut() else {
            return;
        };
        close_dropdown(dialog);
        let current = PrinterSetupFocus::ALL
            .iter()
            .position(|candidate| *candidate == dialog.focus)
            .unwrap_or(0);
        let step = if backward {
            PrinterSetupFocus::ALL.len() - 1
        } else {
            1
        };
        for offset in 1..=PrinterSetupFocus::ALL.len() {
            let index = (current + offset * step) % PrinterSetupFocus::ALL.len();
            let candidate = PrinterSetupFocus::ALL[index];
            if focus_enabled(dialog, candidate) {
                dialog.focus = candidate;
                break;
            }
        }
    }

    pub(super) fn activate_printer_setup_focus(&mut self) -> Task<Message> {
        let Some(dialog) = self.printer_setup_dialog.as_ref() else {
            return Task::none();
        };
        if let Some(message) = highlighted_dropdown_message(dialog) {
            return Task::done(message);
        }
        let focus = dialog.focus;
        if !focus_enabled(dialog, focus) {
            return Task::none();
        }
        Task::done(match focus {
            PrinterSetupFocus::Close | PrinterSetupFocus::Cancel => Message::ClosePrinterSetup,
            PrinterSetupFocus::Printer => {
                Message::PrinterSetupDropdownToggled(PrinterSetupDropdown::Printer)
            }
            PrinterSetupFocus::Properties => Message::PrinterSetupProperties,
            PrinterSetupFocus::Paper => {
                Message::PrinterSetupDropdownToggled(PrinterSetupDropdown::Paper)
            }
            PrinterSetupFocus::Source => {
                Message::PrinterSetupDropdownToggled(PrinterSetupDropdown::Source)
            }
            PrinterSetupFocus::Portrait => {
                Message::PrinterSetupOrientationSelected(PrinterOrientation::Portrait)
            }
            PrinterSetupFocus::Landscape => {
                Message::PrinterSetupOrientationSelected(PrinterOrientation::Landscape)
            }
            PrinterSetupFocus::Ok => Message::PrinterSetupConfirmed,
        })
    }
}

fn close_dropdown(dialog: &mut PrinterSetupDialog) {
    dialog.open_dropdown = None;
    dialog.dropdown_highlight = None;
}

fn dropdown_focus(dropdown: PrinterSetupDropdown) -> PrinterSetupFocus {
    match dropdown {
        PrinterSetupDropdown::Printer => PrinterSetupFocus::Printer,
        PrinterSetupDropdown::Paper => PrinterSetupFocus::Paper,
        PrinterSetupDropdown::Source => PrinterSetupFocus::Source,
    }
}

fn dropdown_option_count(dialog: &PrinterSetupDialog, dropdown: PrinterSetupDropdown) -> usize {
    match dropdown {
        PrinterSetupDropdown::Printer => dialog.printers.len(),
        PrinterSetupDropdown::Paper => dialog
            .configuration
            .as_ref()
            .map_or(0, |configuration| configuration.papers.len()),
        PrinterSetupDropdown::Source => dialog
            .configuration
            .as_ref()
            .map_or(0, |configuration| configuration.sources.len()),
    }
}

fn selected_dropdown_index(
    dialog: &PrinterSetupDialog,
    dropdown: PrinterSetupDropdown,
) -> Option<usize> {
    match dropdown {
        PrinterSetupDropdown::Printer => {
            let selected = dialog.selected_name.as_deref()?;
            dialog
                .printers
                .iter()
                .position(|printer| printer.name == selected)
        }
        PrinterSetupDropdown::Paper => {
            let configuration = dialog.configuration.as_ref()?;
            let selected = configuration.settings.paper_id?;
            configuration
                .papers
                .iter()
                .position(|paper| paper.id == selected)
        }
        PrinterSetupDropdown::Source => {
            let configuration = dialog.configuration.as_ref()?;
            let selected = configuration.settings.source_id?;
            configuration
                .sources
                .iter()
                .position(|source| source.id == selected)
        }
    }
}

fn highlighted_dropdown_message(dialog: &PrinterSetupDialog) -> Option<Message> {
    let dropdown = dialog.open_dropdown?;
    let index = dialog
        .dropdown_highlight
        .or_else(|| selected_dropdown_index(dialog, dropdown))?;
    match dropdown {
        PrinterSetupDropdown::Printer => dialog
            .printers
            .get(index)
            .map(|printer| Message::PrinterSetupSelected(printer.name.clone())),
        PrinterSetupDropdown::Paper => dialog
            .configuration
            .as_ref()?
            .papers
            .get(index)
            .map(|paper| Message::PrinterSetupPaperSelected(paper.id)),
        PrinterSetupDropdown::Source => dialog
            .configuration
            .as_ref()?
            .sources
            .get(index)
            .map(|source| Message::PrinterSetupSourceSelected(source.id)),
    }
}

fn focus_enabled(dialog: &PrinterSetupDialog, focus: PrinterSetupFocus) -> bool {
    let configuration_ready = dialog.configuration.is_some() && !dialog.configuration_loading;
    match focus {
        PrinterSetupFocus::Close | PrinterSetupFocus::Cancel => true,
        PrinterSetupFocus::Printer => !dialog.loading && !dialog.printers.is_empty(),
        PrinterSetupFocus::Properties => configuration_ready && !dialog.properties_pending,
        PrinterSetupFocus::Paper => {
            configuration_ready
                && dialog
                    .configuration
                    .as_ref()
                    .is_some_and(|configuration| !configuration.papers.is_empty())
        }
        PrinterSetupFocus::Source => {
            configuration_ready
                && dialog
                    .configuration
                    .as_ref()
                    .is_some_and(|configuration| !configuration.sources.is_empty())
        }
        PrinterSetupFocus::Portrait | PrinterSetupFocus::Landscape => configuration_ready,
        PrinterSetupFocus::Ok => configuration_ready && !dialog.properties_pending,
    }
}
