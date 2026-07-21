mod content;
mod dropdown;
mod labels;
mod localization;
mod preview;
mod styles;

use self::labels::{PropertyLabel, label};
use self::styles::{active_tab_line, attention_panel_style, tab_style};
use super::styles::{footer_button, separator};
use crate::app::{
    Message, PrinterPropertiesDialog, PrinterPropertiesFocus, PrinterPropertiesTab,
    PrinterSetupDialog,
};
use crate::i18n::Lang;
use crate::view::icons;
use crate::view::styles::modal_backdrop_style;
use crate::view::theme::{tokyo_red, tokyo_text, ui_text};
use crate::view::widgets::modal_icon_button_focused;
use iced::widget::{Space, button, column, container, mouse_area, opaque, row, stack};
use iced::{Alignment, Element, Length, alignment};
use std::time::Instant;

const DIALOG_WIDTH: f32 = 1040.0;

pub(super) fn printer_properties_modal_overlay<'a>(
    setup: &'a PrinterSetupDialog,
    properties: &'a PrinterPropertiesDialog,
    lang: Lang,
) -> Element<'a, Message> {
    let backdrop = mouse_area(
        container(Space::new())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(modal_backdrop_style),
    )
    .on_press(Message::ClosePrinterProperties);
    let panel = printer_properties_panel(
        setup,
        properties,
        lang,
        Length::Fixed(DIALOG_WIDTH),
        Length::Shrink,
    );
    let centred = container(opaque(panel))
        .center_x(Length::Fill)
        .center_y(Length::Fill);
    stack![opaque(backdrop), centred]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

pub(super) fn printer_properties_window_view<'a>(
    setup: &'a PrinterSetupDialog,
    properties: &'a PrinterPropertiesDialog,
    lang: Lang,
) -> Element<'a, Message> {
    printer_properties_panel(setup, properties, lang, Length::Fill, Length::Fill)
}

fn printer_properties_panel<'a>(
    setup: &'a PrinterSetupDialog,
    properties: &'a PrinterPropertiesDialog,
    lang: Lang,
    width: Length,
    height: Length,
) -> Element<'a, Message> {
    let printer_name = setup.selected_name.as_deref().unwrap_or_default();
    let title = format!("{}: {}", label(lang, PropertyLabel::Title), printer_name);
    let close = modal_icon_button_focused(
        icons::window_close(),
        Some(Message::ClosePrinterProperties),
        label(lang, PropertyLabel::Cancel),
        34.0,
        true,
        properties.focus_is_visible(PrinterPropertiesFocus::Close),
    );
    let header = row![
        ui_text(title, 18, tokyo_text()),
        Space::new().width(Length::Fill),
        close,
    ]
    .align_y(Alignment::Center);
    let body = row![
        content::content(properties, lang),
        preview::side_panel(properties, lang),
    ]
    .spacing(14)
    .height(Length::Fixed(440.0));
    let ready = properties.sheet.is_some() && !properties.loading && !properties.applying;
    let footer = row![
        Space::new().width(Length::Fill),
        footer_button(
            label(lang, PropertyLabel::Cancel),
            true,
            properties.focus_is_visible(PrinterPropertiesFocus::Cancel),
        )
        .on_press(Message::ClosePrinterProperties),
        footer_button(
            label(lang, PropertyLabel::Ok),
            ready,
            properties.focus_is_visible(PrinterPropertiesFocus::Ok),
        )
        .on_press_maybe(ready.then_some(Message::PrinterPropertyConfirmed)),
    ]
    .spacing(10)
    .align_y(Alignment::Center);
    let mut panel_content = column![header, separator(), tabs(properties, lang), body].spacing(12);
    if let Some(error) = display_error(properties) {
        let error: Element<'a, Message> = ui_text(error, 12, tokyo_red()).into();
        panel_content = panel_content.push(error);
    }
    let attention = properties.attention_strength(Instant::now());
    container(
        panel_content
            .push(separator())
            .push(footer)
            .spacing(12)
            .padding(18)
            .width(width),
    )
    .width(width)
    .height(height)
    .style(move |theme| attention_panel_style(theme, attention))
    .into()
}

fn tabs(properties: &PrinterPropertiesDialog, lang: Lang) -> Element<'static, Message> {
    row![
        tab(
            label(lang, PropertyLabel::Favorites),
            PrinterPropertiesTab::Favorites,
            properties,
        ),
        tab(
            label(lang, PropertyLabel::General),
            PrinterPropertiesTab::General,
            properties,
        ),
        tab(
            label(lang, PropertyLabel::Paper),
            PrinterPropertiesTab::Paper,
            properties,
        ),
        tab(
            label(lang, PropertyLabel::Graphics),
            PrinterPropertiesTab::Graphics,
            properties,
        ),
        tab(
            label(lang, PropertyLabel::Advanced),
            PrinterPropertiesTab::Advanced,
            properties,
        ),
    ]
    .spacing(4)
    .width(Length::Fill)
    .into()
}

fn tab(
    title: &'static str,
    tab: PrinterPropertiesTab,
    properties: &PrinterPropertiesDialog,
) -> Element<'static, Message> {
    let selected = tab == properties.tab;
    let focused = properties.focus_is_visible(PrinterPropertiesFocus::Tab(tab));
    let line = container(Space::new())
        .width(Length::Fill)
        .height(Length::Fixed(2.0))
        .style(move |theme| {
            if selected {
                active_tab_line(theme)
            } else {
                container::Style::default()
            }
        });
    column![
        button(
            container(ui_text(title, 13, tokyo_text()))
                .width(Length::Fill)
                .align_x(alignment::Horizontal::Center),
        )
        .padding([8, 6])
        .width(Length::Fill)
        .style(tab_style(selected, focused))
        .on_press(Message::PrinterPropertiesTabSelected(tab)),
        line,
    ]
    .width(Length::FillPortion(1))
    .spacing(2)
    .into()
}

fn display_error(properties: &PrinterPropertiesDialog) -> Option<String> {
    properties.error.clone().or_else(|| {
        properties
            .sheet
            .as_ref()
            .and_then(|sheet| sheet.provider_error.clone())
    })
}
