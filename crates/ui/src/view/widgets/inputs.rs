use iced::widget::{container, text_input};
use iced::{Element, Length, Padding, alignment};

use super::super::styles::{input_borderless_style, input_shell_style};
use super::super::theme::MONO_FONT;
use crate::app::Message;

const COMPACT_INPUT_FONT_SIZE: u32 = 13;

pub(in crate::view) fn text_input_shell<'a>(
    placeholder: &'static str,
    value: &'a str,
    on_input: fn(String) -> Message,
    width: Length,
) -> Element<'a, Message> {
    text_input_shell_with_metrics(
        placeholder,
        value,
        on_input,
        width,
        16,
        Padding {
            top: 6.0,
            right: 9.0,
            bottom: 6.0,
            left: 9.0,
        },
        Length::Shrink,
    )
}

pub(in crate::view) fn compact_text_input_shell<'a>(
    placeholder: &'static str,
    value: &'a str,
    on_input: fn(String) -> Message,
    width: Length,
) -> Element<'a, Message> {
    text_input_shell_with_metrics(
        placeholder,
        value,
        on_input,
        width,
        COMPACT_INPUT_FONT_SIZE,
        Padding {
            top: 3.0,
            right: 8.0,
            bottom: 3.0,
            left: 8.0,
        },
        Length::Fixed(30.0),
    )
}

fn text_input_shell_with_metrics<'a>(
    placeholder: &'static str,
    value: &'a str,
    on_input: fn(String) -> Message,
    width: Length,
    font_size: u32,
    padding: Padding,
    height: Length,
) -> Element<'a, Message> {
    container(
        text_input(placeholder, value)
            .on_input(on_input)
            .font(MONO_FONT)
            .size(font_size)
            .padding(padding)
            .width(Length::Fill)
            .style(input_borderless_style),
    )
    .width(width)
    .height(height)
    .align_y(alignment::Vertical::Center)
    .style(|theme| input_shell_style(theme, false))
    .into()
}
