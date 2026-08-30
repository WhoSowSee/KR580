use super::super::window_chrome;
use super::super::{Message, style, widgets};
use iced::widget::{Space, column, container, row, svg, text};
use iced::{Alignment, Element, Length, alignment};

const WIDTH: f32 = 164.0;
const TOP_SPACER: f32 = 32.0;
const CHIP_WIDTH: f32 = 116.0;
const CHIP_HEIGHT: f32 = 94.0;
const BUS_NUMBER_WIDTH: f32 = 48.0;

pub(super) fn rail() -> Element<'static, Message> {
    let body = column![
        Space::new().height(Length::Fixed(TOP_SPACER)),
        chip_mark(),
        text("КР580")
            .font(style::FONT_BOLD)
            .size(27)
            .color(style::TEXT)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center),
        widgets::horizontal_rule(),
        metadata(env!("CARGO_PKG_VERSION").to_owned()),
        metadata(window_chrome::platform_label()),
        widgets::horizontal_rule(),
        bus_table(),
        Space::new().height(Length::Fill),
    ]
    .spacing(16)
    .padding(16)
    .align_x(Alignment::Start);

    container(body)
        .width(Length::Fixed(WIDTH))
        .height(Length::Fill)
        .into()
}

fn chip_mark() -> Element<'static, Message> {
    container(
        svg(window_chrome::CHIP.clone())
            .width(Length::Fixed(CHIP_WIDTH))
            .height(Length::Fixed(CHIP_HEIGHT))
            .style(|_theme, _status| svg::Style {
                color: Some(style::TEXT),
            }),
    )
    .width(Length::Fill)
    .align_x(alignment::Horizontal::Center)
    .into()
}

fn metadata(value: String) -> Element<'static, Message> {
    text(value)
        .font(style::MONO_FONT)
        .size(12)
        .color(style::MUTED)
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .into()
}

fn bus_table() -> Element<'static, Message> {
    container(
        column![
            bus_row("01", "ADDR"),
            widgets::horizontal_rule(),
            bus_row("02", "DATA"),
            widgets::horizontal_rule(),
            bus_row("03", "CTRL"),
            widgets::horizontal_rule(),
            bus_row("04", "INT")
        ]
        .spacing(0),
    )
    .style(style::group_frame)
    .width(Length::Fill)
    .into()
}

fn bus_row(number: &'static str, label: &'static str) -> Element<'static, Message> {
    container(
        row![
            container(
                text(number)
                    .font(style::MONO_FONT)
                    .size(11)
                    .color(style::GREEN),
            )
            .width(Length::Fixed(BUS_NUMBER_WIDTH))
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center),
            widgets::vertical_rule(),
            bus_label_cell(label),
        ]
        .spacing(0)
        .align_y(Alignment::Center),
    )
    .height(Length::Fixed(28.0))
    .width(Length::Fill)
    .into()
}

fn bus_label_cell(label: &'static str) -> Element<'static, Message> {
    container(
        row![
            text("●").font(style::FONT).size(10).color(style::GREEN),
            text(label)
                .font(style::MONO_FONT)
                .size(10)
                .color(style::MUTED),
        ]
        .spacing(16)
        .align_y(Alignment::Center),
    )
    .padding(iced::Padding::ZERO.left(12))
    .height(Length::Fill)
    .width(Length::Fill)
    .align_y(alignment::Vertical::Center)
    .into()
}
