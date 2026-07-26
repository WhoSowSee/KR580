use super::super::dropdown::{DropdownControl, DropdownItem, control};
use super::super::labels::{localized_paper_name, localized_source_name};
use super::localization::localized_options;
use crate::app::{
    Message, PrinterPropertiesDialog, PrinterPropertiesFocus, PrinterPropertyDropdown,
};
use crate::i18n::Lang;
use iced::Element;
use k580_ui::devices::printer::{
    PrinterFeature, PrinterPaper, PrinterPropertyChange, PrinterSource,
};

pub(super) fn feature(
    feature: &PrinterFeature,
    properties: &PrinterPropertiesDialog,
    lang: Lang,
) -> Element<'static, Message> {
    let key = PrinterPropertyDropdown::Feature(feature.name.clone());
    let options = localized_options(feature, lang);
    let selected_name = feature.selected_option.as_deref();
    let selected_label = options
        .iter()
        .find(|option| Some(option.name.as_str()) == selected_name)
        .map(|option| option.display_name.clone())
        .unwrap_or_else(|| "—".to_owned());
    let feature_name = feature.name.clone();
    let items = options
        .into_iter()
        .map(|option| DropdownItem {
            selected: Some(option.name.as_str()) == selected_name,
            label: option.display_name,
            message: Message::PrinterPropertyFeatureSelected(PrinterPropertyChange::Feature {
                feature_name: feature_name.clone(),
                option_name: option.name,
            }),
        })
        .collect();
    dropdown(
        key.clone(),
        selected_label,
        items,
        properties.open_dropdown.as_ref() == Some(&key),
        !properties.applying,
        properties.focus_is_visible(PrinterPropertiesFocus::Dropdown(key)),
        properties.dropdown_highlight,
    )
}

pub(super) fn paper(
    papers: Vec<PrinterPaper>,
    selected: Option<PrinterPaper>,
    properties: &PrinterPropertiesDialog,
    lang: Lang,
) -> Element<'static, Message> {
    let selected_id = selected.as_ref().map(|paper| paper.id);
    let label = selected
        .map(|paper| localized_paper_name(&paper, lang))
        .unwrap_or_else(|| "—".to_owned());
    let items = papers
        .into_iter()
        .map(|paper| DropdownItem {
            selected: Some(paper.id) == selected_id,
            label: localized_paper_name(&paper, lang),
            message: Message::PrinterPropertyPaperSelected(paper.id),
        })
        .collect();
    dropdown_for(PrinterPropertyDropdown::Paper, label, items, properties)
}

pub(super) fn source(
    sources: Vec<PrinterSource>,
    selected: Option<PrinterSource>,
    properties: &PrinterPropertiesDialog,
    lang: Lang,
) -> Element<'static, Message> {
    let selected_id = selected.as_ref().map(|source| source.id);
    let label = selected
        .map(|source| localized_source_name(&source, lang))
        .unwrap_or_else(|| "—".to_owned());
    let items = sources
        .into_iter()
        .map(|source| DropdownItem {
            selected: Some(source.id) == selected_id,
            label: localized_source_name(&source, lang),
            message: Message::PrinterPropertySourceSelected(source.id),
        })
        .collect();
    dropdown_for(PrinterPropertyDropdown::Source, label, items, properties)
}

pub(super) fn preset(
    properties: &PrinterPropertiesDialog,
    placeholder: &str,
) -> Element<'static, Message> {
    let selected = properties.selected_preset.as_deref();
    let label = selected.unwrap_or(placeholder).to_owned();
    let items = properties
        .presets
        .iter()
        .map(|preset| DropdownItem {
            selected: Some(preset.name.as_str()) == selected,
            label: preset.name.clone(),
            message: Message::PrinterPropertyPresetSelected(preset.name.clone()),
        })
        .collect();
    dropdown_for(PrinterPropertyDropdown::Preset, label, items, properties)
}

fn dropdown_for(
    key: PrinterPropertyDropdown,
    label: String,
    items: Vec<DropdownItem>,
    properties: &PrinterPropertiesDialog,
) -> Element<'static, Message> {
    dropdown(
        key.clone(),
        label,
        items,
        properties.open_dropdown.as_ref() == Some(&key),
        !properties.applying,
        properties.focus_is_visible(PrinterPropertiesFocus::Dropdown(key)),
        properties.dropdown_highlight,
    )
}

fn dropdown(
    key: PrinterPropertyDropdown,
    label: String,
    items: Vec<DropdownItem>,
    opened: bool,
    enabled: bool,
    focused: bool,
    highlighted: Option<usize>,
) -> Element<'static, Message> {
    control(
        label,
        items,
        DropdownControl {
            opened,
            enabled,
            focused,
            toggle: Message::PrinterPropertyDropdownToggled(key.clone()),
            dismiss: Message::PrinterPropertyDropdownDismissed(key),
            highlighted,
        },
    )
}
