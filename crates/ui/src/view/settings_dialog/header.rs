use iced::widget::{Space, button, container, row, svg};
use iced::{Element, Length, alignment};

use super::super::icons;
use super::super::theme::{tokyo_text, ui_text};
use super::consts::HEADER_HEIGHT;
use super::styles::header_close_button_style;
use crate::app::Message;
use crate::i18n::{Key, Lang};

pub(super) fn settings_header(lang: Lang) -> Element<'static, Message> {
    let title = ui_text(lang.t(Key::SettingsTitle), 16, tokyo_text());

    let close_glyph = svg(icons::window_close())
        .width(Length::Fixed(14.0))
        .height(Length::Fixed(14.0))
        .style(|_theme, _status| svg::Style {
            color: Some(tokyo_text()),
        });

    let close_button = button(
        container(close_glyph)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center),
    )
    .on_press(Message::CloseSettings)
    .padding(0)
    .width(Length::Fixed(28.0))
    .height(Length::Fixed(28.0))
    .style(|_theme, status| header_close_button_style(status));

    container(
        row![title, Space::new().width(Length::Fill), close_button,]
            .align_y(alignment::Vertical::Center),
    )
    .padding(iced::Padding {
        top: 0.0,
        right: 16.0,
        bottom: 0.0,
        left: 20.0,
    })
    .width(Length::Fill)
    .height(Length::Fixed(HEADER_HEIGHT))
    .align_y(alignment::Vertical::Center)
    .into()
}
