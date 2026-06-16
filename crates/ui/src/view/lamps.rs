//! Control-lamp strip on the schematic plate (F1, F2, SYNC, READY, …).
//!
//! Idle (`tact_phase == None`) mirrors the reference KR-580 emulator's
//! at-rest silhouette: F2 / SYNC / READY / INTE / WR lit. Pins flip to
//! phase-driven values only during a `step_tact` walk. HOLD / HLDA /
//! DBIN stay dark – we don't model the machine-cycle pins.

use iced::widget::{Row, column, tooltip};
use iced::{Element, Length, alignment};
use k580_core::Cpu8080State;

use super::theme::{TOKYO_RED, TOKYO_TEXT, mono_text};
use crate::app::Message;
use crate::i18n::{Key, Lang};

const LAMP_WIDTH: f32 = 44.0;

const LAMP_ORDER: [(&str, Key); 11] = [
    ("F2", Key::LampF2),
    ("F1", Key::LampF1),
    ("SYNC", Key::LampSync),
    ("READY", Key::LampReady),
    ("WAIT", Key::LampWait),
    ("HOLD", Key::LampHold),
    ("INT", Key::LampInt),
    ("INTE", Key::LampInte),
    ("DBIN", Key::LampDbin),
    ("WR", Key::LampWr),
    ("HLDA", Key::LampHlda),
];

fn lamp_states(cpu: &Cpu8080State) -> [bool; 11] {
    let phase = cpu.tact_phase;
    let idle = phase.is_none();
    let f2 = idle || matches!(phase, Some(p) if p % 2 == 1);
    let f1 = matches!(phase, Some(p) if p % 2 == 0);
    let sync = idle || phase == Some(0);
    let ready = !cpu.halted;
    let wait = cpu.halted;
    let hold = false;
    let int = cpu.interrupt_request_pending;
    let inte = idle || cpu.interrupt_enable;
    let dbin = false;
    let wr = idle;
    let hlda = false;
    [f2, f1, sync, ready, wait, hold, int, inte, dbin, wr, hlda]
}

pub(super) fn control_lamps(cpu: &Cpu8080State, lang: Lang) -> Element<'_, Message> {
    let states = lamp_states(cpu);
    let children = LAMP_ORDER
        .iter()
        .copied()
        .zip(states.iter().copied())
        .map(|((label, key), active)| control_lamp(label, lang.t(key), active));

    Row::with_children(children)
        .spacing(0)
        .align_y(alignment::Vertical::Center)
        .into()
}

fn control_lamp(
    label: &'static str,
    hint: &'static str,
    active: bool,
) -> Element<'static, Message> {
    let dot_color = if active { TOKYO_RED } else { TOKYO_TEXT };

    let face: Element<'static, Message> = column![
        mono_text(label, 9, TOKYO_TEXT).align_x(alignment::Horizontal::Center),
        mono_text("●", 16, dot_color).align_x(alignment::Horizontal::Center),
    ]
    .width(Length::Fixed(LAMP_WIDTH))
    .spacing(2)
    .align_x(alignment::Horizontal::Center)
    .into();

    tooltip(
        face,
        super::tooltips::long_tooltip_body(hint),
        tooltip::Position::Bottom,
    )
    .gap(4.0)
    .padding(12.0)
    .delay(super::tooltips::EXPLANATORY_TOOLTIP_DELAY)
    .snap_within_viewport(true)
    .into()
}
