//! Container styles: backgrounds, borders, and a couple of shared
//! primitive helpers (`surface_style`, `solid_style`) that other modules
//! reuse to assemble their own variants.

use iced::widget::container;
use iced::{Background, Border, Color, Theme};

use super::super::theme::{
    TOKYO_BG, TOKYO_BLUE, TOKYO_BOARD, TOKYO_BORDER, TOKYO_RED, TOKYO_SURFACE, TOKYO_TEXT,
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
/// already provides its own internal visual language (mux frame, ALU
/// frame, schematic block readouts), so wrapping it in another framed
/// surface read as redundant. The fill stays so the schematic still
/// sits on the same `TOKYO_BOARD` plate as everything else and the
/// background does not shift between panes.
pub(crate) fn schematic_board_style(_theme: &Theme) -> container::Style {
    surface_style(Some(TOKYO_BOARD), 0.0, 0.0, Color::TRANSPARENT)
}

/// Hairline divider used under the menu bar in place of a full bubble
/// border. Renders as a 1-px container filled with the regular
/// `TOKYO_BORDER` tone, so the seam between the title bar and the
/// schematic plate underneath stays visible without bringing back the
/// rounded-corner chrome.
pub(crate) fn menu_bar_divider_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(TOKYO_BORDER)),
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

