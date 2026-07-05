//! Individual cell builders for the memory list row.

use iced::widget::{Text, container, mouse_area, text_input};
use iced::{Element, Length, Theme, alignment};

use super::super::styles::inline_value_input_style;
use super::super::theme::{MONO_FONT, mono_text, tokyo_text};
use crate::app::{MEMORY_INLINE_INPUT_ID, Message};

const VALUE_GLYPH_WIDTH: f32 = 28.0;

/// `mouse_area`-wrapped cell that fires `on_press`/`on_double_click`
/// with `FillPortion(1)`. Not a `button` because `button::update`
/// captures `ButtonPressed` and an outer `mouse_area` would then go
/// blind to every click.
fn cell_mouse_area<'a>(
    label: Text<'a>,
    on_press: Message,
    on_double_click: Message,
) -> Element<'a, Message> {
    let body = container(
        label
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_y(alignment::Vertical::Center);
    mouse_area(body)
        .on_press(on_press)
        .on_double_click(on_double_click)
        .interaction(iced::mouse::Interaction::Pointer)
        .into()
}

pub(super) fn address_cell(address: u16, accent: iced::Color) -> Element<'static, Message> {
    container(cell_mouse_area(
        mono_text(format!("{address:04X}"), 14, accent),
        Message::MemorySelected(address),
        Message::MemoryReplace(address),
    ))
    .width(Length::FillPortion(1))
    .height(Length::Fill)
    .into()
}

/// Two stacked listeners: an inner `mouse_area` over the mnemonic
/// glyph fires `OpcodeDropdownToggled`, the outer one fires
/// `MemorySelected`. `mouse_area::update` ignores presses an inner
/// widget already captured, so each gesture reaches one listener.
pub(super) fn command_cell(command: String, address: u16) -> Element<'static, Message> {
    let glyph: Element<'static, Message> = mouse_area(mono_text(command, 14, tokyo_text()))
        .on_press(Message::OpcodeDropdownToggled(address))
        .on_double_click(Message::MemoryReplace(address))
        .interaction(iced::mouse::Interaction::Pointer)
        .into();

    let body = container(glyph)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center);

    let cell = mouse_area(body)
        .on_press(Message::MemorySelected(address))
        .on_double_click(Message::MemoryReplace(address));

    container(cell)
        .width(Length::FillPortion(1))
        .height(Length::Fill)
        .into()
}

pub(super) fn memory_value_cell<'a>(
    value: u8,
    address: u16,
    selected: bool,
    value_color: iced::Color,
    inline_value_input: &'a str,
    inline_placeholder: &'a str,
) -> Element<'a, Message> {
    if selected {
        let style = move |theme: &Theme, status: text_input::Status| {
            let mut style = inline_value_input_style(theme, status);
            style.value = value_color;
            style
        };
        let editor: Element<'a, Message> = container(
            text_input(inline_placeholder, inline_value_input)
                .id(MEMORY_INLINE_INPUT_ID)
                .on_input(move |value| Message::InlineMemoryValueChanged(address, value))
                .on_submit(Message::ApplyInlineMemoryValue(address))
                .font(MONO_FONT)
                .size(14)
                .padding(0)
                .align_x(alignment::Horizontal::Center)
                .width(Length::Fixed(VALUE_GLYPH_WIDTH))
                .style(style),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .into();

        let body = mouse_area(editor)
            .on_press(Message::MemorySelected(address))
            .on_double_click(Message::MemoryReplace(address))
            .interaction(iced::mouse::Interaction::Pointer);

        container(body)
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .into()
    } else {
        value_cell_button(value, address, value_color)
    }
}

/// Single-click edits in place; double-click starts with an empty replacement field.
fn value_cell_button(
    value: u8,
    address: u16,
    value_color: iced::Color,
) -> Element<'static, Message> {
    let glyph: Element<'static, Message> =
        mouse_area(mono_text(format!("{value:02X}"), 14, value_color))
            .on_press(Message::MemoryEnter(address))
            .on_double_click(Message::MemoryReplace(address))
            .interaction(iced::mouse::Interaction::Text)
            .into();

    let body = mouse_area(
        container(glyph)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center),
    )
    .on_press(Message::MemorySelected(address))
    .on_double_click(Message::MemoryReplace(address))
    .interaction(iced::mouse::Interaction::Pointer);

    container(body)
        .width(Length::FillPortion(1))
        .height(Length::Fill)
        .into()
}
