//! Control-lamp strip on the schematic plate (F1, F2, SYNC, READY, …).
//!
//! Eleven indicator lamps that reflect live CPU pin state, rendered as
//! horizontal label + dot columns. The logic lives in its own module so
//! the main schematic file stays focused on panel composition.
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

use iced::widget::{Row, column};
use iced::{Element, Length, alignment};
use k580_core::Cpu8080State;

use super::theme::{TOKYO_RED, TOKYO_TEXT, mono_text};
use crate::app::Message;

/// Geometry of a single lamp column. Wide enough for READY / HLDA as
/// horizontal 8 px labels inside the framed "Сигналы управления" row.
const LAMP_WIDTH: f32 = 44.0;

/// Eleven labels in the order the reference panel paints them.
const LAMP_ORDER: [&str; 11] = [
    "F2", "F1", "SYNC", "READY", "WAIT", "HOLD", "INT", "INTE", "DBIN", "WR", "HLDA",
];

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
        .zip(states.iter().copied())
        .map(|(label, active)| control_lamp(label, active));

    Row::with_children(children)
        .spacing(0)
        .align_y(alignment::Vertical::Center)
        .into()
}

/// Single lamp column: horizontal caption on top, dot below. The user
/// asked for the caption above the lamp — it puts the label-then-dot
/// reading order in the same direction the eye naturally walks the
/// row.
///
/// The dot uses the same `flag_dot` idiom — `TOKYO_RED` when the
/// signal is asserted, `TOKYO_TEXT` when idle — so the eye reads the
/// lamp row with the same vocabulary it just learned for the flag
/// strip (Z/S/P/C/AC).
fn control_lamp(label: &'static str, active: bool) -> Element<'static, Message> {
    let dot_color = if active { TOKYO_RED } else { TOKYO_TEXT };

    column![
        mono_text(label, 9, TOKYO_TEXT).align_x(alignment::Horizontal::Center),
        mono_text("●", 16, dot_color).align_x(alignment::Horizontal::Center),
    ]
    .width(Length::Fixed(LAMP_WIDTH))
    .spacing(2)
    .align_x(alignment::Horizontal::Center)
    .into()
}
