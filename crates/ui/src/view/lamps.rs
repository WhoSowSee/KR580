//! Control-lamp strip on the schematic plate (F1, F2, SYNC, READY, ...).
//!
//! The lamps are derived from the active T-state and machine-cycle kind.
//! HOLD stays dark because DMA requests are not part of the core model.

use iced::widget::{Row, column, tooltip};
use iced::{Element, Length, alignment};
use k580_core::{Cpu8080State, MachineCycleKind, kind_at, layout_for, position_for};

use super::theme::{mono_text, tokyo_inactive_lamp, tokyo_red, tokyo_text};
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
    let cycle = active_cycle(cpu);
    let active_phase = cpu
        .last_completed_tact_phase
        .filter(|_| cpu.tact_walk_active());
    let idle = cycle.is_none();
    let f2 = idle || active_phase.is_some_and(|phase| phase % 2 == 1);
    let f1 = active_phase.is_some_and(|phase| phase % 2 == 0);
    let sync = idle || cycle.is_some_and(|(_, t_in_cycle)| t_in_cycle == 1);
    let wait = cpu.halted || cycle.is_some_and(|(kind, _)| kind == MachineCycleKind::HaltAck);
    let ready = !wait;
    let hold = false;
    let int = cpu.interrupt_request_pending;
    let inte = cpu.interrupt_enable;
    let dbin = cycle.is_some_and(|(kind, t_in_cycle)| is_read_cycle(kind) && t_in_cycle >= 2);
    let wr = cycle.is_some_and(|(kind, t_in_cycle)| is_write_cycle(kind) && t_in_cycle >= 2);
    let hlda = wait;
    [f2, f1, sync, ready, wait, hold, int, inte, dbin, wr, hlda]
}

fn active_cycle(cpu: &Cpu8080State) -> Option<(MachineCycleKind, u8)> {
    let phase = cpu
        .last_completed_tact_phase
        .filter(|_| cpu.tact_walk_active())?;
    let opcode = cpu.timing_opcode();
    if opcode == 0x76 && phase >= 4 {
        return Some((MachineCycleKind::HaltAck, phase - 3));
    }
    let layout = layout_for(opcode);
    let taken = cpu.timing_branch_taken(layout, phase);
    let position = position_for(layout, taken, phase)?;
    let kind = kind_at(opcode, (position.m_cycle - 1) as usize, taken)?;
    Some((kind, position.t_in_cycle))
}

fn is_read_cycle(kind: MachineCycleKind) -> bool {
    matches!(
        kind,
        MachineCycleKind::M1Fetch
            | MachineCycleKind::MemoryRead
            | MachineCycleKind::StackRead
            | MachineCycleKind::IoRead
            | MachineCycleKind::InterruptAck
    )
}

fn is_write_cycle(kind: MachineCycleKind) -> bool {
    matches!(
        kind,
        MachineCycleKind::MemoryWrite | MachineCycleKind::StackWrite | MachineCycleKind::IoWrite
    )
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
    let dot_color = if active {
        tokyo_red()
    } else {
        tokyo_inactive_lamp()
    };

    let face: Element<'static, Message> = column![
        mono_text(label, 9, tokyo_text()).align_x(alignment::Horizontal::Center),
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

#[cfg(test)]
mod tests {
    use super::lamp_states;
    use k580_core::{Cpu8080State, NullBus};

    fn step_tacts(cpu: &mut Cpu8080State, bus: &mut NullBus, count: u8) {
        for _ in 0..count {
            cpu.step_tact(bus).unwrap();
        }
    }

    #[test]
    fn cold_start_shows_t1_fetch_baseline_without_fake_bus_strobes() {
        let cpu = Cpu8080State::default();
        assert_eq!(
            lamp_states(&cpu),
            [
                true, false, true, true, false, false, false, false, false, false, false,
            ]
        );
    }

    #[test]
    fn read_and_write_strobes_follow_machine_cycles() {
        let mut cpu = Cpu8080State::default();
        cpu.memory.write(0, 0xD3);
        cpu.memory.write(1, 0x04);
        let mut bus = NullBus::default();

        step_tacts(&mut cpu, &mut bus, 7);
        assert!(lamp_states(&cpu)[8]);
        assert!(!lamp_states(&cpu)[9]);

        cpu.step_tact(&mut bus).unwrap();
        assert!(lamp_states(&cpu)[2]);
        assert!(!lamp_states(&cpu)[9]);

        cpu.step_tact(&mut bus).unwrap();
        assert!(lamp_states(&cpu)[9]);
    }

    #[test]
    fn hlt_second_cycle_lights_wait_and_hlda() {
        let mut cpu = Cpu8080State::default();
        cpu.memory.write(0, 0x76);
        let mut bus = NullBus::default();

        step_tacts(&mut cpu, &mut bus, 5);
        let states = lamp_states(&cpu);
        assert!(!states[3]);
        assert!(states[4]);
        assert!(states[10]);
    }
}
