//! Container styles: backgrounds, borders, and a couple of shared
//! primitive helpers (`surface_style`, `solid_style`) that other modules
//! reuse to assemble their own variants.

use iced::widget::container;
use iced::{Background, Border, Color, Theme};

use super::super::theme::{
    TOKYO_BLUE, TOKYO_BOARD, TOKYO_BORDER, TOKYO_RED, TOKYO_SELECTION_BLUE, TOKYO_TEXT,
    TOKYO_YELLOW,
};

pub(crate) fn app_style(_theme: &Theme) -> container::Style {
    surface_style(Some(TOKYO_BOARD), 0.0, 0.0, Color::TRANSPARENT)
}

pub(crate) fn menu_bar_style(_theme: &Theme) -> container::Style {
    surface_style(Some(TOKYO_BOARD), 0.0, 0.0, Color::TRANSPARENT)
}

pub(crate) fn board_style(_theme: &Theme) -> container::Style {
    surface_style(Some(TOKYO_BOARD), 8.0, 1.0, TOKYO_BORDER)
}

/// Outline-only variant of `board_style` for the left schematic panel:
/// the schematic already provides its own outline language, so an extra
/// rounded border around it reads as redundant.
pub(crate) fn schematic_board_style(_theme: &Theme) -> container::Style {
    surface_style(Some(TOKYO_BOARD), 0.0, 0.0, Color::TRANSPARENT)
}

/// Divider slot under the menu bar — paints in the plate colour so it
/// disappears visually while keeping the 1-px layout slot dropdowns
/// position against.
pub(crate) fn menu_bar_divider_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(TOKYO_BOARD)),
        border: Border {
            radius: 0.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        ..container::Style::default()
    }
}

pub(crate) fn panel_style(theme: &Theme) -> container::Style {
    board_style(theme)
}

/// Shared surface for hover tooltips. Matches `status_tooltip_style`
/// so every tooltip in the app uses the same darker plate fill.
pub(crate) fn inset_style(_theme: &Theme) -> container::Style {
    surface_style(Some(TOKYO_BOARD), 6.0, 1.0, TOKYO_BORDER)
}

pub(crate) fn status_tooltip_style(_theme: &Theme) -> container::Style {
    surface_style(Some(TOKYO_BOARD), 6.0, 1.0, TOKYO_BORDER)
}

/// `error_inset_style` reuses `TOKYO_BOARD` so the notice reads as
/// "another bubble on the same plate" instead of a foreign light box;
/// the red 1.5 px border alone carries the "this is an error" signal.
pub(crate) fn error_inset_style(_theme: &Theme) -> container::Style {
    surface_style(Some(TOKYO_BOARD), 8.0, 1.5, TOKYO_RED)
}

/// `error_inset_style` for the passive "info" notice shown when a
/// legacy-format `.580` is opened. Same plate-on-plate chrome but
/// `TOKYO_YELLOW` border instead of `TOKYO_RED` — "heads up, not an
/// error".
pub(crate) fn info_inset_style(_theme: &Theme) -> container::Style {
    surface_style(Some(TOKYO_BOARD), 8.0, 1.5, TOKYO_YELLOW)
}

pub(crate) fn schematic_block_style(_theme: &Theme) -> container::Style {
    surface_style(None, 6.0, 1.0, TOKYO_BORDER)
}

/// Header strip of the multiplexer panel. Transparent at rest, with
/// a full-rectangle border so the upper edge stays visible against
/// the plate (the outer `mux_panel_style` perimeter "disappears"
/// against `TOKYO_BOARD` there). Top corners share the 6 px radius;
/// bottom corners stay square so the strip butts into the next
/// section caption.
pub(crate) fn mux_header_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: None,
        text_color: Some(TOKYO_TEXT),
        border: Border {
            radius: iced::border::Radius {
                top_left: 6.0,
                top_right: 6.0,
                bottom_right: 0.0,
                bottom_left: 0.0,
            },
            width: 1.0,
            color: TOKYO_BORDER,
        },
        ..container::Style::default()
    }
}

pub(crate) fn mux_panel_style(_theme: &Theme) -> container::Style {
    surface_style(None, 6.0, 1.0, TOKYO_BORDER)
}

/// Outline-only chrome for individual chips inside the multiplexer
/// (W/Z scratch pair, SP/PC inline readouts) — matches `mux_panel_style`
/// at a smaller scale.
pub(crate) fn mux_chip_style(_theme: &Theme) -> container::Style {
    surface_style(None, 6.0, 1.0, TOKYO_BORDER)
}

