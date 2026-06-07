//! Text input styles. Two flavours: a borderless one used inside the
//! spinner / value shells (the visible focus ring is rendered by
//! `input_shell_style` on the surrounding container), and a transparent
//! one used by the inline memory cell editor.
//!
//! The previous fully-bordered `input_style` was retired together with
//! the focus-tracking refactor in `runtime/`: relying on iced's internal
//! `text_input::Status::Focused` to draw the blue ring made it possible
//! for two sibling inputs to *both* be drawn as focused (iced captures
//! pointer presses inside `text_input::update` and never propagates a
//! "blur" to other inputs in the same tree). Driving the ring from the
//! shell lets us decouple the visual indicator from iced's internal
//! per-widget state and source it from a single value
//! (`DesktopApp::focused_input`) that `MousePressed` ->
//! `reconcile_focus_at` keeps in sync with whatever iced reports as
//! actually focused.

use iced::widget::text_input;
use iced::{Background, Border, Color, Theme};

use super::super::theme::{TOKYO_GREEN, TOKYO_MAGENTA, TOKYO_MUTED, TOKYO_TEXT};

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

pub(crate) fn disabled_input_borderless_style(
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
        value: TOKYO_MUTED,
        selection: Color::TRANSPARENT,
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
