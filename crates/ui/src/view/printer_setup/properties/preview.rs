use super::super::styles::{footer_button, group_style};
use super::dropdown;
use super::labels::{PropertyLabel, label};
use super::styles::{input_style, paper_style};
use crate::app::{
    Message, PRINTER_PROPERTIES_PRESET_INPUT_ID, PrinterPropertiesDialog, PrinterPropertiesFocus,
};
use crate::i18n::Lang;
use crate::view::theme::{MONO_FONT, tokyo_muted, tokyo_text, ui_text};
use iced::widget::{Space, column, container, row, text, text_input};
use iced::{Alignment, Color, Element, Length, alignment};
use k580_ui::devices::printer::PrinterOrientation;

pub(super) fn side_panel<'a>(
    properties: &'a PrinterPropertiesDialog,
    lang: Lang,
) -> Element<'a, Message> {
    column![profiles(properties, lang), preview(properties, lang)]
        .spacing(12)
        .height(Length::Fill)
        .width(Length::Fixed(286.0))
        .into()
}

fn profiles<'a>(properties: &'a PrinterPropertiesDialog, lang: Lang) -> Element<'a, Message> {
    let selector = dropdown::preset(properties, label(lang, PropertyLabel::Profiles));
    let name = text_input(
        label(lang, PropertyLabel::PresetName),
        &properties.preset_name,
    )
    .id(PRINTER_PROPERTIES_PRESET_INPUT_ID)
    .on_input(Message::PrinterPropertyPresetNameChanged)
    .size(12)
    .padding([7, 10])
    .style(move |_theme, _status| {
        input_style(properties.focus_is_visible(PrinterPropertiesFocus::PresetName))
    })
    .width(Length::Fill);
    let can_save = properties.sheet.is_some()
        && !properties.applying
        && !properties.preset_name.trim().is_empty();
    let can_delete = properties.selected_preset.is_some() && !properties.applying;
    let controls = column![
        selector,
        name,
        row![
            footer_button(
                label(lang, PropertyLabel::Delete),
                can_delete,
                properties.focus_is_visible(PrinterPropertiesFocus::PresetDelete),
            )
            .width(Length::Fill)
            .on_press_maybe(can_delete.then_some(Message::PrinterPropertyPresetDelete)),
            footer_button(
                label(lang, PropertyLabel::Save),
                can_save,
                properties.focus_is_visible(PrinterPropertiesFocus::PresetSave),
            )
            .width(Length::Fill)
            .on_press_maybe(can_save.then_some(Message::PrinterPropertyPresetSave)),
        ]
        .spacing(8),
    ]
    .spacing(8);
    bordered(
        column![
            ui_text(label(lang, PropertyLabel::Profiles), 13, tokyo_text()),
            controls,
        ]
        .spacing(9)
        .into(),
    )
}

fn preview<'a>(properties: &'a PrinterPropertiesDialog, lang: Lang) -> Element<'a, Message> {
    let orientation = properties
        .sheet
        .as_ref()
        .map(|sheet| sheet.configuration.settings.orientation)
        .unwrap_or_default();
    let (width, height) = match orientation {
        PrinterOrientation::Portrait => (128.0, 180.0),
        PrinterOrientation::Landscape => (180.0, 128.0),
    };
    let preview_text = preview_text(&properties.preview_text);
    let page: Element<'a, Message> = if preview_text.is_empty() {
        container(ui_text(
            label(lang, PropertyLabel::NoPreview),
            11,
            tokyo_muted(),
        ))
        .width(Length::Fixed(width))
        .height(Length::Fixed(height))
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .style(paper_style)
        .into()
    } else {
        container(
            text(preview_text)
                .font(MONO_FONT)
                .size(7)
                .color(Color::from_rgb8(32, 32, 32))
                .width(Length::Fill),
        )
        .padding(16)
        .width(Length::Fixed(width))
        .height(Length::Fixed(height))
        .style(paper_style)
        .into()
    };
    let paper_name = properties
        .sheet
        .as_ref()
        .and_then(|sheet| sheet.configuration.settings.paper_name.as_deref())
        .unwrap_or("—");
    bordered(
        column![
            ui_text(label(lang, PropertyLabel::Preview), 13, tokyo_text()),
            container(
                column![
                    Space::new().height(Length::Fill),
                    row![
                        Space::new().width(Length::Fill),
                        page,
                        Space::new().width(Length::Fill),
                    ],
                    Space::new().height(Length::Fill),
                ]
                .height(Length::Fixed(196.0))
            )
            .width(Length::Fill),
            ui_text(paper_name.to_owned(), 11, tokyo_muted()),
        ]
        .spacing(8)
        .align_x(Alignment::Center)
        .into(),
    )
}

fn bordered(content: Element<'_, Message>) -> Element<'_, Message> {
    container(content)
        .padding(12)
        .width(Length::Fill)
        .style(group_style)
        .into()
}

fn preview_text(value: &str) -> String {
    value
        .lines()
        .take(24)
        .map(|line| line.chars().take(58).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}
