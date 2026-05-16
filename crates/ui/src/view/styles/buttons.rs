//! Button styles. Each helper picks a backing colour and border based on
//! the button's interaction status, plus optional accent / selected
//! flags for the more elaborate variants.

use iced::widget::button;
use iced::{Background, Border, Color};

use super::super::theme::{
    TOKYO_BORDER, TOKYO_GREEN, TOKYO_SURFACE, TOKYO_SURFACE_2, TOKYO_SURFACE_3, TOKYO_TEXT,
};

pub(crate) fn capsule_button_style(
    status: button::Status,
    accent: Color,
    selected: bool,
) -> button::Style {
    let active = is_button_active(status);
    let background = if selected {
        Color::from_rgba8(0xBB, 0x9A, 0xF7, 0.28)
    } else if active {
        TOKYO_SURFACE_3
    } else {
        TOKYO_SURFACE
    };
    let border_color = if active || selected {
        accent
    } else {
        TOKYO_BORDER
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: TOKYO_TEXT,
        border: Border {
            radius: 6.0.into(),
            width: if selected { 1.5 } else { 1.0 },
            color: border_color,
        },
        ..button::Style::default()
    }
}

pub(crate) fn is_button_active(status: button::Status) -> bool {
    matches!(status, button::Status::Hovered | button::Status::Pressed)
}

pub(crate) fn flat_button_style(text_color: Color) -> button::Style {
    button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color,
        border: Border::default(),
        ..button::Style::default()
    }
}

pub(crate) fn menu_button_style(status: button::Status) -> button::Style {
    let background = if is_button_active(status) {
        TOKYO_SURFACE_2
    } else {
        Color::TRANSPARENT
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: TOKYO_TEXT,
        border: Border {
            radius: 6.0.into(),
            ..Border::default()
        },
        ..button::Style::default()
    }
}

pub(crate) fn step_button_style(status: button::Status) -> button::Style {
    let background = if is_button_active(status) {
        TOKYO_SURFACE_3
    } else {
        TOKYO_SURFACE
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: TOKYO_TEXT,
        border: Border {
            radius: 3.0.into(),
            width: 1.0,
            color: TOKYO_BORDER,
        },
        ..button::Style::default()
    }
}

pub(crate) fn mux_button_style(
    status: button::Status,
    accent: Color,
    selected: bool,
) -> button::Style {
    let active = is_button_active(status);
    let background = if selected {
        Color::from_rgba8(0xBB, 0x9A, 0xF7, 0.45)
    } else if active {
        TOKYO_SURFACE_3
    } else {
        TOKYO_SURFACE
    };
    let border_color = if selected || active {
        accent
    } else {
        TOKYO_BORDER
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: TOKYO_TEXT,
        border: Border {
            radius: 4.0.into(),
            width: 1.0,
            color: border_color,
        },
        ..button::Style::default()
    }
}

pub(crate) fn cell_button_style(_status: button::Status) -> button::Style {
    flat_button_style(TOKYO_TEXT)
}

pub(crate) fn value_button_style(_status: button::Status) -> button::Style {
    flat_button_style(TOKYO_GREEN)
}

pub(crate) fn opcode_option_style(status: button::Status) -> button::Style {
    let background = if is_button_active(status) {
        TOKYO_SURFACE_3
    } else {
        Color::TRANSPARENT
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: TOKYO_TEXT,
        border: Border::default(),
        ..button::Style::default()
    }
}
