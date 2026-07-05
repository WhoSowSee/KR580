use iced::widget::{Space, button, column, container, row, stack, text_input};
use iced::{Element, Length, Padding, alignment};

use super::super::styles::{input_borderless_style, input_shell_style};
use super::super::theme::{MONO_FONT, mono_text, tokyo_green, tokyo_text, ui_text};
use super::styles::{
    checkbox_style, checklist_button_style, flag_checkbox_style, group_label_style,
    group_panel_style,
};
use crate::app::Message;

const CHECKBOX_SIZE: f32 = 16.0;
const FLAG_CHECKBOX_SIZE: f32 = 16.0;

pub(super) fn group_box<'a>(
    title: &'static str,
    content: impl Into<Element<'a, Message>>,
    width: Length,
) -> Element<'a, Message> {
    let panel: Element<'a, Message> = container(content)
        .padding(Padding {
            top: 18.0,
            right: 12.0,
            bottom: 12.0,
            left: 12.0,
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .style(group_panel_style)
        .into();
    let label = row![
        Space::new().width(Length::Fill),
        container(ui_text(title, 14, tokyo_text()))
            .padding([0, 6])
            .style(group_label_style),
        Space::new().width(Length::Fill),
    ];

    stack![
        column![Space::new().height(Length::Fixed(9.0)), panel]
            .height(Length::Fill)
            .width(Length::Fill),
        label,
    ]
    .width(width)
    .height(Length::Fill)
    .into()
}

pub(super) fn checkbox_row(
    label_text: &'static str,
    checked: bool,
    message: Message,
) -> Element<'static, Message> {
    let mark: Element<'static, Message> = if checked {
        mono_text("✓", 13, tokyo_green())
            .align_x(alignment::Horizontal::Center)
            .into()
    } else {
        Space::new().into()
    };
    let box_face = container(mark)
        .width(Length::Fixed(CHECKBOX_SIZE))
        .height(Length::Fixed(CHECKBOX_SIZE))
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .style(move |_theme| checkbox_style(checked));

    button(
        row![box_face, ui_text(label_text, 13, tokyo_text())]
            .spacing(8)
            .align_y(alignment::Vertical::Center),
    )
    .on_press(message)
    .padding([2, 4])
    .width(Length::Fill)
    .style(move |_theme, status| checklist_button_style(status))
    .into()
}

pub(super) fn flag_checkbox(
    label_text: &'static str,
    checked: bool,
    message: Message,
) -> Element<'static, Message> {
    let mark: Element<'static, Message> = if checked {
        mono_text("✓", 12, tokyo_green())
            .align_x(alignment::Horizontal::Center)
            .into()
    } else {
        Space::new().into()
    };
    let box_face = container(mark)
        .width(Length::Fixed(FLAG_CHECKBOX_SIZE))
        .height(Length::Fixed(FLAG_CHECKBOX_SIZE))
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .style(move |_theme| flag_checkbox_style(checked));

    button(
        column![box_face, ui_text(label_text, 12, tokyo_text())]
            .spacing(2)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center),
    )
    .on_press(message)
    .padding([2, 4])
    .width(Length::Fill)
    .style(move |_theme, status| checklist_button_style(status))
    .into()
}

pub(super) fn input_shell<'a>(
    value: &'a str,
    width: f32,
    on_input: fn(String) -> Message,
    mono: bool,
) -> Element<'a, Message> {
    let mut input = text_input("", value)
        .on_input(on_input)
        .padding(Padding {
            top: 5.0,
            right: 9.0,
            bottom: 5.0,
            left: 9.0,
        })
        .size(14)
        .align_x(alignment::Horizontal::Center)
        .width(Length::Fill)
        .style(input_borderless_style);

    if mono {
        input = input.font(MONO_FONT);
    }

    container(input)
        .width(Length::Fixed(width))
        .style(|theme| input_shell_style(theme, false))
        .into()
}

pub(super) fn suffix(value: &'static str) -> Element<'static, Message> {
    mono_text(value, 13, tokyo_text()).into()
}

pub(super) fn label(value: &'static str) -> Element<'static, Message> {
    ui_text(value, 13, tokyo_text()).into()
}
