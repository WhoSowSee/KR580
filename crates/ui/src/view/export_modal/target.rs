use iced::widget::{Space, button, column, container, opaque, row, svg, text_input};
use iced::{Element, Length, Padding, alignment};

use super::super::icons;
use super::super::styles::{input_borderless_style, input_shell_style};
use super::super::theme::{tokyo_muted, tokyo_text, ui_text};
use super::super::widgets::modal_icon_button;
use super::controls::label;
use super::local_icons;
use super::styles::{combo_arrow_style, dropdown_option_style, dropdown_panel_style};
use crate::app::{ExportTab, Message};
use crate::i18n::{Key, Lang};

const PAGE_LABEL_WIDTH: f32 = 76.0;
const SECTION_LABEL_WIDTH: f32 = 62.0;
const FIELD_WIDTH: f32 = 200.0;
const TARGET_HEIGHT: f32 = 32.0;
const DROPDOWN_OFFSET: f32 = TARGET_HEIGHT + 3.0;
const ARROW_WIDTH: f32 = 28.0;
const ICON_SIZE: f32 = TARGET_HEIGHT;

pub(super) fn target_selector<'a>(
    tab: ExportTab,
    value: &'a str,
    dropdown_open: bool,
    lang: Lang,
) -> Element<'a, Message> {
    let label_key = match tab {
        ExportTab::Xlsx => Key::ExportPageLabel,
        ExportTab::Text => Key::ExportSectionLabel,
    };
    let add_icon = match tab {
        ExportTab::Xlsx => local_icons::file_spreadsheet(),
        ExportTab::Text => local_icons::file_text(),
    };
    let add_tooltip = match tab {
        ExportTab::Xlsx => Key::ExportAddPageTooltip,
        ExportTab::Text => Key::ExportAddSectionTooltip,
    };
    let delete_tooltip = match tab {
        ExportTab::Xlsx => Key::ExportDeletePageTooltip,
        ExportTab::Text => Key::ExportDeleteSectionTooltip,
    };
    let label_width = target_label_width(tab);

    let row = row![
        container(label(lang.t(label_key)))
            .width(Length::Fixed(label_width))
            .height(Length::Fixed(TARGET_HEIGHT))
            .align_y(alignment::Vertical::Center),
        combo_box(value, dropdown_open),
        modal_icon_button(
            add_icon,
            Message::ExportTargetAdd,
            lang.t(add_tooltip),
            ICON_SIZE,
        ),
        modal_icon_button(
            local_icons::trash(),
            Message::ExportTargetDelete,
            lang.t(delete_tooltip),
            ICON_SIZE,
        ),
    ]
    .spacing(6)
    .align_y(alignment::Vertical::Center)
    .height(Length::Fixed(TARGET_HEIGHT));

    row.into()
}

pub(super) fn target_dropdown_overlay(
    tab: ExportTab,
    options: &[String],
    highlighted: Option<usize>,
) -> Element<'static, Message> {
    let label_width = target_label_width(tab);

    column![
        Space::new().height(Length::Fixed(DROPDOWN_OFFSET)),
        row![
            Space::new().width(Length::Fixed(label_width + 6.0)),
            opaque(dropdown(options, highlighted)),
        ]
        .width(Length::Fill),
        Space::new().height(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

pub(super) fn target_row_height() -> f32 {
    TARGET_HEIGHT
}

fn target_label_width(tab: ExportTab) -> f32 {
    match tab {
        ExportTab::Xlsx => PAGE_LABEL_WIDTH,
        ExportTab::Text => SECTION_LABEL_WIDTH,
    }
}

fn combo_box<'a>(value: &'a str, open: bool) -> Element<'a, Message> {
    let input = text_input("", value)
        .on_input(Message::ExportTargetChanged)
        .padding(Padding {
            top: 3.0,
            right: 6.0,
            bottom: 3.0,
            left: 8.0,
        })
        .size(14)
        .width(Length::Fill)
        .style(input_borderless_style);
    let chevron = svg(icons::chevron_down())
        .width(Length::Fixed(14.0))
        .height(Length::Fixed(14.0))
        .style(|_theme, _status| svg::Style {
            color: Some(tokyo_muted()),
        });
    let arrow = button(
        container(chevron)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center),
    )
    .on_press(Message::ExportTargetDropdownToggled)
    .padding(0)
    .width(Length::Fixed(ARROW_WIDTH))
    .height(Length::Fixed(TARGET_HEIGHT))
    .style(move |_theme, status| combo_arrow_style(status, open));

    container(
        row![input, arrow]
            .spacing(0)
            .align_y(alignment::Vertical::Center),
    )
    .width(Length::Fixed(FIELD_WIDTH))
    .height(Length::Fixed(TARGET_HEIGHT))
    .style(|theme| input_shell_style(theme, false))
    .into()
}

fn dropdown(options: &[String], highlighted: Option<usize>) -> Element<'static, Message> {
    let mut list = column![].spacing(0);
    for (index, option) in options.iter().enumerate() {
        list = list.push(dropdown_option(option.clone(), highlighted == Some(index)));
    }
    container(list)
        .padding(4)
        .width(Length::Fixed(FIELD_WIDTH))
        .style(dropdown_panel_style)
        .into()
}

fn dropdown_option(label_text: String, highlighted: bool) -> Element<'static, Message> {
    let message_value = label_text.clone();
    button(
        container(ui_text(label_text, 13, tokyo_text()))
            .padding([6, 9])
            .width(Length::Fill),
    )
    .on_press(Message::ExportTargetSelected(message_value))
    .padding(0)
    .width(Length::Fill)
    .style(move |_theme, status| dropdown_option_style(status, highlighted))
    .into()
}
