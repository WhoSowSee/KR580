use super::super::super::{
    PRINTER_PROPERTIES_PRESET_INPUT_ID, PrinterPropertiesDialog, PrinterPropertiesFocus,
    PrinterPropertiesTab, PrinterPropertyDropdown, printer_property_parameter_input_id,
};
use crate::app::{DesktopApp, Message};
use iced::advanced::widget::operation::focusable::{Focusable, unfocus};
use iced::advanced::widget::operation::{Operation, Outcome};
use iced::widget::{Id, operation};
use iced::{Rectangle, Task};
use k580_ui::devices::printer::{PrinterFeature, PrinterFeatureGroup, PrinterOrientation};

impl DesktopApp {
    pub(super) fn begin_printer_properties_focus_cycle(&self, backward: bool) -> Task<Message> {
        iced::advanced::widget::operate(find_focused_or_none())
            .map(move |focused| Message::PrinterPropertiesFocusResolved { focused, backward })
    }

    pub(super) fn resolve_printer_properties_focus(
        &mut self,
        focused: Option<Id>,
        backward: bool,
    ) -> Task<Message> {
        let native_focus = self
            .properties()
            .and_then(|properties| focused.and_then(|id| input_focus(properties, &id)));
        if let Some(native_focus) = native_focus
            && let Some(properties) = self.properties_mut()
        {
            properties.focus = native_focus;
        }
        self.cycle_printer_properties_focus(backward)
    }

    pub(super) fn activate_printer_properties_focus(&mut self) -> Task<Message> {
        if let Some(properties) = self.properties_mut() {
            properties.focus_visible = false;
        }
        let Some(properties) = self.properties() else {
            return Task::none();
        };
        if properties.open_dropdown.is_some() {
            return self.activate_property_dropdown();
        }
        let focus = properties.focus.clone();
        if let PrinterPropertiesFocus::Tab(tab) = focus {
            if let Some(properties) = self.properties_mut() {
                properties.tab = tab;
                properties.open_dropdown = None;
                properties.dropdown_highlight = None;
            }
            return Task::none();
        }
        activation_message(properties).map_or_else(Task::none, Task::done)
    }

    fn cycle_printer_properties_focus(&mut self, backward: bool) -> Task<Message> {
        let Some(properties) = self.properties() else {
            return Task::none();
        };
        let order = focus_order(properties);
        if order.is_empty() {
            return Task::none();
        }
        let current = order
            .iter()
            .position(|candidate| candidate == &properties.focus)
            .unwrap_or(if backward { 0 } else { order.len() - 1 });
        let next = if backward {
            (current + order.len() - 1) % order.len()
        } else {
            (current + 1) % order.len()
        };
        let focus = order[next].clone();
        if let Some(properties) = self.properties_mut() {
            properties.focus = focus.clone();
            properties.focus_visible = true;
            properties.open_dropdown = None;
            properties.dropdown_highlight = None;
        }
        focus_task(focus)
    }
}

fn focus_order(properties: &PrinterPropertiesDialog) -> Vec<PrinterPropertiesFocus> {
    let mut order = vec![
        PrinterPropertiesFocus::Close,
        PrinterPropertiesFocus::Tab(PrinterPropertiesTab::Favorites),
        PrinterPropertiesFocus::Tab(PrinterPropertiesTab::General),
        PrinterPropertiesFocus::Tab(PrinterPropertiesTab::Paper),
        PrinterPropertiesFocus::Tab(PrinterPropertiesTab::Graphics),
        PrinterPropertiesFocus::Tab(PrinterPropertiesTab::Advanced),
    ];
    if let Some(sheet) = properties.sheet.as_ref() {
        match properties.tab {
            PrinterPropertiesTab::Favorites => {
                let favorites = sheet
                    .features
                    .iter()
                    .filter(|feature| is_favorite(&feature.name))
                    .collect::<Vec<_>>();
                push_features(
                    &mut order,
                    if favorites.is_empty() {
                        sheet.features.iter().take(10).collect()
                    } else {
                        favorites
                    },
                );
            }
            PrinterPropertiesTab::General => push_features(
                &mut order,
                sheet
                    .features
                    .iter()
                    .filter(|feature| feature.group == PrinterFeatureGroup::General)
                    .collect(),
            ),
            PrinterPropertiesTab::Paper => {
                if !sheet.configuration.papers.is_empty() {
                    order.push(PrinterPropertiesFocus::Dropdown(
                        PrinterPropertyDropdown::Paper,
                    ));
                }
                if !sheet.configuration.sources.is_empty() {
                    order.push(PrinterPropertiesFocus::Dropdown(
                        PrinterPropertyDropdown::Source,
                    ));
                }
                order.extend([
                    PrinterPropertiesFocus::Portrait,
                    PrinterPropertiesFocus::Landscape,
                ]);
                push_features(
                    &mut order,
                    sheet
                        .features
                        .iter()
                        .filter(|feature| feature.group == PrinterFeatureGroup::Paper)
                        .collect(),
                );
            }
            PrinterPropertiesTab::Graphics => push_features(
                &mut order,
                sheet
                    .features
                    .iter()
                    .filter(|feature| feature.group == PrinterFeatureGroup::Graphics)
                    .collect(),
            ),
            PrinterPropertiesTab::Advanced => {
                push_features(&mut order, sheet.features.iter().collect());
                for parameter in sheet
                    .parameters
                    .iter()
                    .filter(|parameter| local_name(&parameter.name) != "PageDevmodeSnapshot")
                {
                    order.push(PrinterPropertiesFocus::ParameterInput(
                        parameter.name.clone(),
                    ));
                    if !properties.applying {
                        order.push(PrinterPropertiesFocus::ParameterApply(
                            parameter.name.clone(),
                        ));
                    }
                }
            }
        }
    }
    if !properties.presets.is_empty() && !properties.applying {
        order.push(PrinterPropertiesFocus::Dropdown(
            PrinterPropertyDropdown::Preset,
        ));
    }
    order.push(PrinterPropertiesFocus::PresetName);
    if properties.selected_preset.is_some() && !properties.applying {
        order.push(PrinterPropertiesFocus::PresetDelete);
    }
    if properties.sheet.is_some()
        && !properties.applying
        && !properties.preset_name.trim().is_empty()
    {
        order.push(PrinterPropertiesFocus::PresetSave);
    }
    order.push(PrinterPropertiesFocus::Cancel);
    if properties.sheet.is_some() && !properties.loading && !properties.applying {
        order.push(PrinterPropertiesFocus::Ok);
    }
    order
}

