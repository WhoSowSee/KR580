//! Vector icons used by the UI.
//!
//! Each handle is built once from the bytes embedded at compile time
//! (`include_bytes!`) and then cloned cheaply by callers. iced's
//! `svg::Handle` is reference-counted internally, so the cost of a
//! `clone()` is bounded; the `LazyLock` guard exists so the first paint
//! does not allocate the handle inside the layout pass.

use std::sync::LazyLock;

use iced::widget::svg;

/// Bytes for an action-panel icon, located under `assets/icons/actions/`.
macro_rules! action_icon_bytes {
    ($name:literal) => {
        include_bytes!(concat!("../../../../assets/icons/actions/", $name, ".svg"))
    };
}

static PLAY: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("play").as_slice()));
static PAUSE: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("pause").as_slice()));
static STEP_FORWARD: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("step-forward").as_slice()));
static REDO_DOT: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("redo-dot").as_slice()));
static RESET_RAM: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("reset-ram").as_slice()));
static REFRESH_CCW: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("refresh-ccw").as_slice()));
static RESET_REGISTERS: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("reset-registers").as_slice()));
static CHEVRONS_RIGHT: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("chevrons-right").as_slice()));

/// Lucide `play` — solid right-pointing triangle. Used for "run program".
pub(super) fn play() -> svg::Handle {
    PLAY.clone()
}

/// Lucide `pause` — twin vertical bars. Used for the "stop / pause" toggle
/// state of the run button: when the user has armed the run state we swap
/// the play glyph for this one to mirror the reference KR-580 emulator.
pub(super) fn pause() -> svg::Handle {
    PAUSE.clone()
}

/// Lucide `step-forward` — triangle with a vertical stop bar. Used for
/// "execute one instruction".
pub(super) fn step_forward() -> svg::Handle {
    STEP_FORWARD.clone()
}

/// Lucide `redo-dot` — a curved arrow with a dot under it. Used for
/// "execute one machine tact".
pub(super) fn redo_dot() -> svg::Handle {
    REDO_DOT.clone()
}

/// Lucide `refresh-ccw` — counter-clockwise circular arrow pair. Used as
/// the running-state replacement for the "step instruction" glyph: while
/// a program is actually executing on the CPU, the second action button
/// becomes a "restart" affordance that resets the registers/flags and
/// re-runs the program from `0x0000`.
pub(super) fn refresh_ccw() -> svg::Handle {
    REFRESH_CCW.clone()
}

/// Custom KR-580 glyph: a memory module silhouette with a circular
/// reset arrow above it. Used for "reset RAM".
pub(super) fn reset_ram() -> svg::Handle {
    RESET_RAM.clone()
}

/// Custom KR-580 glyph: stacked register cells with a circular reset
/// arrow on the right. Used for "reset registers".
pub(super) fn reset_registers() -> svg::Handle {
    RESET_REGISTERS.clone()
}

/// Lucide `chevrons-right` — twin right-pointing chevrons. Used as the
/// glyph for the "apply" (Enter) buttons next to the memory cell and
/// register value editors. Reads as "commit and move on".
pub(super) fn chevrons_right() -> svg::Handle {
    CHEVRONS_RIGHT.clone()
}
