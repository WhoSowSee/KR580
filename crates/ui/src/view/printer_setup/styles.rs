use iced::widget::{Space, button, container, radio};
use iced::{Background, Border, Color, Element, Length, alignment};

use super::super::styles::modal_field_button_style;
use super::super::theme::{
    tokyo_blue, tokyo_board, tokyo_border, tokyo_muted, tokyo_surface, tokyo_text, ui_text,
};
use crate::app::Message;

pub(super) fn footer_button(
    label: &'static str,
    enabled: bool,
    focused: bool,
) -> button::Button<'static, Message> {
    let color = if enabled { tokyo_text() } else { tokyo_muted() };
    button(
        container(ui_text(label, 14, color))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center),
    )
    .width(Length::Fixed(118.0))
    .height(Length::Fixed(36.0))
    .padding(0)
    .style(move |_theme, status| {
        let mut style = modal_field_button_style(status);
        if focused {
            style.border.color = tokyo_blue();
        }
        style
    })
}

pub(super) fn dropdown_anchor_style(
    status: button::Status,
    opened: bool,
    focused: bool,
) -> button::Style {
    let background = match (opened, status) {
        (true, _) => Color {
            a: 0.6,
            ..tokyo_surface()
        },
        (false, button::Status::Hovered) => Color {
            a: 0.4,
            ..tokyo_surface()
        },
        (false, button::Status::Pressed) => Color {
            a: 0.6,
            ..tokyo_surface()
        },
        _ => Color::TRANSPARENT,
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: tokyo_text(),
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: if focused {
                tokyo_blue()
            } else {
                tokyo_border()
            },
        },
        ..button::Style::default()
    }
}

pub(super) fn dropdown_panel_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(tokyo_board())),
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: tokyo_border(),
        },
        ..container::Style::default()
    }
}

pub(super) fn dropdown_option_style(status: button::Status, selected: bool) -> button::Style {
    let background = match (selected, status) {
        (true, _) => tokyo_surface(),
        (false, button::Status::Hovered) => Color {
            a: 0.5,
            ..tokyo_surface()
        },
        (false, button::Status::Pressed) => tokyo_surface(),
        _ => Color::TRANSPARENT,
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: tokyo_text(),
        border: Border::default(),
        ..button::Style::default()
    }
}

pub(super) fn separator() -> Element<'static, Message> {
    container(Space::new())
        .height(Length::Fixed(1.0))
        .width(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(Background::Color(Color {
                a: 0.6,
                ..tokyo_border()
            })),
            ..container::Style::default()
        })
        .into()
}

pub(super) fn group_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        border: Border {
            radius: 4.0.into(),
            width: 1.0,
            color: tokyo_border(),
        },
        ..container::Style::default()
    }
}

pub(super) fn radio_style(focused: bool) -> impl Fn(&iced::Theme, radio::Status) -> radio::Style {
    move |_theme, status| {
        let selected = match status {
            radio::Status::Active { is_selected } | radio::Status::Hovered { is_selected } => {
                is_selected
            }
        };
        radio::Style {
            background: Background::Color(Color::TRANSPARENT),
            dot_color: if selected {
                tokyo_blue()
            } else {
                Color::TRANSPARENT
            },
            border_width: if focused { 2.0 } else { 1.0 },
            border_color: if focused {
                tokyo_blue()
            } else {
                tokyo_border()
            },
            text_color: Some(tokyo_text()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_radio_keeps_blue_dot_without_focus_border() {
        let style = radio_style(false)(
            &iced::Theme::Dark,
            radio::Status::Active { is_selected: true },
        );
        assert_eq!(style.dot_color, tokyo_blue());
        assert_eq!(style.border_color, tokyo_border());
    }
}
