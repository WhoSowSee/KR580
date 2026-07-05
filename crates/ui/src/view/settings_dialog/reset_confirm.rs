use iced::widget::{Space, button, column, container, mouse_area, opaque, row, stack};
use iced::{Element, Length};

use super::super::theme::{tokyo_text, ui_text};
use super::styles::{footer_button_style, modal_backdrop_style, modal_dialog_style};
use crate::app::{Message, ResetConfirmFocus};
use crate::i18n::{Key, Lang};

pub(super) fn reset_confirm_overlay(
    focus: ResetConfirmFocus,
    lang: Lang,
) -> Element<'static, Message> {
    let backdrop = mouse_area(
        container(Space::new())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(modal_backdrop_style),
    )
    .on_press(Message::SettingsResetCancelled);

    let cancel =
        button(container(ui_text(lang.t(Key::DiscardCancel), 13, tokyo_text())).padding([6, 16]))
            .on_press(Message::SettingsResetCancelled)
            .padding(0)
            .style(move |_theme, status| {
                footer_button_style(status, focus == ResetConfirmFocus::Cancel)
            });

    let reset = button(
        container(ui_text(
            lang.t(Key::SettingsResetConfirmAction),
            13,
            tokyo_text(),
        ))
        .padding([6, 16]),
    )
    .on_press(Message::SettingsResetConfirmed)
    .padding(0)
    .style(move |_theme, status| footer_button_style(status, focus == ResetConfirmFocus::Confirm));

    let buttons = row![
        Space::new().width(Length::Fill),
        cancel,
        Space::new().width(Length::Fixed(8.0)),
        reset,
    ]
    .width(Length::Fill);

    let card = container(
        column![
            ui_text(lang.t(Key::SettingsResetConfirmTitle), 16, tokyo_text()),
            Space::new().height(Length::Fixed(8.0)),
            ui_text(lang.t(Key::SettingsResetConfirmBody), 13, tokyo_text()),
            Space::new().height(Length::Fixed(16.0)),
            buttons,
        ]
        .width(Length::Fixed(360.0)),
    )
    .padding(16)
    .style(modal_dialog_style);

    let centred = column![
        Space::new().height(Length::Fill),
        row![
            Space::new().width(Length::Fill),
            opaque(card),
            Space::new().width(Length::Fill),
        ]
        .width(Length::Fill),
        Space::new().height(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill);

    stack![opaque(backdrop), centred]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