pub(crate) fn legend_label_style(_theme: &Theme) -> container::Style {
    surface_style(Some(TOKYO_BOARD), 0.0, 0.0, Color::TRANSPARENT)
}

pub(crate) fn transparent_style(_theme: &Theme) -> container::Style {
    surface_style(None, 0.0, 0.0, Color::TRANSPARENT)
}

/// Lights up the spinner shell border in blue while the embedded text
/// input owns keyboard focus. The shell stays neutral on hover so the
/// only feedback that promises "you can type here" is the focus ring,
/// matching the convention of native form fields.
pub(crate) fn input_shell_style(_theme: &Theme, focused: bool) -> container::Style {
    let border_color = if focused { TOKYO_BLUE } else { TOKYO_BORDER };

    surface_style(Some(TOKYO_BOARD), 6.0, 1.0, border_color)
}

/// Floating opcode picker. Matches the surrounding board panels so
/// it reads as part of the same surface, not a darker pop-up. All
/// four corners get the same radius — the picker floats over the
/// memory list, so a square top edge would look clipped.
pub(crate) fn opcode_dropdown_style(_theme: &Theme) -> container::Style {
    container::Style {
        text_color: Some(TOKYO_TEXT),
        background: Some(Background::Color(TOKYO_BOARD)),
        border: Border {
            radius: iced::border::Radius {
                top_left: 7.0,
                top_right: 7.0,
                bottom_right: 7.0,
                bottom_left: 7.0,
            },
            width: 1.0,
            color: TOKYO_BORDER,
        },
        ..container::Style::default()
    }
}

/// Halted rows override the regular selection styling with a red
/// fill (no extra border, to match the weight of the blue selection).
pub(crate) fn memory_row_container_style(selected: bool, halted: bool) -> container::Style {
    if halted {
        return container::Style {
            background: Some(Background::Color(Color::from_rgba8(0xF7, 0x76, 0x8E, 0.22))),
            text_color: Some(TOKYO_TEXT),
            border: Border {
                radius: 6.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            ..container::Style::default()
        };
    }

    let background = if selected {
        Some(Background::Color(TOKYO_SELECTION_BLUE))
    } else {
        None
    };

    container::Style {
        background,
        text_color: Some(TOKYO_TEXT),
        border: Border {
            // Round only the highlighted row so 1-px separators between
            // unhighlighted rows still meet edge-to-edge.
            radius: if selected { 6.0.into() } else { 0.0.into() },
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        ..container::Style::default()
    }
}

pub(crate) fn solid_style(color: Color, radius: f32) -> container::Style {
    container::Style {
        background: Some(Background::Color(color)),
        border: Border {
            radius: radius.into(),
            ..Border::default()
        },
        ..container::Style::default()
    }
}

pub(crate) fn surface_style(
    background: Option<Color>,
    radius: f32,
    border_width: f32,
    border_color: Color,
) -> container::Style {
    container::Style {
        text_color: Some(TOKYO_TEXT),
        background: background.map(Background::Color),
        border: Border {
            radius: radius.into(),
            width: border_width,
            color: border_color,
        },
        ..container::Style::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opcode_dropdown_has_rounded_top_corners() {
        let style = opcode_dropdown_style(&Theme::TokyoNight);

        assert!(style.border.radius.top_left > 0.0);
        assert!(style.border.radius.top_right > 0.0);
    }

    #[test]
    fn regular_tooltips_share_status_tooltip_surface() {
        let inset = inset_style(&Theme::TokyoNight);
        let status = status_tooltip_style(&Theme::TokyoNight);

        assert_eq!(inset.background, status.background);
    }

    #[test]
    fn input_shell_background_matches_app_plate() {
        let idle = input_shell_style(&Theme::TokyoNight, false);
        let focused = input_shell_style(&Theme::TokyoNight, true);

        assert_eq!(idle.background, Some(Background::Color(TOKYO_BOARD)));
        assert_eq!(focused.background, Some(Background::Color(TOKYO_BOARD)));
    }

    #[test]
    fn menu_bar_divider_paints_as_app_plate() {
        let divider = menu_bar_divider_style(&Theme::TokyoNight);

        assert_eq!(divider.background, Some(Background::Color(TOKYO_BOARD)));
    }
}
