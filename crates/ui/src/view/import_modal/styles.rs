use iced::widget::{button, container};
use iced::{Background, Border, Color};

use super::super::styles::input_shell_style;
use super::super::theme::{
    TOKYO_BOARD, TOKYO_BORDER, TOKYO_RED, TOKYO_SURFACE, TOKYO_SURFACE_2, TOKYO_TEXT,
};

pub(super) fn modal_backdrop_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color {
            r: 0.07,
            g: 0.07,
            b: 0.13,
            a: 0.70,
        })),
        border: Border::default(),
        ..container::Style::default()
    }
}

pub(super) fn modal_dialog_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        text_color: Some(TOKYO_TEXT),
        background: Some(Background::Color(TOKYO_BOARD)),
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: TOKYO_BORDER,
        },
        ..container::Style::default()
    }
}

pub(super) fn group_panel_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        text_color: Some(TOKYO_TEXT),
        background: Some(Background::Color(TOKYO_BOARD)),
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: TOKYO_BORDER,
        },
        ..container::Style::default()
    }
}

pub(super) fn group_label_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        text_color: Some(TOKYO_TEXT),
        background: Some(Background::Color(TOKYO_BOARD)),
        border: Border::default(),
        ..container::Style::default()
    }
}

pub(super) fn field_button_style(status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => Color {
            a: 0.45,
            ..TOKYO_SURFACE
        },
        button::Status::Pressed => TOKYO_SURFACE_2,
        _ => Color::TRANSPARENT,
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: TOKYO_TEXT,
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: TOKYO_BORDER,
        },
        ..button::Style::default()
    }
}

pub(super) fn icon_button_style(status: button::Status) -> button::Style {
    field_button_style(status)
}

pub(super) fn footer_button_style(status: button::Status) -> button::Style {
    field_button_style(status)
}

pub(super) fn dropdown_panel_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        text_color: Some(TOKYO_TEXT),
        background: Some(Background::Color(TOKYO_BOARD)),
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: TOKYO_BORDER,
        },
        ..container::Style::default()
    }
}

pub(super) fn dropdown_option_style(status: button::Status, highlighted: bool) -> button::Style {
    let background = match (highlighted, status) {
        (true, _) => TOKYO_SURFACE,
        (false, button::Status::Hovered) => Color {
            a: 0.45,
            ..TOKYO_SURFACE
        },
        (false, button::Status::Pressed) => TOKYO_SURFACE_2,
        _ => Color::TRANSPARENT,
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: TOKYO_TEXT,
        border: Border {
            radius: 4.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        ..button::Style::default()
    }
}

pub(super) fn badge_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color {
            a: 0.35,
            ..TOKYO_SURFACE
        })),
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: TOKYO_BORDER,
        },
        ..container::Style::default()
    }
}

pub(super) fn error_shell_style(theme: &iced::Theme) -> container::Style {
    let mut style = input_shell_style(theme, false);
    style.border.color = TOKYO_RED;
    style
}
