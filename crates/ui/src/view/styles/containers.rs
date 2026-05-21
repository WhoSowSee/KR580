//! Container styles: backgrounds, borders, and a couple of shared
//! primitive helpers (`surface_style`, `solid_style`) that other modules
//! reuse to assemble their own variants.

use iced::widget::container;
use iced::{Background, Border, Color, Theme};

use super::super::theme::{
    TOKYO_BG, TOKYO_BLUE, TOKYO_BOARD, TOKYO_BORDER, TOKYO_MAGENTA, TOKYO_SURFACE, TOKYO_SURFACE_2,
    TOKYO_TEXT,
};

pub(crate) fn app_style(_theme: &Theme) -> container::Style {
    surface_style(Some(TOKYO_BOARD), 0.0, 0.0, Color::TRANSPARENT)
}

pub(crate) fn menu_bar_style(_theme: &Theme) -> container::Style {
    surface_style(Some(TOKYO_BOARD), 6.0, 1.0, TOKYO_BORDER)
}

pub(crate) fn board_style(_theme: &Theme) -> container::Style {
    surface_style(Some(TOKYO_BOARD), 8.0, 1.0, TOKYO_BORDER)
}

pub(crate) fn panel_style(theme: &Theme) -> container::Style {
    board_style(theme)
}

pub(crate) fn inset_style(_theme: &Theme) -> container::Style {
    surface_style(Some(TOKYO_SURFACE), 6.0, 1.0, TOKYO_BORDER)
}

pub(crate) fn schematic_block_style(_theme: &Theme) -> container::Style {
    surface_style(
        Some(Color::from_rgba8(0x24, 0x26, 0x3A, 0.92)),
        6.0,
        1.0,
        TOKYO_BORDER,
    )
}

pub(crate) fn alu_style(_theme: &Theme) -> container::Style {
    surface_style(
        Some(Color::from_rgb8(0x25, 0x27, 0x3D)),
        6.0,
        1.5,
        TOKYO_MAGENTA,
    )
}

pub(crate) fn mux_header_style(_theme: &Theme) -> container::Style {
    surface_style(Some(TOKYO_SURFACE_2), 0.0, 0.0, TOKYO_BORDER)
}

pub(crate) fn mux_panel_style(_theme: &Theme) -> container::Style {
    surface_style(Some(TOKYO_SURFACE), 6.0, 1.0, TOKYO_BORDER)
}

pub(crate) fn legend_label_style(_theme: &Theme) -> container::Style {
    surface_style(Some(TOKYO_BOARD), 0.0, 0.0, Color::TRANSPARENT)
}

/// Thin vertical line used between icon-button groups in the action
/// panel. Painted with the same colour as the surrounding panel border
/// so it reads as a continuation of the frame rather than as a foreign
/// pixel column.
pub(crate) fn divider_style(_theme: &Theme) -> container::Style {
    surface_style(Some(TOKYO_BORDER), 0.0, 0.0, Color::TRANSPARENT)
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

    surface_style(Some(TOKYO_BG), 6.0, 1.0, border_color)
}

pub(crate) fn opcode_dropdown_style(_theme: &Theme) -> container::Style {
    // Match the surrounding board panels (memory list, register editor, etc.)
    // so the floating picker reads as part of the same surface instead of a
    // darker pop-up sitting on top of it.
    surface_style(Some(TOKYO_BOARD), 7.0, 1.0, TOKYO_BORDER)
}

pub(crate) fn memory_row_container_style(selected: bool) -> container::Style {
    let background = if selected {
        Some(Background::Color(Color::from_rgba8(0x7A, 0xA2, 0xF7, 0.18)))
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
