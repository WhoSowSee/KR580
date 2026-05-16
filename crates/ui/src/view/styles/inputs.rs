//! Text input styles. Three flavours: a fully-bordered input, a
//! borderless one used inside the spinner shells, and a transparent one
//! used by the inline memory cell editor.

use iced::widget::text_input;
use iced::{Background, Border, Color, Theme};

use super::super::theme::{
    TOKYO_BG, TOKYO_BLUE, TOKYO_BORDER, TOKYO_CYAN, TOKYO_GREEN, TOKYO_MAGENTA, TOKYO_MUTED,
    TOKYO_TEXT,
};

pub(crate) fn input_style(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let border_color = match status {
        text_input::Status::Focused { .. } => TOKYO_BLUE,
        text_input::Status::Hovered => TOKYO_CYAN,
        text_input::Status::Active | text_input::Status::Disabled => TOKYO_BORDER,
    };

    text_input::Style {
        background: Background::Color(TOKYO_BG),
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: border_color,
        },
        icon: TOKYO_MUTED,
        placeholder: TOKYO_MUTED,
        value: TOKYO_TEXT,
        selection: TOKYO_MAGENTA,
    }
}

pub(crate) fn input_borderless_style(
    _theme: &Theme,
    _status: text_input::Status,
) -> text_input::Style {
    text_input::Style {
        background: Background::Color(Color::TRANSPARENT),
        border: Border {
            radius: 0.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        icon: TOKYO_MUTED,
        placeholder: TOKYO_MUTED,
        value: TOKYO_TEXT,
        selection: TOKYO_MAGENTA,
    }
}

pub(crate) fn inline_value_input_style(
    _theme: &Theme,
    _status: text_input::Status,
) -> text_input::Style {
    text_input::Style {
        background: Background::Color(Color::TRANSPARENT),
        border: Border {
            radius: 0.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        icon: TOKYO_MUTED,
        placeholder: TOKYO_MUTED,
        value: TOKYO_GREEN,
        selection: TOKYO_MAGENTA,
    }
}
