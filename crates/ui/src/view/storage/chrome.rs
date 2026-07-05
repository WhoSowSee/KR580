use std::time::Duration;

use iced::widget::{Space, button, container, mouse_area, row, svg};
use iced::{Background, Border, Color, Element, Length, Theme, alignment};

use super::super::icons;
use super::super::theme::{
    tokyo_blue, tokyo_board, tokyo_border, tokyo_device_accent, tokyo_modal_backdrop, tokyo_muted,
    tokyo_selection_blue, tokyo_surface, tokyo_surface_2, tokyo_text,
};
use super::super::tooltips::hover_tooltip;
use crate::app::{Message, ToolWindowKind};
use crate::i18n::{Key, Lang};

const ICON_BUTTON_SIZE: f32 = 32.0;
const ICON_GLYPH_SIZE: f32 = 18.0;

pub(in crate::view) fn device_backdrop_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color {
            a: 0.85,
            ..tokyo_modal_backdrop()
        })),
        ..container::Style::default()
    }
}

pub(in crate::view) fn device_buffer_style(_theme: &Theme) -> container::Style {
    container::Style {
        text_color: Some(tokyo_text()),
        background: Some(Background::Color(tokyo_board())),
        border: Border {
            radius: 4.0.into(),
            width: 1.0,
            color: tokyo_border(),
        },
        ..container::Style::default()
    }
}

pub(in crate::view) fn window_controls(
    kind: ToolWindowKind,
    detached: bool,
    always_on_top: bool,
    lang: Lang,
) -> Element<'static, Message> {
    let title = container(Space::new())
        .width(Length::Fill)
        .height(Length::Fixed(ICON_BUTTON_SIZE));
    let drag_handle: Element<'_, Message> = if detached {
        mouse_area(title)
            .on_press(Message::ToolWindowDragStart(kind))
            .into()
    } else {
        title.into()
    };
    let window_toggle = icon_button(
        if detached {
            icons::panel_attach()
        } else {
            icons::panel_detach()
        },
        Some(if detached {
            Message::AttachToolWindow(kind)
        } else {
            Message::DetachToolWindow(kind)
        }),
        lang.t(if detached {
            Key::MonitorAttach
        } else {
            Key::MonitorDetach
        }),
        false,
        None,
    );
    let pin: Element<'_, Message> = if detached {
        row![
            icon_button(
                icons::pin(),
                Some(Message::ToggleToolWindowAlwaysOnTop(kind)),
                lang.t(if always_on_top {
                    Key::MonitorUnpin
                } else {
                    Key::MonitorPin
                }),
                always_on_top,
                None,
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
        pin,
    ]
    .width(Length::Fill)
    .align_y(alignment::Vertical::Center)
    .into()
}

pub(in crate::view) fn icon_button(
    handle: svg::Handle,
    on_press: Option<Message>,
    hint: &'static str,
    active: bool,
    shortcut: Option<String>,
) -> Element<'static, Message> {
    let is_disabled = on_press.is_none() && !active;
    let glyph_color = if active {
        tokyo_device_accent(tokyo_blue())
    } else if is_disabled {
        tokyo_muted()
    } else {
        tokyo_device_accent(tokyo_text())
    };
    let glyph = svg(handle)
        .width(Length::Fixed(ICON_GLYPH_SIZE))
        .height(Length::Fixed(ICON_GLYPH_SIZE))
        .style(move |_theme, _status| svg::Style {
            color: Some(glyph_color),
        });

    let mut button = button(
        container(glyph)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center),
    );
    if let Some(message) = on_press {
        button = button.on_press(message);
    }
    let face = button
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

fn icon_button_style(status: button::Status, active: bool) -> button::Style {
    let disabled = matches!(status, button::Status::Disabled) && !active;
    let background = match (status, active) {
        (button::Status::Pressed, _) if !disabled => tokyo_surface_2(),
        (button::Status::Hovered, _) if !disabled => tokyo_surface(),
        (_, true) if !disabled => tokyo_selection_blue(),
        _ => tokyo_board(),
    };
    let border_color = if disabled {
        Color {
            a: 0.35,
            ..tokyo_border()
        }
    } else if active {
        tokyo_device_accent(tokyo_blue())
    } else {
        tokyo_border()
    };
    let text_color = if disabled {
        tokyo_muted()
    } else {
        tokyo_text()
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color,
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: border_color,
        },
        ..button::Style::default()
    }
}
