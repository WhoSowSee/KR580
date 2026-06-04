//! Monitor (port 00h) inspection window.

mod canvas;
mod hex_popup;
mod sections;
mod styles;

use std::time::Duration;

use iced::widget::{Space, button, column, container, mouse_area, opaque, row, stack, svg};
use iced::{Element, Length};
use k580_app::MonitorState;

use crate::app::{HexStreamFilter, Message};
use crate::i18n::{Key, Lang};
use crate::view::icons;
use crate::view::theme::TOKYO_TEXT;
use crate::view::tooltips::{hover_tooltip, shortcut_hint};

use hex_popup::hex_popup_overlay;
use sections::{pixel_layer_section, text_layer_section, unified_screen_section};
use styles::{
    ICON_BUTTON_SIZE, ICON_GLYPH_SIZE, MODAL_MARGIN, backdrop_style, dialog_style,
    icon_button_style,
};

pub(in crate::view) fn monitor_window_overlay<'a>(
    state: &'a MonitorState,
    split: bool,
    hex_popup: bool,
    hex_filter: HexStreamFilter,
    hex_reveal: bool,
    lang: Lang,
) -> Element<'a, Message> {
    let backdrop = mouse_area(
        container(Space::new())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(backdrop_style),
    )
    .on_press(Message::CloseMonitor);

    let header = monitor_header(split, lang);

    let layer_section: Element<'_, Message> = if split {
        column![
            container(pixel_layer_section(state, lang))
                .width(Length::Fill)
                .height(Length::FillPortion(3)),
            Space::new().height(Length::Fixed(12.0)),
            container(text_layer_section(state, lang))
                .width(Length::Fill)
                .height(Length::FillPortion(1)),
        ]
        .height(Length::Fill)
        .into()
    } else {
        unified_screen_section(state, lang)
    };

    let body = column![
        header,
        Space::new().height(Length::Fixed(12.0)),
        layer_section,
    ]
    .width(Length::Fill)
    .height(Length::Fill);

    let dialog = container(body)
        .padding(16)
        .style(dialog_style)
        .width(Length::Fill)
        .height(Length::Fill);

    let inset_dialog = column![
        Space::new().height(Length::Fixed(MODAL_MARGIN)),
        row![
            Space::new().width(Length::Fixed(MODAL_MARGIN)),
            opaque(dialog),
            Space::new().width(Length::Fixed(MODAL_MARGIN)),
        ]
        .height(Length::Fill),
        Space::new().height(Length::Fixed(MODAL_MARGIN)),
    ]
    .width(Length::Fill)
    .height(Length::Fill);

    let monitor_layer: Element<'_, Message> = stack![opaque(backdrop), inset_dialog]
        .width(Length::Fill)
        .height(Length::Fill)
        .into();

    if hex_popup {
        stack![
            monitor_layer,
            hex_popup_overlay(state, hex_filter, hex_reveal, lang)
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    } else {
        monitor_layer
    }
}

fn monitor_header<'a>(split: bool, lang: Lang) -> Element<'a, Message> {
    let toggle_tooltip = if split {
        Key::MonitorViewUnified
    } else {
        Key::MonitorViewSplit
    };
    let toggle_icon = if split {
        icons::square_merge_vertical()
    } else {
        icons::square_split_vertical()
    };

    row![
        Space::new().width(Length::Fill),
        icon_button(
            toggle_icon,
            Message::ToggleMonitorSplit,
            lang.t(toggle_tooltip),
            None,
        ),
        Space::new().width(Length::Fixed(6.0)),
        icon_button(
            icons::binary(),
            Message::ToggleMonitorHexPopup,
            lang.t(Key::MonitorHexBuffer),
            None,
        ),
        Space::new().width(Length::Fixed(6.0)),
        icon_button(
            icons::brush_cleaning(),
            Message::ClearMonitorBuffer,
            lang.t(Key::MonitorClearBuffer),
            None,
        ),
        Space::new().width(Length::Fixed(6.0)),
        icon_button(
            icons::image(),
            Message::SaveMonitorImage,
            lang.t(Key::MonitorSaveImage),
            None,
        ),
        Space::new().width(Length::Fixed(6.0)),
        icon_button(
            icons::window_close(),
            Message::CloseMonitor,
            lang.t(Key::MonitorClose),
            shortcut_hint(&Message::CloseMonitor),
        ),
    ]
    .align_y(iced::alignment::Vertical::Center)
    .spacing(0)
    .into()
}

fn icon_button(
    handle: svg::Handle,
    on_press: Message,
    hint: &'static str,
    shortcut: Option<&'static str>,
) -> Element<'static, Message> {
    let glyph = svg(handle)
        .width(Length::Fixed(ICON_GLYPH_SIZE))
        .height(Length::Fixed(ICON_GLYPH_SIZE))
        .style(|_theme, _status| svg::Style {
            color: Some(TOKYO_TEXT),
        });

    let face = button(
        container(glyph)
            .padding(0)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center),
    )
    .on_press(on_press)
    .padding(0)
    .width(Length::Fixed(ICON_BUTTON_SIZE))
    .height(Length::Fixed(ICON_BUTTON_SIZE))
    .style(|_theme, status| icon_button_style(status));

    hover_tooltip(
        face.into(),
        hint,
        shortcut,
        iced::widget::tooltip::Position::Bottom,
        Duration::from_millis(450),
    )
}
