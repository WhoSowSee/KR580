//! Control-lamp strip on the schematic plate (F1, F2, SYNC, READY, …).
//!
//! Eleven indicator lamps that reflect live CPU pin state, rendered as a
//! row of dot + vertical caption. Lives in its own module because:
//!
//! 1. The label-rotation pipeline owns a small SVG-generation helper
//!    that has no business sitting in the main schematic file.
//! 2. `crates/ui/src/view/schematic.rs` was bumping the workspace's
//!    400-line ceiling — pulling the lamp strip out keeps the parent
//!    file at a comfortable size without making the lamp logic harder
//!    to find (it stays one `use super::lamps::control_lamps;` away).
//!
//! ## Why SVG for the captions
//!
//! iced 0.14's `Text` widget cannot rotate. The reference panel the
//! user is matching against has its lamp captions running sideways
//! (text reads bottom-to-top, but each glyph stays upright relative
//! to the reader — the column rotates 90° counter-clockwise). The
//! cleanest way to get that under iced is to render each label as a
//! tiny SVG with a `<text transform="rotate(-90)">` block, then feed
//! the SVG through `iced::widget::svg`. Authoring with
//! `fill="currentColor"` lets the same `svg::Style { color: … }`
//! tinting we already use for the device chips paint the caption
//! `TOKYO_MUTED`/`TOKYO_TEXT` without re-encoding the SVG.
//!
//! Each label is fixed at compile time, so the eleven SVG handles are
//! built once into a `LazyLock` array and cloned cheaply into the view
//! tree on every paint.
//!
//! ## Signal mapping
//!
//! The reference KR-580 emulator the user is matching against parks the
//! chip at the idle T1 of the very first M1 fetch the moment the window
//! opens — so F2 / SYNC / READY / INTE / WR all light at startup, even
//! before a single instruction has been clocked. The user filed exactly
//! that mismatch ("почему в оригинальном эмуляторе по умолчанию горят
//! F2, SYNC, READY, INTE, WR а у меня только READY"), so our mapping
//! treats `tact_phase.is_none()` (the between-instruction idle the rest
//! of the UI calls "Такт 1") as that same opening T1 silhouette. The
//! pins flip to their phase-driven values only while a real `step_tact`
//! walk is in flight.
//!
//! | Lamp  | Meaning on the real 8080      | Source on `Cpu8080State` (idle: `tact_phase == None`) |
//! |-------|-------------------------------|-------------------------------------------------------|
//! | F2    | Clock phase 2                 | idle: on; walking: even `tact_phase`                  |
//! | F1    | Clock phase 1                 | idle: off; walking: odd `tact_phase`                  |
//! | SYNC  | Status byte latched (T1)      | idle: on; walking: `tact_phase == Some(0)`            |
//! | READY | Memory/IO acknowledged        | `!halted` (we do not stall the bus)                   |
//! | WAIT  | CPU is waiting (or halted)    | `halted`                                              |
//! | HOLD  | DMA hold request              | always `false` — DMA is not modeled                   |
//! | INT   | Interrupt request line high   | `interrupt_request_pending`                           |
//! | INTE  | Interrupts enabled            | `interrupt_enable` OR idle (matches reference T1)     |
//! | DBIN  | Data bus input strobe         | always `false` — machine-cycle state not modeled      |
//! | WR    | Write strobe                  | idle: on (M1 fetch drives WR low/active); off otherwise |
//! | HLDA  | Hold acknowledge              | always `false` — DMA is not modeled                   |
//!
//! HOLD / HLDA / DBIN stay dark on purpose: the emulator runs at the
//! instruction-boundary level and does not surface the machine cycle
//! state these pins toggle on. Showing them as static off is more
//! honest than driving them with an approximation that does not match
//! the schoolbook timing diagrams.

use std::sync::LazyLock;

use iced::widget::{Row, column, svg};
use iced::{Element, Length, alignment};
use k580_core::Cpu8080State;

use super::theme::{TOKYO_RED, TOKYO_TEXT, mono_text};
use crate::app::Message;

