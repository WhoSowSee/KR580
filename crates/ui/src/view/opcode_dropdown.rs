//! Floating opcode picker that drops out of a memory row.

use iced::widget::{Space, button, column, container, opaque, row, scrollable, stack, text_input};
use iced::{Element, Length, alignment};

use super::styles::{input_borderless_style, opcode_dropdown_style, opcode_option_style};
use super::theme::{MONO_FONT, mono_text, tokyo_green, tokyo_text};
use super::utils::row_separator;
use super::widgets::compact_scrollbar;
use crate::app::{
    Message, OPCODE_SCROLL_ID, OPCODE_SEARCH_INPUT_ID, OpcodeChoice, filtered_opcode_choices,
};
use crate::i18n::{Key, Lang};

pub(super) const OPCODE_DROPDOWN_HEIGHT: f32 = 224.0;
const OPCODE_LIST_HEIGHT: f32 = 172.0;
const OPCODE_OPTION_HEIGHT: f32 = 27.0;

pub(super) fn opcode_dropdown_overlay<'a>(
    address: u16,
    search: &'a str,
    highlighted: usize,
    scroll_offset: f32,
    reveal: bool,
    top: f32,
    lang: Lang,
) -> Element<'a, Message> {
    column![
        Space::new().height(Length::Fixed(top)),
        row![
            Space::new().width(Length::Fill),
            opaque(opcode_dropdown(
                address,
                search,
                highlighted,
                scroll_offset,
                reveal,
                lang,
            )),
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
    scroll_offset: f32,
    reveal: bool,
    lang: Lang,
) -> Element<'a, Message> {
    let choices = filtered_opcode_choices(search);
    let max_offset = choices.len() as f32 * OPCODE_OPTION_HEIGHT - OPCODE_LIST_HEIGHT;
    let options = column(
        choices
            .into_iter()
            .enumerate()
            .map(|(index, choice)| opcode_option(address, choice, index == highlighted)),
    );

    let options = scrollable(options)
        .id(OPCODE_SCROLL_ID)
        .height(Length::Fixed(OPCODE_LIST_HEIGHT))
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::hidden(),
        ))
        .on_scroll(|viewport| Message::OpcodeScrolled(viewport.absolute_offset().y));
    let options = stack![
        options,
        compact_scrollbar(
            scroll_offset,
            max_offset,
            reveal,
            Message::OpcodeScrollbarDragged,
        ),
    ]
    .width(Length::Fill)
    .height(Length::Fixed(OPCODE_LIST_HEIGHT));

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
        options,
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
    .height(Length::Fixed(OPCODE_OPTION_HEIGHT))
    .style(move |_theme, status| opcode_option_style(status, highlighted))
    .into()
}
