use iced::widget::button;
use iced::{Background, Border, Color, Padding, Theme};

pub(super) use crate::view::styles::panel_style as dialog_style;
use crate::view::theme::{
    tokyo_blue, tokyo_board, tokyo_border, tokyo_modal_backdrop, tokyo_selection_blue,
    tokyo_surface, tokyo_surface_2, tokyo_text,
};

pub(super) const HEX_GROUP: usize = 16;
pub(super) const ICON_BUTTON_SIZE: f32 = 32.0;
pub(super) const ICON_GLYPH_SIZE: f32 = 18.0;
pub(super) const MODAL_MARGIN: f32 = 60.0;

pub(super) fn backdrop_style(_theme: &Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(Background::Color(Color {
            a: 0.85,
            ..tokyo_modal_backdrop()
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
            a: 0.55,
            ..tokyo_modal_backdrop()
        })),
        ..iced::widget::container::Style::default()
    }
}

pub(super) fn framebuffer_style(_theme: &Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(Background::Color(tokyo_board())),
        border: Border {
            radius: 4.0.into(),
            width: 1.0,
            color: tokyo_border(),
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
        tokyo_selection_blue()
    } else {
        match status {
            button::Status::Pressed => tokyo_surface_2(),
            button::Status::Hovered => tokyo_surface(),
            _ => tokyo_board(),
        }
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: tokyo_text(),
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: if active { tokyo_blue() } else { tokyo_border() },
        },
        ..button::Style::default()
    }
}