/// Geometry of a single lamp column. Width is generous enough to fit
/// the longest caption (`READY`, `HLDA`) at 9 px after rotation, plus
/// breathing room. Height of the SVG label is enough for a 5-letter
/// word at the same point size.
const LAMP_WIDTH: f32 = 18.0;
const LABEL_HEIGHT: f32 = 36.0;
const LABEL_FONT_SIZE: f32 = 9.0;

/// Eleven labels in the order the reference panel paints them, paired
/// with the live signal each lamp reflects. Read by `control_lamps`
/// to walk the `LAMP_LABEL_SVGS` array in lockstep.
const LAMP_ORDER: [&str; 11] = [
    "F2", "F1", "SYNC", "READY", "WAIT", "HOLD", "INT", "INTE", "DBIN", "WR", "HLDA",
];

/// SVG bytes for each lamp caption, rotated +90°. Built once on first
/// access — the SVG string is small (a few hundred bytes per label),
/// and `svg::Handle::from_memory` is the same primitive `icons.rs`
/// uses for the static SVGs embedded with `include_bytes!`.
///
/// Authoring details:
/// - `viewBox` is `0 0 LAMP_WIDTH LABEL_HEIGHT` so iced's resvg
///   backend lays the rotated text into the same box the widget
///   reserves with `width`/`height`.
/// - `<text fill="currentColor">` keeps the tinting pipeline working
///   (set once via `svg::Style { color: … }` at the call site).
/// - The transform pipeline is `translate(centre, baseline) rotate(90)`,
///   so the rotation pivots around the visual centre of each
///   glyph row instead of the origin (which would push the text off
///   the canvas). +90° (clockwise) is what produces the "left-to-right
///   reading top-to-bottom" orientation the user asked for —
///   `rotate(-90)` (counter-clockwise) reads bottom-to-top, which
///   the reference panel does NOT use.
/// - `text-anchor="middle"` plus `dominant-baseline="middle"` keeps
///   the text centred within the SVG box once rotated.
static LAMP_LABEL_SVGS: LazyLock<[svg::Handle; 11]> = LazyLock::new(|| {
    LAMP_ORDER.map(|label| {
        let cx = LAMP_WIDTH / 2.0;
        let cy = LABEL_HEIGHT / 2.0;
        let body = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" \
                 width=\"{w}\" height=\"{h}\" viewBox=\"0 0 {w} {h}\">\
               <text x=\"0\" y=\"0\" \
                     transform=\"translate({cx} {cy}) rotate(90)\" \
                     text-anchor=\"middle\" dominant-baseline=\"middle\" \
                     font-family=\"sans-serif\" font-size=\"{fs}\" \
                     font-weight=\"600\" fill=\"currentColor\">{label}</text>\
             </svg>",
            w = LAMP_WIDTH,
            h = LABEL_HEIGHT,
            cx = cx,
            cy = cy,
            fs = LABEL_FONT_SIZE,
        );
        svg::Handle::from_memory(body.into_bytes())
    })
});

