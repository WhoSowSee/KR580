//! Floating opcode picker that drops out of a memory row.
//!
//! Lives in its own module because the dropdown needs a couple of helpers
//! (`OpcodeChoice`, the search filter) that only it cares about, and
//! gluing them into the memory list module would obscure the row layout.

use iced::widget::{Column, Space, button, column, container, opaque, row, scrollable, text_input};
use iced::{Element, Length, alignment};

use super::styles::{
    input_borderless_style, opcode_dropdown_style, opcode_option_style, scrollable_style,
};
use super::theme::{MONO_FONT, mono_text, tokyo_green, tokyo_text};
use super::utils::row_separator;
use crate::app::{Message, OPCODE_SEARCH_INPUT_ID, OpcodeChoice, filtered_opcode_choices};
use crate::i18n::{Key, Lang};

pub(super) const OPCODE_DROPDOWN_HEIGHT: f32 = 224.0;

pub(super) fn opcode_dropdown_overlay<'a>(
    address: u16,
    search: &'a str,
    highlighted: usize,
    reveal: bool,
    top: f32,
    lang: Lang,
) -> Element<'a, Message> {
    column![
        Space::new().height(Length::Fixed(top)),
        row![
            Space::new().width(Length::Fill),
            opaque(opcode_dropdown(address, search, highlighted, reveal, lang)),
            Space::new().width(Length::Fixed(24.0)),
        ]
        .width(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn opcode_dropdown<'a>(
    address: u16,
    search: &'a str,
    highlighted: usize,
    reveal: bool,
    lang: Lang,
) -> Element<'a, Message> {
    let mut options = Column::new().spacing(0);

    for (index, choice) in filtered_opcode_choices(search).into_iter().enumerate() {
        options = options.push(opcode_option(address, choice, index == highlighted));
    }

    let content = column![
        text_input(lang.t(Key::OpcodeSearchPlaceholder), search)
            .id(OPCODE_SEARCH_INPUT_ID)
            .on_input(Message::OpcodeSearchChanged)
            .font(MONO_FONT)
            .size(13)
            .padding(6)
            .width(Length::Fill)
            .style(input_borderless_style),
        row_separator(),
        scrollable(options)
            .height(Length::Fixed(172.0))
            .style(move |theme, status| scrollable_style(reveal, theme, status))
            .on_scroll(|_| Message::OpcodeScrolled),
    ]
    .spacing(4);

    container(content)
        .padding(6)
        .width(Length::Fixed(226.0))
        .style(opcode_dropdown_style)
        .into()
}

fn opcode_option(
    address: u16,
    choice: OpcodeChoice,
    highlighted: bool,
) -> Element<'static, Message> {
    button(
        row![
            mono_text(format!("{:02X}", choice.value), 13, tokyo_green())
                .width(Length::Fixed(34.0)),
            mono_text(choice.mnemonic, 13, tokyo_text()).width(Length::Fill),
        ]
        .spacing(8)
        .align_y(alignment::Vertical::Center),
    )
    .on_press(Message::OpcodeSelected(address, choice.value))
    .padding(5)
    .width(Length::Fill)
    .style(move |_theme, status| opcode_option_style(status, highlighted))
    .into()
}
