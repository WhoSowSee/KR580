//! Button styles. Each helper picks a backing colour and border based on
//! the button's interaction status, plus optional accent / selected
//! flags for the more elaborate variants.

use iced::widget::button;
use iced::{Background, Border, Color};

use super::super::theme::{
    TOKYO_BG, TOKYO_BORDER, TOKYO_MUTED, TOKYO_RED, TOKYO_SELECTION_BLUE, TOKYO_SURFACE,
    TOKYO_SURFACE_2, TOKYO_SURFACE_3, TOKYO_TEXT,
};

/// Capsule-shaped button used by `functional_block` (Аккумулятор /
/// Буферный регистр 1 / Буферный регистр 2) on the schematic plate.
///
/// Resting state has no fill: the border is what makes every
/// schematic slot read as part of one outline-only family. Hover and
/// press temporarily raise the surface tone so the chip still
/// telegraphs interactivity.
///
/// The per-chip accent only paints the border under hover/press/
/// selected. Resting frames stay neutral so the row does not flare
/// with coloured outlines just because the cursor passes by.
pub(crate) fn schematic_block_button_style(
    status: button::Status,
    accent: Color,
    selected: bool,
) -> button::Style {
    let active = is_button_active(status);
    let background = if selected {
        Some(Color::from_rgba8(0xBB, 0x9A, 0xF7, 0.18))
    } else if active {
        Some(TOKYO_SURFACE_3)
    } else {
        None
    };
    let border_color = if active || selected {
        accent
    } else {
        TOKYO_BORDER
    };

    button::Style {
        background: background.map(Background::Color),
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
/// Reset…). Reuses the same chrome as the editor `↵` button: neutral
/// border at all times, with only the surface tone shifting on hover /
/// press. The colour-coded affordance comes from the SVG glyph itself
/// (each button has its own `accent` tint), so the border can stay calm
/// without losing the per-button identity. Keeps the row of action
/// chips visually coherent with the surrounding inputs instead of
/// flaring up a coloured frame whenever the cursor lands on a chip.
///
/// `Disabled` is its own visual branch: the surface stays at the
/// resting `TOKYO_BG` tone, but the border drops to the same low-alpha
/// tint the menu separator uses and the text colour fades to
/// `TOKYO_MUTED`. The glyph itself is greyed out by
/// `icon_action_button_glyph_color` (called from the chip widget when
/// the caller passed `None` for `message`); the border + text fade
/// here is what tells the *frame* "this chip is locked", so the user
/// reads disabled-ness from the chrome even before parsing the muted
/// glyph.
pub(crate) fn action_button_style(status: button::Status) -> button::Style {
    let disabled = matches!(status, button::Status::Disabled);
    let background = match status {
        button::Status::Pressed => TOKYO_SURFACE_2,
        button::Status::Hovered => TOKYO_SURFACE,
        _ => TOKYO_BG,
    };
    let border_color = if disabled {
        Color {
            a: 0.35,
            ..TOKYO_BORDER
        }
    } else {
        TOKYO_BORDER
    };
    let text_color = if disabled { TOKYO_MUTED } else { TOKYO_TEXT };

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

/// Disabled variant of `menu_button_style`. Same chrome (transparent
/// background, 6 px radius, no border) but the row never lights up on
/// hover/press because we never publish an `on_press` for it — the
/// button stays in `Status::Disabled` for the whole render pass and
/// the surface stays flat. Text colour is `TOKYO_MUTED` so the row
/// reads as «недоступно сейчас» the same way the disabled action-strip
/// chips do (see `action_chip_style`'s muted-tint branch). Used by
/// «Сбросить флаг HLT» when the halt flip-flop is already off — the
/// row stays in the menu so the user can still discover it and read
/// the shortcut, but the visual weight tells them clicking changes
/// nothing.
pub(crate) fn menu_button_disabled_style(_status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: TOKYO_MUTED,
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

pub(crate) fn schematic_select_button_style(
    status: button::Status,
    selected: bool,
) -> button::Style {
    let background = if selected {
        Some(TOKYO_SELECTION_BLUE)
    } else {
        match status {
            button::Status::Hovered => Some(TOKYO_SURFACE),
            button::Status::Pressed => Some(TOKYO_SURFACE_2),
            _ => None,
        }
    };
    let (border_width, border_color) = if selected || is_button_active(status) {
        (0.0, Color::TRANSPARENT)
    } else {
        (1.0, TOKYO_BORDER)
    };

    button::Style {
        background: background.map(Background::Color),
        text_color: TOKYO_TEXT,
        border: Border {
            radius: 4.0.into(),
            width: border_width,
            color: border_color,
        },
        ..button::Style::default()
    }
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

/// Style for the minimise / maximise caption buttons in the custom
/// title bar. Transparent at rest so the bar reads as a single
/// contiguous surface; a faint surface tint lights up on hover so the
/// caption zone still telegraphs interactivity. Mirrors the native
/// caption convention: no border, square corners would conflict with
/// the rest of the chrome so we keep the 4 px radius the menu uses.
pub(crate) fn caption_button_style(status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => TOKYO_SURFACE_2,
        button::Status::Pressed => TOKYO_SURFACE_3,
        _ => Color::TRANSPARENT,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: TOKYO_TEXT,
        border: Border {
            radius: 4.0.into(),
            ..Border::default()
        },
        ..button::Style::default()
    }
}

/// Style for the close caption button. Same flat-by-default chrome as
/// `caption_button_style`, except the hover/press surface flares red
/// so the destructive action lands with the same affordance as the
/// native window manager's close glyph. We do not recolour the SVG
/// stroke itself — the red surface already reads "warning", and the
/// glyph stays legible against it.
pub(crate) fn close_caption_button_style(status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => TOKYO_RED,
        button::Status::Pressed => Color {
            a: 0.85,
            ..TOKYO_RED
        },
        _ => Color::TRANSPARENT,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: TOKYO_TEXT,
        border: Border {
            radius: 4.0.into(),
            ..Border::default()
        },
        ..button::Style::default()
    }
}