pub(crate) fn inset_style(_theme: &Theme) -> container::Style {
    surface_style(Some(TOKYO_SURFACE), 6.0, 1.0, TOKYO_BORDER)
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

/// Tone shared by every framed slot on the schematic plate. Lives in
/// one named constant so `schematic_block_style`, `mux_panel_style`,
/// `mux_chip_style`, and `schematic_block_button_style` (in
/// `buttons.rs`) all paint the same exact swatch — a stray
/// hand-typed `0x1C, 0x1E, 0x2E` somewhere else would drift this tone
/// over time without anyone noticing. `#1C1E2E @ 0.92` sits one notch
/// darker than `TOKYO_BG` (`#1A1B26`) and lighter than `TOKYO_BOARD`
/// (`#121320`) — the chips read as recessed slots cut into the plate
/// rather than lifted cards on top of it.
pub(crate) const SCHEMATIC_BLOCK_FILL: Color = Color {
    r: 0x1C as f32 / 255.0,
    g: 0x1E as f32 / 255.0,
    b: 0x2E as f32 / 255.0,
    a: 0.92,
};

pub(crate) fn schematic_block_style(_theme: &Theme) -> container::Style {
    // Every framed slot on the schematic plate (Цикл/Такт, the speed
    // switch, the device chips, the schematic readouts — Буфер
    // данных / Регистр признаков / Регистр команд / PSW, the Мультиплексор
    // panel and its inner chips) routes through this style so the
    // panels share one continuous tone. Fill is the shared
    // `SCHEMATIC_BLOCK_FILL` constant — see its doc comment for why.
    surface_style(Some(SCHEMATIC_BLOCK_FILL), 6.0, 1.0, TOKYO_BORDER)
}

pub(crate) fn mux_header_style(_theme: &Theme) -> container::Style {
    // Header strip of the multiplexer panel: the strip used to wear
    // `TOKYO_BOARD` so it merged with the surrounding plate, but the
    // plate-coloured strip is what made the Мультиплексор panel read
    // brighter than its siblings on the rest of the schematic. The
    // strip now wears `SCHEMATIC_BLOCK_FILL` — same tone as the panel
    // body underneath and as `schematic_block_style` everywhere
    // else — so the whole multiplexer family shares one swatch with
    // the rest of the left-panel chrome.
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
        background: Some(Background::Color(SCHEMATIC_BLOCK_FILL)),
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
    // Multiplexer outer panel — same `SCHEMATIC_BLOCK_FILL` as every
    // other framed slot on the schematic plate so the panel reads as
    // a sibling of the surrounding chips, not a card lifted on top of
    // them. The user explicitly asked for "сделай заливку как у
    // других блоков"; the previous `TOKYO_BOARD` fill matched the
    // plate behind the chips, which is why the Мультиплексор
    // looked uniformly brighter than its neighbours.
    surface_style(Some(SCHEMATIC_BLOCK_FILL), 6.0, 1.0, TOKYO_BORDER)
}

/// Chrome for individual chips inside the multiplexer panel — the
/// W/Z scratch pair, the SP/PC inline readouts, and (via
/// `mux_button_style`'s neutral status) the regular РОН buttons all
/// land on the same `SCHEMATIC_BLOCK_FILL` as `mux_panel_style`. The
/// outer panel and every chip inside it sharing one surface tone is
/// what makes the panel read as "a frame full of bordered slots cut
/// into the plate" instead of "a card holding a stack of darker
/// cards". 6 px corner radius matches `mux_panel_style` so the
/// chips visually echo the parent frame at a smaller scale.
pub(crate) fn mux_chip_style(_theme: &Theme) -> container::Style {
    surface_style(Some(SCHEMATIC_BLOCK_FILL), 6.0, 1.0, TOKYO_BORDER)
}

/// Variant of `mux_chip_style` used by the interactive РОН register
/// chips that the user can click to switch the register editor.
/// Behaves like `mux_chip_style` for the unselected state, and
/// flips the fill to the same `TOKYO_BLUE @ 0.18` wash that
/// `memory_row_container_style` paints on the selected memory row
/// when `selected` is `true`. The frame stays neutral
/// (`TOKYO_BORDER`) in both states — selection reads from the fill
/// alone, per the user's explicit "только заливка должна быть
/// синей" feedback.
///
/// Why a `container::Style` and not a `button::Style`: the РОН
/// chips were historically `button`s, which only fire `on_press`
/// on `Status::Released`. The user reported a sluggish feel
/// switching between registers — between the press and the
/// release the eye reads the latency as input lag. Routing the
/// click through a `mouse_area` instead makes it fire on the
/// `Pressed` edge (matching the memory-row cells), and the
/// surrounding chrome becomes a plain `container`. Keeping the
/// style here lets the chip wear exactly the same dress as the
/// W/Z static chips next to it, but driven by a snappier gesture
/// pipeline.
pub(crate) fn mux_register_chip_style(selected: bool) -> container::Style {
    if selected {
        // Byte-for-byte identical to `memory_row_container_style`'s
        // selected fill (`Color::from_rgba8(0x7A, 0xA2, 0xF7, 0.18)`).
        // Same hue, same alpha, same role: "this row / chip is the
        // active selection". Sharing the wash keeps the visual
        // idiom transferable between panels.
        container::Style {
            text_color: Some(TOKYO_TEXT),
            background: Some(Background::Color(Color::from_rgba8(0x7A, 0xA2, 0xF7, 0.18))),
            border: Border {
                radius: 6.0.into(),
                width: 1.0,
                color: TOKYO_BORDER,
            },
            ..container::Style::default()
        }
    } else {
        mux_chip_style(&Theme::Dark)
    }
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

    surface_style(Some(TOKYO_BG), 6.0, 1.0, border_color)
}

pub(crate) fn opcode_dropdown_style(_theme: &Theme) -> container::Style {
    // Match the surrounding board panels (memory list, register editor, etc.)
    // so the floating picker reads as part of the same surface instead of a
    // darker pop-up sitting on top of it.
    //
    // Top corners are square, bottom corners keep the 7 px radius. The
    // dropdown's top edge always anchors against another surface — the
    // menu bar's bottom hairline for the file/MP menus, the memory row
    // for the opcode picker — so a rounded top edge would round *into*
    // that anchor and break the "panel hangs off the line" illusion.
    // Squaring just the top edge lets the divider/row meet the frame
    // edge-to-edge while the bottom of the panel still reads as a
    // discrete bubble floating over the schematic.
    container::Style {
        text_color: Some(TOKYO_TEXT),
        background: Some(Background::Color(TOKYO_BOARD)),
        border: Border {
            radius: iced::border::Radius {
                top_left: 0.0,
                top_right: 0.0,
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
