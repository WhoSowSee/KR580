//! Monitor (port 00h) inspection window.

mod canvas;
mod hex_popup;
mod sections;
mod styles;

use std::time::Duration;

use crate::backend::MonitorState;
use iced::widget::{Space, button, column, container, mouse_area, opaque, row, stack, svg};
use iced::{Element, Length};

use crate::app::{HexStreamFilter, Message, ToolWindowKind};
use crate::i18n::{Key, Lang};
use crate::view::icons;
use crate::view::theme::{tokyo_blue, tokyo_device_accent, tokyo_text};
use crate::view::tooltips::hover_tooltip;

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

    let body = monitor_content(state, split, false, false, hex_popup, lang);

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

pub(in crate::view) fn monitor_window<'a>(
    state: &'a MonitorState,
    split: bool,
    hex_popup: bool,
    hex_filter: HexStreamFilter,
    hex_reveal: bool,
    always_on_top: bool,
    lang: Lang,
) -> Element<'a, Message> {
    let body = container(monitor_content(
        state,
        split,
        true,
        always_on_top,
        hex_popup,
        lang,
    ))
    .padding(16)
    .style(dialog_style)
    .width(Length::Fill)
    .height(Length::Fill);
    if hex_popup {
        stack![body, hex_popup_overlay(state, hex_filter, hex_reveal, lang)]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    } else {
        body.into()
    }
}

fn monitor_content<'a>(
    state: &'a MonitorState,
    split: bool,
    detached: bool,
    always_on_top: bool,
    hex_popup: bool,
    lang: Lang,
) -> Element<'a, Message> {
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
    column![
        monitor_header(split, detached, always_on_top, hex_popup, lang),
        Space::new().height(Length::Fixed(12.0)),
        layer_section,
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn monitor_header<'a>(
    split: bool,
    detached: bool,
    always_on_top: bool,
    hex_popup: bool,
    lang: Lang,
) -> Element<'a, Message> {
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

    let title = container(Space::new())
        .width(Length::Fill)
        .height(Length::Fixed(ICON_BUTTON_SIZE));
    let drag_handle: Element<'_, Message> = if detached {
        mouse_area(title)
            .on_press(Message::ToolWindowDragStart(ToolWindowKind::Monitor))
            .into()
    } else {
        title.into()
    };
    let window_toggle = if detached {
        icon_button(
            icons::panel_attach(),
            Message::AttachToolWindow(ToolWindowKind::Monitor),
            lang.t(Key::MonitorAttach),
            None,
            false,
        )
    } else {
        icon_button(
            icons::panel_detach(),
            Message::DetachToolWindow(ToolWindowKind::Monitor),
            lang.t(Key::MonitorDetach),
            None,
            false,
        )
    };
    let pin_toggle: Element<'_, Message> = if detached {
        row![
            icon_button(
                icons::pin(),
                Message::ToggleToolWindowAlwaysOnTop(ToolWindowKind::Monitor),
                lang.t(if always_on_top {
                    Key::MonitorUnpin
                } else {
                    Key::MonitorPin
                }),
                None,
                always_on_top,
            ),
            Space::new().width(Length::Fixed(6.0)),
        ]
        .into()
    } else {
        Space::new().width(Length::Shrink).into()
    };

    row![
        drag_handle,
        window_toggle,
        Space::new().width(Length::Fixed(6.0)),
        pin_toggle,
        icon_button(
            toggle_icon,
            Message::ToggleMonitorSplit,
            lang.t(toggle_tooltip),
            None,
            false,
        ),
        Space::new().width(Length::Fixed(6.0)),
        icon_button(
            icons::binary(),
            Message::ToggleMonitorHexPopup,
            lang.t(Key::MonitorHexBuffer),
            None,
            hex_popup,
        ),
        Space::new().width(Length::Fixed(6.0)),
        icon_button(
            icons::brush_cleaning(),
            Message::ClearMonitorBuffer,
            lang.t(Key::MonitorClearBuffer),
            None,
            false,
        ),
        Space::new().width(Length::Fixed(6.0)),
        icon_button(
            icons::image(),
            Message::SaveMonitorImage,
            lang.t(Key::MonitorSaveImage),
            None,
            false,
        ),
        Space::new().width(Length::Fixed(6.0)),
        icon_button(
            icons::window_close(),
            Message::CloseMonitor,
            lang.t(Key::MonitorClose),
            Some("Esc".to_owned()),
            false,
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
    shortcut: Option<String>,
    active: bool,
) -> Element<'static, Message> {
    let glyph_color = if active {
        tokyo_device_accent(tokyo_blue())
    } else {
        tokyo_device_accent(tokyo_text())
    };
    let glyph = svg(handle)
        .width(Length::Fixed(ICON_GLYPH_SIZE))
        .height(Length::Fixed(ICON_GLYPH_SIZE))
        .style(move |_theme, _status| svg::Style {
            color: Some(glyph_color),
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
    .style(move |_theme, status| icon_button_style(status, active));

    hover_tooltip(
        face.into(),
        hint,
        shortcut,
        iced::widget::tooltip::Position::Bottom,
        Duration::from_millis(450),
    )
}
