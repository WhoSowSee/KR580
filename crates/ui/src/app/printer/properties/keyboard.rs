use super::super::{PrinterPropertiesDialog, PrinterPropertyDropdown};
use crate::app::{DesktopApp, Message};
use iced::Task;
use k580_ui::devices::printer::PrinterPropertyChange;

impl DesktopApp {
    pub(super) fn toggle_property_dropdown(&mut self, dropdown: PrinterPropertyDropdown) {
        let selected = self
            .properties()
            .and_then(|properties| selected_index(properties, &dropdown));
        let Some(properties) = self.properties_mut() else {
            return;
        };
        if properties.open_dropdown.as_ref() == Some(&dropdown) {
            properties.open_dropdown = None;
            properties.dropdown_highlight = None;
            return;
        }
        properties.open_dropdown = Some(dropdown);
        properties.dropdown_highlight = selected;
    }

    pub(super) fn move_property_dropdown_highlight(&mut self, direction: i32) {
        let Some(properties) = self.properties_mut() else {
            return;
        };
        let Some(dropdown) = properties.open_dropdown.clone() else {
            return;
        };
        let count = option_count(properties, &dropdown);
        if count == 0 {
            return;
        }
        let current = properties
            .dropdown_highlight
            .or_else(|| selected_index(properties, &dropdown))
            .unwrap_or(0) as i32;
        properties.dropdown_highlight =
            Some((current - direction).clamp(0, count as i32 - 1) as usize);
    }

    pub(super) fn activate_property_dropdown(&self) -> Task<Message> {
        self.properties()
            .and_then(highlighted_message)
            .map_or_else(Task::none, Task::done)
    }
}

fn option_count(properties: &PrinterPropertiesDialog, dropdown: &PrinterPropertyDropdown) -> usize {
    match dropdown {
        PrinterPropertyDropdown::Feature(name) => properties
            .sheet
            .as_ref()
            .and_then(|sheet| sheet.features.iter().find(|feature| feature.name == *name))
            .map_or(0, |feature| feature.options.len()),
        PrinterPropertyDropdown::Paper => properties
            .sheet
            .as_ref()
            .map_or(0, |sheet| sheet.configuration.papers.len()),
        PrinterPropertyDropdown::Source => properties
            .sheet
            .as_ref()
            .map_or(0, |sheet| sheet.configuration.sources.len()),
        PrinterPropertyDropdown::Preset => properties.presets.len(),
    }
}

fn selected_index(
    properties: &PrinterPropertiesDialog,
    dropdown: &PrinterPropertyDropdown,
) -> Option<usize> {
    match dropdown {
        PrinterPropertyDropdown::Feature(name) => {
            let feature = properties
                .sheet
                .as_ref()?
                .features
                .iter()
                .find(|feature| feature.name == *name)?;
            let selected = feature.selected_option.as_deref()?;
            feature
                .options
                .iter()
                .position(|option| option.name == selected)
        }
        PrinterPropertyDropdown::Paper => {
            let configuration = &properties.sheet.as_ref()?.configuration;
            let selected = configuration.settings.paper_id?;
            configuration
                .papers
                .iter()
                .position(|paper| paper.id == selected)
        }
        PrinterPropertyDropdown::Source => {
            let configuration = &properties.sheet.as_ref()?.configuration;
            let selected = configuration.settings.source_id?;
            configuration
                .sources
                .iter()
                .position(|source| source.id == selected)
        }
        PrinterPropertyDropdown::Preset => {
            let selected = properties.selected_preset.as_deref()?;
            properties
                .presets
                .iter()
                .position(|preset| preset.name == selected)
        }
    }
}

fn highlighted_message(properties: &PrinterPropertiesDialog) -> Option<Message> {
    let dropdown = properties.open_dropdown.as_ref()?;
    let index = properties
        .dropdown_highlight
        .or_else(|| selected_index(properties, dropdown))?;
    match dropdown {
        PrinterPropertyDropdown::Feature(name) => {
            let feature = properties
                .sheet
                .as_ref()?
                .features
                .iter()
                .find(|feature| feature.name == *name)?;
            let option = feature.options.get(index)?;
            Some(Message::PrinterPropertyFeatureSelected(
                PrinterPropertyChange::Feature {
                    feature_name: feature.name.clone(),
                    option_name: option.name.clone(),
                },
            ))
        }
        PrinterPropertyDropdown::Paper => properties
            .sheet
            .as_ref()?
            .configuration
            .papers
            .get(index)
            .map(|paper| Message::PrinterPropertyPaperSelected(paper.id)),
        PrinterPropertyDropdown::Source => properties
            .sheet
            .as_ref()?
            .configuration
            .sources
            .get(index)
            .map(|source| Message::PrinterPropertySourceSelected(source.id)),
        PrinterPropertyDropdown::Preset => properties
            .presets
            .get(index)
            .map(|preset| Message::PrinterPropertyPresetSelected(preset.name.clone())),
    }
}
