use iced::widget::button;
use iced::{Background, Border, Color, Padding, Theme};

pub(super) use crate::view::styles::panel_style as dialog_style;
use crate::view::theme::{
    TOKYO_BLUE, TOKYO_BOARD, TOKYO_BORDER, TOKYO_SURFACE, TOKYO_SURFACE_2, TOKYO_TEXT,
};

pub(super) const HEX_GROUP: usize = 16;
pub(super) const ICON_BUTTON_SIZE: f32 = 32.0;
pub(super) const ICON_GLYPH_SIZE: f32 = 18.0;
pub(super) const MODAL_MARGIN: f32 = 60.0;

pub(super) fn backdrop_style(_theme: &Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(Background::Color(Color {
            r: 0.07,
            g: 0.07,
            b: 0.13,
            a: 0.85,
        })),
        border: Border {
            radius: 0.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        ..iced::widget::container::Style::default()
    }
}

pub(super) fn popup_backdrop_style(_theme: &Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(Background::Color(Color {
            r: 0.05,
            g: 0.05,
            b: 0.10,
            a: 0.55,
        })),
        ..iced::widget::container::Style::default()
    }
}

pub(super) fn framebuffer_style(_theme: &Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(Background::Color(TOKYO_BOARD)),
        border: Border {
            radius: 4.0.into(),
            width: 1.0,
            color: TOKYO_BORDER,
        },
        ..iced::widget::container::Style::default()
    }
}

pub(super) fn framebuffer_padding(empty: bool) -> Padding {
    Padding {
        top: if empty { 28.0 } else { 12.0 },
        right: 12.0,
        bottom: 12.0,
        left: 12.0,
    }
}

pub(super) fn icon_button_style(status: button::Status, active: bool) -> button::Style {
    let background = if active {
        TOKYO_SURFACE_2
    } else {
        match status {
            button::Status::Pressed => TOKYO_SURFACE_2,
            button::Status::Hovered => TOKYO_SURFACE,
            _ => TOKYO_BOARD,
        }
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: TOKYO_TEXT,
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: if active { TOKYO_BLUE } else { TOKYO_BORDER },
        },
        ..button::Style::default()
    }
}
