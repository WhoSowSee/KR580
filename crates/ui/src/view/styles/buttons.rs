//! Button styles. Each helper picks a backing colour and border based on
//! the button's interaction status, plus optional accent / selected
//! flags for the more elaborate variants.

use iced::widget::button;
use iced::{Background, Border, Color};

use super::super::theme::{
    TOKYO_BG, TOKYO_BORDER, TOKYO_GREEN, TOKYO_SURFACE, TOKYO_SURFACE_2, TOKYO_SURFACE_3,
    TOKYO_TEXT,
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

/// Style for the action buttons in the "Управление" panel (Run, Step,
/// Reset…). Reuses the same surface palette as the editor `↵` button so
/// the row of action chips reads as part of the surrounding panels, and
/// uses `accent` only for the border affordance on hover/press. The
/// neutral idle border keeps the panel calm; the colour shows up only
/// when the user is about to commit, mirroring the existing register /
/// memory editor convention.
pub(crate) fn action_button_style(status: button::Status, accent: Color) -> button::Style {
    let active = is_button_active(status);
    let background = match status {
        button::Status::Pressed => TOKYO_SURFACE_2,
        button::Status::Hovered => TOKYO_SURFACE,
        _ => TOKYO_BG,
    };
    let border_color = if active { accent } else { TOKYO_BORDER };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: TOKYO_TEXT,
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: border_color,
        },
        ..button::Style::default()
    }
}

/// Style for the `↵` apply buttons next to each editor field. Visually
/// matches the surrounding text inputs: same background colour, same
/// border radius, neutral border that does not light up on hover. The
/// only feedback is a slightly lighter surface tint when the cursor
/// hovers, with a touch more contrast on press.
pub(crate) fn enter_button_style(status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Pressed => TOKYO_SURFACE_2,
        button::Status::Hovered => TOKYO_SURFACE,
        _ => TOKYO_BG,
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
    // Render the spinner arrows as inline glyphs: transparent background,
    // no border. A subtle surface tint on hover/press is enough to signal
    // interactivity without making them look like detached chips that sit
    // on top of the input.
    let background = if is_button_active(status) {
        Color::from_rgba8(0x36, 0x3B, 0x59, 0.45)
    } else {
        Color::TRANSPARENT
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: TOKYO_TEXT,
        border: Border {
            radius: 3.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
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
