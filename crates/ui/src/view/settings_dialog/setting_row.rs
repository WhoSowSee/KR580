use iced::widget::{Space, column, container, row};
use iced::{Element, Length, alignment};

use super::super::theme::{TOKYO_BORDER, TOKYO_MUTED, TOKYO_TEXT, ui_text};
use super::consts::LABEL_COLUMN_WIDTH;
use crate::app::Message;

/// Two-column setting row: label + hint on the left, control on the
/// right. Fixed-width left column keeps every control on the page on
/// the same vertical axis regardless of label length.
pub(super) fn setting_row<'a>(
    label: &'static str,
    hint: &'static str,
    control: Element<'a, Message>,
) -> Element<'a, Message> {
    let label_column = column![
        ui_text(label, 14, TOKYO_TEXT),
        Space::new().height(Length::Fixed(4.0)),
        ui_text(hint, 11, TOKYO_MUTED),
    ]
    .width(Length::Fixed(LABEL_COLUMN_WIDTH));

    row![
        label_column,
        Space::new().width(Length::Fixed(20.0)),
        container(control)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Right),
    ]
    .align_y(alignment::Vertical::Top)
    .into()
}

pub(super) fn separator_horizontal() -> Element<'static, Message> {
    container(Space::new())
        .width(Length::Fill)
        .height(Length::Fixed(1.0))
        .style(|_theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(iced::Color {
                a: 0.6,
                ..TOKYO_BORDER
            })),
            ..iced::widget::container::Style::default()
        })
        .into()
}

pub(super) fn separator_vertical() -> Element<'static, Message> {
    container(Space::new())
        .width(Length::Fixed(1.0))
        .height(Length::Fill)
        .style(|_theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(iced::Color {
                a: 0.6,
                ..TOKYO_BORDER
            })),
            ..iced::widget::container::Style::default()
        })
        .into()
}
