use super::super::styles::{footer_button, group_style, radio_style};
use super::dropdown;
use super::labels::{PropertyLabel, label};
use super::localization::{feature_label, parameter_label, parameter_visible};
use super::styles::input_style;
use crate::app::{
    Message, PrinterPropertiesDialog, PrinterPropertiesFocus, PrinterPropertiesTab,
    printer_property_parameter_input_id,
};
use crate::i18n::Lang;
use iced::widget::{column, container, radio, row, scrollable, text_input};
use iced::{Alignment, Element, Length, Padding, alignment};
use k580_ui::devices::printer::{PrinterFeature, PrinterFeatureGroup, PrinterOrientation};

pub(super) fn content<'a>(
    properties: &'a PrinterPropertiesDialog,
    lang: Lang,
) -> Element<'a, Message> {
    if properties.loading {
        return centred_label(label(lang, PropertyLabel::Loading));
    }
    let Some(sheet) = properties.sheet.as_ref() else {
        return centred_label(
            properties
                .error
                .as_deref()
                .unwrap_or_else(|| label(lang, PropertyLabel::ProviderError)),
        );
    };
    let body = match properties.tab {
        PrinterPropertiesTab::Favorites => {
            let favorites = sheet
                .features
                .iter()
                .filter(|feature| is_favorite(&feature.name))
                .collect::<Vec<_>>();
            feature_list(
                if favorites.is_empty() {
                    sheet.features.iter().take(10).collect()
                } else {
                    favorites
                },
                properties,
                lang,
            )
        }
        PrinterPropertiesTab::General => feature_list(
            sheet
                .features
                .iter()
                .filter(|feature| feature.group == PrinterFeatureGroup::General)
                .collect(),
            properties,
            lang,
        ),
        PrinterPropertiesTab::Paper => paper_content(properties, lang),
        PrinterPropertiesTab::Graphics => feature_list(
            sheet
                .features
                .iter()
                .filter(|feature| feature.group == PrinterFeatureGroup::Graphics)
                .collect(),
            properties,
            lang,
        ),
        PrinterPropertiesTab::Advanced => advanced_content(properties, lang),
    };
    scrollable(body)
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::hidden(),
        ))
        .height(Length::Fixed(440.0))
        .width(Length::Fill)
        .into()
}

fn feature_list<'a>(
    features: Vec<&'a PrinterFeature>,
    properties: &'a PrinterPropertiesDialog,
    lang: Lang,
) -> Element<'a, Message> {
    if features.is_empty() {
        return centred_label(label(lang, PropertyLabel::NoFeatures));
    }
    let rows = features
        .into_iter()
        .map(|feature| feature_row(feature, properties, lang))
        .collect::<Vec<_>>();
    column(rows).spacing(10).width(Length::Fill).into()
}

fn feature_row(
    feature: &PrinterFeature,
    properties: &PrinterPropertiesDialog,
    lang: Lang,
) -> Element<'static, Message> {
    row![
        container(crate::view::theme::ui_text(
            feature_label(feature, lang),
            13,
            crate::view::theme::tokyo_text(),
        ))
        .padding(Padding {
            top: 9.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        })
        .width(Length::Fixed(230.0)),
        dropdown::feature(feature, properties, lang),
    ]
    .spacing(14)
    .align_y(Alignment::Start)
    .into()
}

fn paper_content<'a>(properties: &'a PrinterPropertiesDialog, lang: Lang) -> Element<'a, Message> {
    let sheet = properties.sheet.as_ref().unwrap();
    let configuration = &sheet.configuration;
    let selected_paper = configuration.selected_paper().cloned();
    let selected_source = configuration.selected_source().cloned();
    let standard = column![
        standard_row(
            label(lang, PropertyLabel::Size),
            dropdown::paper(configuration.papers.clone(), selected_paper, properties),
        ),
        standard_row(
            label(lang, PropertyLabel::Source),
            dropdown::source(configuration.sources.clone(), selected_source, properties),
        ),
        row![
            crate::view::theme::ui_text(
                label(lang, PropertyLabel::Orientation),
                13,
                crate::view::theme::tokyo_text(),
            )
            .width(Length::Fixed(230.0)),
            radio(
                label(lang, PropertyLabel::Portrait),
                PrinterOrientation::Portrait,
                Some(configuration.settings.orientation),
                Message::PrinterPropertyOrientationSelected,
            )
            .size(18)
            .spacing(8)
            .text_size(13)
            .style(radio_style(
                properties.focus == PrinterPropertiesFocus::Portrait,
            )),
            radio(
                label(lang, PropertyLabel::Landscape),
                PrinterOrientation::Landscape,
                Some(configuration.settings.orientation),
                Message::PrinterPropertyOrientationSelected,
            )
            .size(18)
            .spacing(8)
            .text_size(13)
            .style(radio_style(
                properties.focus == PrinterPropertiesFocus::Landscape,
            )),
        ]
        .spacing(16)
        .align_y(Alignment::Center),
    ]
    .spacing(12);
    let driver_features = sheet
        .features
        .iter()
        .filter(|feature| feature.group == PrinterFeatureGroup::Paper)
        .collect::<Vec<_>>();
    let mut content = column![group(standard.into())].spacing(12);
    if !driver_features.is_empty() {
        content = content.push(feature_list(driver_features, properties, lang));
    }
    content.into()
}

