//! Button styles. Each helper picks a backing colour and border based on
//! the button's interaction status, plus optional accent / selected
//! flags for the more elaborate variants.

use iced::widget::button;
use iced::{Background, Border, Color};

use crate::app::TopMenuIndicator;

use super::super::theme::{
    tokyo_blue, tokyo_board, tokyo_border, tokyo_muted, tokyo_red, tokyo_surface, tokyo_surface_2,
    tokyo_surface_3, tokyo_surface_3_tint, tokyo_text,
};

fn is_button_active(status: button::Status) -> bool {
    matches!(status, button::Status::Hovered | button::Status::Pressed)
}

/// Action-panel chrome (Run, Step, Reset…). Neutral border at rest;
/// only the surface tone shifts on hover/press. Per-button identity
/// comes from the SVG glyph's accent tint.
///
/// `Disabled` is a separate visual branch: surface stays at the
/// resting `tokyo_board()` tone but the border drops to a low-alpha
/// tint and text fades to `tokyo_muted()`. The glyph itself is greyed
/// out separately by the chip widget when `message` is `None` –
/// border + text fade here is what tells the *frame* that the chip
/// is locked.
pub(crate) fn action_button_style(status: button::Status) -> button::Style {
    let disabled = matches!(status, button::Status::Disabled);
    let background = match status {
        button::Status::Pressed => tokyo_surface_2(),
        button::Status::Hovered => tokyo_surface(),
        _ => tokyo_board(),
    };
    let border_color = if disabled {
        Color {
            a: 0.35,
            ..tokyo_border()
        }
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

/// Style for the `↵` apply buttons next to each editor field. Visually
/// matches the surrounding text inputs: same background colour, same
/// border radius, neutral border that does not light up on hover. The
/// only feedback is a slightly lighter surface tint when the cursor
/// hovers, with a touch more contrast on press.
pub(crate) fn enter_button_style(status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Pressed => tokyo_surface_2(),
        button::Status::Hovered => tokyo_surface(),
        _ => tokyo_board(),
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: tokyo_text(),
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: tokyo_border(),
        },
        ..button::Style::default()
    }
}

pub(crate) fn modal_field_button_style(status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => Color {
            a: 0.45,
            ..tokyo_surface()
        },
        button::Status::Pressed => tokyo_surface_2(),
        _ => Color::TRANSPARENT,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: tokyo_text(),
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: tokyo_border(),
        },
        ..button::Style::default()
    }
}

pub(crate) fn modal_tab_button_style(status: button::Status, active: bool) -> button::Style {
    let background = match (active, status) {
        (true, _) => tokyo_surface(),
        (false, button::Status::Hovered) => Color {
            a: 0.45,
            ..tokyo_surface()
        },
        (false, button::Status::Pressed) => tokyo_surface_2(),
        _ => Color::TRANSPARENT,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: tokyo_text(),
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: tokyo_border(),
        },
        ..button::Style::default()
    }
}

pub(crate) fn modal_dropdown_option_style(
    status: button::Status,
    highlighted: bool,
) -> button::Style {
    let background = match (highlighted, status) {
        (true, _) => tokyo_surface(),
        (false, button::Status::Hovered) => Color {
            a: 0.45,
            ..tokyo_surface()
        },
        (false, button::Status::Pressed) => tokyo_surface_2(),
        _ => Color::TRANSPARENT,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: tokyo_text(),
        border: Border {
            radius: 4.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        ..button::Style::default()
    }
}

pub(crate) fn menu_button_style(
    status: button::Status,
    indicator: TopMenuIndicator,
) -> button::Style {
    let background = match indicator {
        TopMenuIndicator::ArrowFill => tokyo_surface_2(),
        TopMenuIndicator::TabRing => Color::TRANSPARENT,
        TopMenuIndicator::Hidden if is_button_active(status) => tokyo_surface_2(),
        TopMenuIndicator::Hidden => Color::TRANSPARENT,
    };
    let keyboard_focused = indicator == TopMenuIndicator::TabRing;

    button::Style {
        background: Some(Background::Color(background)),
        text_color: tokyo_text(),
        border: Border {
            radius: 6.0.into(),
            width: if keyboard_focused { 1.0 } else { 0.0 },
            color: if keyboard_focused {
                tokyo_blue()
            } else {
                Color::TRANSPARENT
            },
        },
        ..button::Style::default()
    }
}

/// Disabled variant of `menu_button_style`: same chrome but the row
/// never lights up on hover/press because we never publish an
/// `on_press` for it. Text colour is `tokyo_muted()` so the row reads
/// as "currently unavailable" while staying discoverable in the menu
/// (used by the "clear HLT flag" entry when the flip-flop is off).
pub(crate) fn menu_button_disabled_style(_status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: tokyo_muted(),
        border: Border {
            radius: 6.0.into(),
            ..Border::default()
        },
        ..button::Style::default()
    }
}

pub(crate) fn step_button_style(status: button::Status) -> button::Style {
    // Inline glyphs: transparent at rest, faint surface tint on
    // hover/press – no border so they don't read as detached chips.
    let background = if is_button_active(status) {
        tokyo_surface_3_tint()
    } else {
        Color::TRANSPARENT
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: tokyo_text(),
        border: Border {
            radius: 3.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        ..button::Style::default()
    }
}

pub(crate) fn opcode_option_style(status: button::Status, highlighted: bool) -> button::Style {
    let background = if highlighted || is_button_active(status) {
        tokyo_surface_3()
    } else {
        Color::TRANSPARENT
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: tokyo_text(),
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
        button::Status::Hovered => tokyo_surface_2(),
        button::Status::Pressed => tokyo_surface_3(),
        _ => Color::TRANSPARENT,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: tokyo_text(),
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
/// stroke itself – the red surface already reads "warning", and the
/// glyph stays legible against it.
pub(crate) fn close_caption_button_style(status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => tokyo_red(),
        button::Status::Pressed => Color {
            a: 0.85,
            ..tokyo_red()
        },
        _ => Color::TRANSPARENT,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: tokyo_text(),
        border: Border {
            radius: 4.0.into(),
            ..Border::default()
        },
        ..button::Style::default()
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::theme::{tokyo_blue, tokyo_board};
    use super::*;

    #[test]
    fn action_button_resting_background_matches_app_plate() {
        let active = action_button_style(button::Status::Active);
        let disabled = action_button_style(button::Status::Disabled);

        assert_eq!(active.background, Some(Background::Color(tokyo_board())));
        assert_eq!(disabled.background, Some(Background::Color(tokyo_board())));
    }

    #[test]
    fn enter_button_resting_background_matches_app_plate() {
        let style = enter_button_style(button::Status::Active);

        assert_eq!(style.background, Some(Background::Color(tokyo_board())));
    }

    #[test]
    fn keyboard_focused_menu_item_uses_blue_border_without_fill() {
        let style = menu_button_style(button::Status::Active, TopMenuIndicator::TabRing);

        assert_eq!(
            style.background,
            Some(Background::Color(Color::TRANSPARENT))
        );
        assert_eq!(style.border.width, 1.0);
        assert_eq!(style.border.color, tokyo_blue());
    }

    #[test]
    fn arrow_focused_menu_item_uses_hover_fill_without_border() {
        let style = menu_button_style(button::Status::Active, TopMenuIndicator::ArrowFill);

        assert_eq!(style.background, Some(Background::Color(tokyo_surface_2())));
        assert_eq!(style.border.width, 0.0);
        assert_eq!(style.border.color, Color::TRANSPARENT);
    }
}