fn push_features(order: &mut Vec<PrinterPropertiesFocus>, features: Vec<&PrinterFeature>) {
    order.extend(
        features
            .into_iter()
            .filter(|feature| {
                feature.options.iter().any(|option| {
                    !option.constrained || feature.selected_option.as_deref() == Some(&option.name)
                })
            })
            .map(|feature| {
                PrinterPropertiesFocus::Dropdown(PrinterPropertyDropdown::Feature(
                    feature.name.clone(),
                ))
            }),
    );
}

fn activation_message(properties: &PrinterPropertiesDialog) -> Option<Message> {
    match &properties.focus {
        PrinterPropertiesFocus::Close | PrinterPropertiesFocus::Cancel => {
            Some(Message::ClosePrinterProperties)
        }
        PrinterPropertiesFocus::Tab(_) => None,
        PrinterPropertiesFocus::Dropdown(dropdown) => {
            Some(Message::PrinterPropertyDropdownToggled(dropdown.clone()))
        }
        PrinterPropertiesFocus::Portrait => Some(Message::PrinterPropertyOrientationSelected(
            PrinterOrientation::Portrait,
        )),
        PrinterPropertiesFocus::Landscape => Some(Message::PrinterPropertyOrientationSelected(
            PrinterOrientation::Landscape,
        )),
        PrinterPropertiesFocus::ParameterInput(name)
        | PrinterPropertiesFocus::ParameterApply(name) => {
            Some(Message::PrinterPropertyParameterApply(name.clone()))
        }
        PrinterPropertiesFocus::PresetName | PrinterPropertiesFocus::PresetSave => {
            (!properties.preset_name.trim().is_empty())
                .then_some(Message::PrinterPropertyPresetSave)
        }
        PrinterPropertiesFocus::PresetDelete => {
            properties.selected_preset.as_ref()?;
            Some(Message::PrinterPropertyPresetDelete)
        }
        PrinterPropertiesFocus::Ok => Some(Message::PrinterPropertyConfirmed),
    }
}

fn input_focus(properties: &PrinterPropertiesDialog, id: &Id) -> Option<PrinterPropertiesFocus> {
    if id == &Id::from(PRINTER_PROPERTIES_PRESET_INPUT_ID) {
        return Some(PrinterPropertiesFocus::PresetName);
    }
    properties
        .sheet
        .as_ref()?
        .parameters
        .iter()
        .find_map(|parameter| {
            (id == &Id::from(printer_property_parameter_input_id(&parameter.name)))
                .then(|| PrinterPropertiesFocus::ParameterInput(parameter.name.clone()))
        })
}

fn focus_task(focus: PrinterPropertiesFocus) -> Task<Message> {
    match focus {
        PrinterPropertiesFocus::PresetName => operation::focus(PRINTER_PROPERTIES_PRESET_INPUT_ID),
        PrinterPropertiesFocus::ParameterInput(name) => {
            operation::focus(printer_property_parameter_input_id(&name))
        }
        _ => iced::advanced::widget::operate(unfocus()),
    }
}

fn is_favorite(name: &str) -> bool {
    matches!(
        local_name(name),
        "PageResolution"
            | "DocumentDarkenText"
            | "DocumentAllTextToBlack"
            | "DocumentFineEdge"
            | "DocumentTonerSave"
            | "JobPageOrder"
            | "DocumentSkipBlankPages"
            | "DocumentNUp"
            | "JobDuplexAllDocumentsContiguously"
    )
}

fn local_name(name: &str) -> &str {
    name.rsplit_once(':').map_or(name, |(_, local)| local)
}

fn find_focused_or_none() -> impl Operation<Option<Id>> {
    struct FindFocused {
        focused: Option<Id>,
    }

    impl Operation<Option<Id>> for FindFocused {
        fn focusable(&mut self, id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            if state.is_focused() {
                self.focused = id.cloned();
            }
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<Option<Id>>)) {
            operate(self);
        }

        fn finish(&self) -> Outcome<Option<Id>> {
            Outcome::Some(self.focused.clone())
        }
    }

    FindFocused { focused: None }
}
