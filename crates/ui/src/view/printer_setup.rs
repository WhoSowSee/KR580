mod dropdown;
mod labels;
mod properties;
mod sections;
mod styles;

use iced::widget::{Space, column, container, mouse_area, opaque, row, stack};
use iced::{Alignment, Element, Length};

use super::icons;
use super::styles::{modal_backdrop_style, panel_style};
use super::theme::{tokyo_red, tokyo_text, ui_text};
use super::widgets::modal_icon_button_focused;
use crate::app::{Message, PrinterSetupDialog, PrinterSetupFocus};
use crate::i18n::Lang;
use labels::{Label, label};
use styles::footer_button;

const DIALOG_WIDTH: f32 = 720.0;
const PRINTER_GROUP_HEIGHT: f32 = 196.0;
const SETTINGS_GROUP_HEIGHT: f32 = 130.0;

pub(super) fn with_printer_setup_overlay<'a>(
    base: Element<'a, Message>,
    dialog: Option<&'a PrinterSetupDialog>,
    lang: Lang,
) -> Element<'a, Message> {
    match dialog {
        Some(dialog) => stack![base, printer_setup_modal_overlay(dialog, lang)]
            .width(Length::Fill)
            .height(Length::Fill)
            .into(),
        None => base,
    }
}

pub(super) fn printer_setup_modal_overlay<'a>(
    dialog: &'a PrinterSetupDialog,
    lang: Lang,
) -> Element<'a, Message> {
    if let Some(properties) = dialog.properties.as_ref() {
        return properties::printer_properties_modal_overlay(dialog, properties, lang);
    }
    let backdrop = mouse_area(
        container(Space::new())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(modal_backdrop_style),
    )
    .on_press(Message::ClosePrinterSetup);
    let close = modal_icon_button_focused(
        icons::window_close(),
        Message::ClosePrinterSetup,
        label(lang, Label::Close),
        34.0,
        dialog.focus_is_visible(PrinterSetupFocus::Close),
    );
    let header = row![
        ui_text(label(lang, Label::Title), 18, tokyo_text()),
        Space::new().width(Length::Fill),
        close,
    ]
    .align_y(Alignment::Center);

    let printer_group = sections::group_box(
        label(lang, Label::Printer),
        sections::printer_section(dialog, lang),
        Length::Fill,
    )
    .height(Length::Fixed(PRINTER_GROUP_HEIGHT));
    let settings_groups = row![
        sections::group_box(
            label(lang, Label::Paper),
            sections::paper_section(dialog, lang),
            Length::Fill,
        )
        .width(Length::FillPortion(3))
        .height(Length::Fill),
        sections::group_box(
            label(lang, Label::Orientation),
            sections::orientation_section(dialog, lang),
            Length::Fill,
        )
        .width(Length::FillPortion(2))
        .height(Length::Fill),
    ]
    .spacing(16)
    .height(Length::Fixed(SETTINGS_GROUP_HEIGHT));
    let ready = dialog.configuration.is_some()
        && !dialog.configuration_loading
        && !dialog.properties_pending;
    let footer = row![
        Space::new().width(Length::Fill),
        footer_button(
            label(lang, Label::Cancel),
            true,
            dialog.focus_is_visible(PrinterSetupFocus::Cancel),
        )
        .on_press(Message::ClosePrinterSetup),
        footer_button(
            label(lang, Label::Ok),
            ready,
            dialog.focus_is_visible(PrinterSetupFocus::Ok),
        )
        .on_press_maybe(ready.then_some(Message::PrinterSetupConfirmed)),
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    let error = dialog.error.clone();
    let mut content = column![header, printer_group, settings_groups].spacing(16);
    if let Some(error) = error {
        let error: Element<'a, Message> = ui_text(error, 12, tokyo_red()).into();
        content = content.push(error);
    }
    let panel = container(
        content
            .push(footer)
            .spacing(12)
            .padding(18)
            .width(Length::Fixed(DIALOG_WIDTH)),
    )
    .style(panel_style);

    let centred = column![
        Space::new().height(Length::Fill),
        row![
            Space::new().width(Length::Fill),
            opaque(panel),
            Space::new().width(Length::Fill),
        ],
        Space::new().height(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill);

    stack![opaque(backdrop), centred]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