/// State of a single lamp — `true` lights the dot red, `false` keeps
/// it the resting `TOKYO_TEXT` tone (same idiom as `flag_dot` in the
/// schematic module).
///
/// Idle (`tact_phase == None`) is treated as the opening T1 of the
/// first M1 fetch — the very state the reference KR-580 emulator
/// shows the moment the window opens, with F2 / SYNC / READY / INTE /
/// WR all lit. The user filed exactly that mismatch, and the
/// alignment matters because "идол по умолчанию" is what every
/// schoolbook diagram pictures alongside the chip pinout. Phase-driven
/// values kick in only while a real `step_tact` walk is in flight,
/// where the user is asking the panel to mirror the live timing
/// diagram of the current machine cycle instead of the at-rest
/// silhouette.
fn lamp_states(cpu: &Cpu8080State) -> [bool; 11] {
    let phase = cpu.tact_phase;
    let idle = phase.is_none();
    // F1/F2 follow the parity of `tact_phase` while the CPU is being
    // walked tact-by-tact. At rest (between instructions) the panel
    // mirrors the reference emulator: F2 lit, F1 dark — the opening
    // T1 silhouette of the first M1 fetch.
    let f2 = idle || matches!(phase, Some(p) if p % 2 == 1);
    let f1 = matches!(phase, Some(p) if p % 2 == 0);
    // SYNC also burns at idle, since the at-rest panel parks at the
    // T1 status-latch edge. During a tact walk it follows phase 0,
    // matching the schoolbook timing diagram.
    let sync = idle || phase == Some(0);
    let ready = !cpu.halted;
    let wait = cpu.halted;
    let hold = false;
    let int = cpu.interrupt_request_pending;
    // INTE lights at idle for the same "match the reference panel at
    // power-on" reason — the reference emulator does not gate the
    // INTE indicator on the EI flip-flop the way real silicon does.
    // During a tact walk the panel returns to telegraphing the actual
    // `interrupt_enable` state so the EI/DI mnemonics stay legible.
    let inte = idle || cpu.interrupt_enable;
    let dbin = false;
    // WR is held active (low on the real chip; "lit" on the panel) at
    // idle to match the reference KR-580's at-rest pin silhouette. We
    // do not surface the per-machine-cycle WR strobe during a tact
    // walk, so the indicator drops back to dark there — same honesty
    // policy as DBIN / HOLD / HLDA.
    let wr = idle;
    let hlda = false;
    [f2, f1, sync, ready, wait, hold, int, inte, dbin, wr, hlda]
}

/// Builds the row of eleven control lamps. Pure function of the CPU
/// snapshot — the call site (`schematic_panel`) hands it the same
/// borrowed `Cpu8080State` it already uses for the rest of the
/// schematic.
pub(super) fn control_lamps(cpu: &Cpu8080State) -> Element<'_, Message> {
    let states = lamp_states(cpu);
    let children = LAMP_ORDER
        .iter()
        .copied()
        .zip(LAMP_LABEL_SVGS.iter().cloned())
        .zip(states.iter().copied())
        .map(|((label, handle), active)| control_lamp(label, handle, active));

    Row::with_children(children)
        .spacing(7)
        .align_y(alignment::Vertical::Center)
        .into()
}

/// Single lamp column: vertical caption on top, dot below. The user
/// asked for the caption above the lamp — it puts the label-then-dot
/// reading order in the same direction the eye naturally walks the
/// row (top-to-bottom inside the column, left-to-right across the
/// strip), and matches the reference KR-580 panel's silhouette where
/// each pin name floats over its indicator.
///
/// The dot uses the same `flag_dot` idiom — `TOKYO_RED` when the
/// signal is asserted, `TOKYO_TEXT` when idle — so the eye reads the
/// lamp row with the same vocabulary it just learned for the flag
/// strip (Z/S/P/C/AC).
///
/// The caption is rendered as an SVG fed by the `LAMP_LABEL_SVGS`
/// cache. `svg::Style { color }` paints the label `TOKYO_TEXT` so it
/// stays legible against the schematic plate without competing with
/// the lit dot for attention.
fn control_lamp(
    label: &'static str,
    handle: svg::Handle,
    active: bool,
) -> Element<'static, Message> {
    let dot_color = if active { TOKYO_RED } else { TOKYO_TEXT };
    let label_widget = svg(handle)
        .width(Length::Fixed(LAMP_WIDTH))
        .height(Length::Fixed(LABEL_HEIGHT))
        .style(|_theme, _status| svg::Style {
            color: Some(TOKYO_TEXT),
        });

    // `mono_text` for the dot keeps the row visually identical to
    // `flag_dot` (the bullet glyph is rendered with the same monospace
    // face). `_label` is unused at runtime — the SVG carries the text
    // — but we keep it on the function signature so call-site readers
    // see which lamp each entry corresponds to without chasing the
    // SVG cache.
    let _ = label;

    column![
        label_widget,
        mono_text("●", 14, dot_color).align_x(alignment::Horizontal::Center),
    ]
    .width(Length::Fixed(LAMP_WIDTH))
    .spacing(2)
    .align_x(alignment::Horizontal::Center)
    .into()
}
