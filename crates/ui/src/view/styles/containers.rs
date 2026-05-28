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

/// Variant of `board_style` for the left "schematic" panel that drops
/// the bubble chrome: no border, no rounded corners. The schematic
/// already provides its own outline language (mux frame, ALU frame,
/// schematic block readouts), so wrapping it in another framed surface
/// read as redundant.
pub(crate) fn schematic_board_style(_theme: &Theme) -> container::Style {
    surface_style(Some(TOKYO_BOARD), 0.0, 0.0, Color::TRANSPARENT)
}

/// Divider slot under the menu bar. It keeps the existing 1-px layout
/// rhythm for dropdown positioning, but paints the same colour as the
/// app plate so the visual separator disappears.
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

/// Variant of `inset_style` for the floating error notice. The user
/// flagged the previous `TOKYO_SURFACE` fill as too light against the
/// surrounding chrome — a notice that is visually *louder* than the
/// rest of the app made the rest read as suppressed even when the
/// overlay was passive. `TOKYO_BOARD` matches the app plate (the
/// background every other panel sits on), so the notice now reads as
/// "another bubble on the same plate" rather than a foreign light
/// box. Border stays `TOKYO_RED` at 1.5 px so the framing alone
/// carries the "this is an error" signal.
pub(crate) fn error_inset_style(_theme: &Theme) -> container::Style {
    surface_style(Some(TOKYO_BOARD), 8.0, 1.5, TOKYO_RED)
}

/// Variant of `error_inset_style` for the passive "info" notice that
/// appears when the user opens a legacy-format `.580` file. Same
/// plate-on-plate chrome as the error notice — `TOKYO_BOARD` fill,
/// 8 px radius, 1.5 px border — but the frame is `TOKYO_YELLOW`
/// instead of `TOKYO_RED`. Yellow signals "heads up, not an error"
/// in the same visual idiom: the user does not need to act, just
/// notice that the snapshot came in via the legacy decoder so a
/// subsequent save round-trips through the right serializer.
pub(crate) fn info_inset_style(_theme: &Theme) -> container::Style {
    surface_style(Some(TOKYO_BOARD), 8.0, 1.5, TOKYO_YELLOW)
}

pub(crate) fn schematic_block_style(_theme: &Theme) -> container::Style {
    surface_style(None, 6.0, 1.0, TOKYO_BORDER)
}

pub(crate) fn mux_header_style(_theme: &Theme) -> container::Style {
    // Header strip of the multiplexer panel. It stays transparent in
    // the resting state so the whole left schematic uses one
    // outline-only chrome language.
    //
    // The full-rectangle border (rather than just a bottom edge)
    // is deliberate: the panel's outer `mux_panel_style` already
    // draws the perimeter, but at the top it lands on the
    // schematic plate which uses the `TOKYO_BOARD` tone, so
    // the user reported the upper edge "disappearing" while the
    // sides and bottom still read fine. Painting the header strip
    // with its own hairline gives the eye a second cue along that
    // upper edge — the panel border now sits on top of a strip
    // that is *also* outlined, doubling the contrast where it was
    // weakest. The bottom edge of this rectangle additionally
    // serves as the divider between the header text and the
    // first section caption underneath.
    //
    // Top corners get the same 6 px radius as the outer panel
    // (`mux_panel_style`) so the rounded plate-cutout shape
    // continues seamlessly through the header strip; the previous
    // square top corners were overwriting the panel's rounded
    // upper edge with a 90° rectangle, which the user flagged as
    // "у верхней части рамки нет скругления". Bottom corners stay
    // square because the strip butts directly into the next
    // section caption — a rounded bottom would round into a
    // sibling row and leave a visible gap on either side.
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
    // Multiplexer outer panel: border only, no resting fill.
    surface_style(None, 6.0, 1.0, TOKYO_BORDER)
}

/// Chrome for individual chips inside the multiplexer panel — the
/// W/Z scratch pair and the SP/PC inline readouts use the same
/// outline-only resting chrome as `mux_panel_style`.
/// The outer panel and every chip inside it read as "a frame full of
/// bordered slots cut into the plate" instead of "a card holding a
/// stack of darker cards". 6 px corner radius matches
/// `mux_panel_style` so the chips visually echo the parent frame at a
/// smaller scale.
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

pub(crate) fn opcode_dropdown_style(_theme: &Theme) -> container::Style {
    // Match the surrounding board panels (memory list, register editor, etc.)
    // so the floating picker reads as part of the same surface instead of a
    // darker pop-up sitting on top of it.
    //
    // The opcode picker floats over the memory list, so all four
    // corners need the same radius. A square top edge made the search
    // popup read as if its top border had been clipped off.
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

pub(crate) fn memory_row_container_style(selected: bool, halted: bool) -> container::Style {
    // Halted rows take precedence over the regular selected/unselected
    // styling: when the program ended on HLT, the row that holds that
    // opcode lights up red so the user sees at a glance which
    // instruction terminated execution. Only the fill is recoloured —
    // no extra border — to match the visual weight of the regular
    // selection highlight (per user feedback: a 1px border on top of
    // the surrounding row chrome read as noisy).
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
            // Round only the highlighted row; the others stay flat so the
            // 1-pixel separators between rows still meet edge-to-edge.
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
