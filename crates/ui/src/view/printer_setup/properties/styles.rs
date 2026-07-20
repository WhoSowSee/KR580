use iced::widget::{button, container, text_input};
use iced::{Background, Border, Color, Shadow};

use super::super::super::theme::{tokyo_blue, tokyo_border, tokyo_muted, tokyo_text};

pub(super) fn tab_style(
    active: bool,
    focused: bool,
) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let text_color = if active {
            tokyo_blue()
        } else if status == button::Status::Hovered {
            tokyo_text()
        } else {
            tokyo_muted()
        };
        button::Style {
            background: None,
            text_color,
            border: Border {
                radius: 4.0.into(),
                width: if focused { 1.0 } else { 0.0 },
                color: tokyo_blue(),
            },
            shadow: Shadow::default(),
            snap: true,
        }
    }
}

pub(super) fn input_style(keyboard_focused: bool) -> text_input::Style {
    text_input::Style {
        background: Background::Color(Color::TRANSPARENT),
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: if keyboard_focused {
                tokyo_blue()
            } else {
                tokyo_border()
            },
        },
        icon: tokyo_muted(),
        placeholder: tokyo_muted(),
        value: tokyo_text(),
        selection: Color {
            a: 0.45,
            ..tokyo_blue()
        },
    }
}

pub(super) fn paper_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::WHITE)),
        border: Border {
            radius: 2.0.into(),
            width: 1.0,
            color: tokyo_border(),
        },
        ..container::Style::default()
    }
}

pub(super) fn active_tab_line(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(tokyo_blue())),
        ..container::Style::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focused_input_border_depends_on_keyboard_focus_visibility() {
        let mouse_style = input_style(false);
        assert_eq!(mouse_style.border.color, tokyo_border());

        let keyboard_style = input_style(true);
        assert_eq!(keyboard_style.border.color, tokyo_blue());
    }
}
