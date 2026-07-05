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

use super::super::theme::{tokyo_green, tokyo_muted, tokyo_text, tokyo_text_selection};

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
        icon: tokyo_muted(),
        placeholder: tokyo_muted(),
        value: tokyo_text(),
        selection: tokyo_text_selection(),
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
        icon: tokyo_muted(),
        placeholder: tokyo_muted(),
        value: tokyo_muted(),
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
        icon: tokyo_muted(),
        placeholder: tokyo_muted(),
        value: tokyo_green(),
        selection: tokyo_text_selection(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn borderless_inputs_use_readable_gray_selection() {
        let style = input_borderless_style(
            &Theme::TokyoNight,
            text_input::Status::Focused { is_hovered: false },
        );

        assert_eq!(style.selection, tokyo_text_selection());
        assert!(style.selection.a < 0.35);
        assert!(style.selection.r > 0.6);
        assert!(style.selection.g > 0.6);
        assert!(style.selection.b > 0.6);
    }

    #[test]
    fn inline_value_inputs_share_readable_selection() {
        let style = inline_value_input_style(
            &Theme::TokyoNight,
            text_input::Status::Focused { is_hovered: false },
        );

        assert_eq!(style.selection, tokyo_text_selection());
    }
}