fn advanced_content<'a>(
    properties: &'a PrinterPropertiesDialog,
    lang: Lang,
) -> Element<'a, Message> {
    let sheet = properties.sheet.as_ref().unwrap();
    let mut items: Vec<Element<'a, Message>> = vec![feature_list(
        sheet.features.iter().collect(),
        properties,
        lang,
    )];
    let parameters = sheet
        .parameters
        .iter()
        .filter(|parameter| parameter_visible(parameter))
        .collect::<Vec<_>>();
    if !parameters.is_empty() {
        let heading: Element<'a, Message> = crate::view::theme::ui_text(
            label(lang, PropertyLabel::Parameters),
            14,
            crate::view::theme::tokyo_text(),
        )
        .into();
        items.push(heading);
        for parameter in parameters {
            let name = parameter.name.clone();
            let apply_name = name.clone();
            let input_id = printer_property_parameter_input_id(&parameter.name);
            let value = properties
                .parameter_values
                .get(&parameter.name)
                .map(String::as_str)
                .unwrap_or(&parameter.value);
            let input = text_input("", value)
                .id(input_id)
                .on_input(move |value| Message::PrinterPropertyParameterChanged {
                    name: name.clone(),
                    value,
                })
                .on_submit(Message::PrinterPropertyParameterApply(apply_name.clone()))
                .size(13)
                .padding([8, 10])
                .style(input_style)
                .width(Length::Fill);
            items.push(
                row![
                    crate::view::theme::ui_text(
                        parameter_label(parameter, lang),
                        13,
                        crate::view::theme::tokyo_text(),
                    )
                    .width(Length::Fixed(230.0)),
                    input,
                    footer_button(
                        label(lang, PropertyLabel::Apply),
                        !properties.applying,
                        properties.focus
                            == PrinterPropertiesFocus::ParameterApply(parameter.name.clone()),
                    )
                    .width(Length::Fixed(96.0))
                    .on_press_maybe((!properties.applying).then_some(
                        Message::PrinterPropertyParameterApply(parameter.name.clone()),
                    )),
                ]
                .spacing(10)
                .align_y(Alignment::Center)
                .into(),
            );
        }
    }
    column(items).spacing(16).into()
}

fn standard_row<'a>(title: &'static str, control: Element<'a, Message>) -> Element<'a, Message> {
    row![
        container(crate::view::theme::ui_text(
            title,
            13,
            crate::view::theme::tokyo_text(),
        ))
        .padding(Padding {
            top: 9.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        })
        .width(Length::Fixed(230.0)),
        control,
    ]
    .spacing(14)
    .align_y(Alignment::Start)
    .into()
}

fn group(content: Element<'static, Message>) -> Element<'static, Message> {
    container(content)
        .padding(14)
        .width(Length::Fill)
        .style(group_style)
        .into()
}

fn centred_label<'a>(text: &'a str) -> Element<'a, Message> {
    container(crate::view::theme::ui_text(
        text,
        13,
        crate::view::theme::tokyo_muted(),
    ))
    .width(Length::Fill)
    .height(Length::Fixed(440.0))
    .align_x(alignment::Horizontal::Center)
    .align_y(alignment::Vertical::Center)
    .into()
}

fn is_favorite(name: &str) -> bool {
    let name = name.rsplit_once(':').map_or(name, |(_, local)| local);
    matches!(
        name,
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
