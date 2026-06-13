use iced::widget::{Space, button, column, container, mouse_area, opaque, row, stack};
use iced::{Element, Length, alignment};
use k580_app::NetworkMode;

use super::icons;
use super::network::{NetworkViewState, center};
use super::storage::chrome::icon_button;
use super::styles::{
    modal_backdrop_style, modal_field_button_style, modal_tab_button_style, panel_style,
};
use super::theme::{TOKYO_MUTED, TOKYO_RED, TOKYO_TEXT, ui_text};
use super::widgets::{modal_footer_button, text_input_shell};
use crate::app::Message;
use crate::i18n::{Key, NetworkKey};

pub(super) fn settings_overlay(view: NetworkViewState<'_>) -> Element<'_, Message> {
    let backdrop = mouse_area(
        container(Space::new())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(modal_backdrop_style),
    )
    .on_press(Message::CloseNetworkSettings);
    let error: Element<'_, Message> = view.error.map_or_else(
        || Space::new().height(Length::Fixed(16.0)).into(),
        |error| ui_text(error, 12, TOKYO_RED).into(),
    );
    let dialog = container(
        column![
            row![
                ui_text(
                    view.lang.t(Key::Network(NetworkKey::SettingsTitle)),
                    16,
                    TOKYO_TEXT
                ),
                Space::new().width(Length::Fill),
                icon_button(
                    icons::window_close(),
                    Some(Message::CloseNetworkSettings),
                    view.lang.t(Key::MonitorClose),
                    false,
                    None,
                ),
            ]
            .align_y(alignment::Vertical::Center),
            row![
                ui_text(view.lang.t(Key::Network(NetworkKey::Mode)), 13, TOKYO_MUTED),
                mode_button(
                    view.lang.t(Key::Network(NetworkKey::ModeClient)),
                    NetworkMode::Client,
                    view.mode == NetworkMode::Client,
                ),
                mode_button(
                    view.lang.t(Key::Network(NetworkKey::ModeServer)),
                    NetworkMode::Server,
                    view.mode == NetworkMode::Server,
                ),
            ]
            .spacing(8)
            .align_y(alignment::Vertical::Center),
            row![
                column![
                    ui_text(view.lang.t(Key::Network(NetworkKey::Host)), 12, TOKYO_MUTED),
                    text_input_shell(
                        "127.0.0.1",
                        view.host,
                        Message::NetworkHostChanged,
                        Length::Fill,
                    ),
                ]
                .spacing(4)
                .width(Length::Fill),
                column![
                    ui_text(view.lang.t(Key::Network(NetworkKey::Port)), 12, TOKYO_MUTED),
                    text_input_shell("5800", view.port, Message::NetworkPortChanged, Length::Fill,),
                ]
                .spacing(4)
                .width(Length::Fixed(110.0)),
            ]
            .spacing(10),
            error,
            row![
                Space::new().width(Length::Fill),
                modal_footer_button(
                    view.lang.t(Key::Network(NetworkKey::Cancel)),
                    Message::CloseNetworkSettings,
                    modal_field_button_style,
                ),
                modal_footer_button(
                    view.lang.t(Key::Network(NetworkKey::Apply)),
                    Message::ApplyNetworkSettings,
                    modal_field_button_style,
                ),
            ]
            .spacing(8),
        ]
        .spacing(12),
    )
    .padding(16)
    .width(Length::Fixed(440.0))
    .style(panel_style);

    stack![opaque(backdrop), center(opaque(dialog))]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn mode_button(
    label: &'static str,
    mode: NetworkMode,
    selected: bool,
) -> Element<'static, Message> {
    button(container(ui_text(label, 13, TOKYO_TEXT)).padding([6, 14]))
        .on_press(Message::NetworkModeChanged(mode))
        .padding(0)
        .style(move |_theme, status| modal_tab_button_style(status, selected))
        .into()
}
