//! Status register (T1 status byte + Russian label) for the schematic.
//! The byte mirrors Intel 8080A datasheet "Status Information":
//!
//! ```text
//! D7  D6  D5    D4    D3   D2  D1   D0
//! MEM INP M1   OUT   HLTA STK WO   INTA
//! R           Read         Bar
//! ```

use super::styles::status_tooltip_style;
use super::theme::{TOKYO_BLUE, TOKYO_TEXT, ui_text};
use super::tooltips::EXPLANATORY_TOOLTIP_DELAY;
use crate::app::Message;
use crate::i18n::{Key, Lang};
use iced::widget::{Space, column, container, row, tooltip};
use iced::{Element, Length, Padding, alignment};
use k580_core::{
    Cpu8080State, MachineCycleKind, MachineCycleLayout, kind_at, layout_for, position_for,
};

/// For conditional opcodes the taken/not-taken branches differ in
/// length, so the current `phase` uniquely identifies the branch.
fn branch_taken_from_phase(layout: MachineCycleLayout, phase: u8) -> bool {
    if let Some(not_taken) = layout.not_taken {
        let not_taken_total: u8 = not_taken.iter().sum();
        if phase < not_taken_total {
            return false;
        }
    }
    true
}

pub(super) fn derive_status_kind(cpu: &Cpu8080State) -> MachineCycleKind {
    // INTA before HLT: an INT raised while halted lifts HLT on the next
    // tact, so the status byte must reflect interrupt-ack already.
    if cpu.interrupt_request_pending && cpu.interrupt_enable {
        return MachineCycleKind::InterruptAck;
    }
    if cpu.halted {
        return MachineCycleKind::HaltAck;
    }

    // Cold start: nothing executed yet, but T1 of the first M1 must
    // already read as `M1Fetch` to match the reference panel.
    let Some(phase) = cpu.last_completed_tact_phase else {
        return MachineCycleKind::M1Fetch;
    };

    let layout = layout_for(cpu.last_fetched_opcode);
    let taken = branch_taken_from_phase(layout, phase);
    let Some(position) = position_for(layout, taken, phase) else {
        return MachineCycleKind::M1Fetch;
    };

    let m_cycle_idx = (position.m_cycle - 1) as usize;
    kind_at(cpu.last_fetched_opcode, m_cycle_idx, taken).unwrap_or(MachineCycleKind::M1Fetch)
}

fn status_bits(byte: u8) -> String {
    format!(
        "{}{}{}{} {}{}{}{}",
        (byte >> 7) & 1,
        (byte >> 6) & 1,
        (byte >> 5) & 1,
        (byte >> 4) & 1,
        (byte >> 3) & 1,
        (byte >> 2) & 1,
        (byte >> 1) & 1,
        byte & 1,
    )
}

pub(super) fn status_register_bits(cpu: &Cpu8080State) -> String {
    let kind = derive_status_kind(cpu);
    let byte = kind.status_byte();
    status_bits(byte)
}

fn status_register_tooltip_body_lines(cpu: &Cpu8080State, lang: Lang) -> [String; 2] {
    let kind = derive_status_kind(cpu);
    let label = match lang {
        Lang::Ru => kind.label_ru(),
        Lang::En => kind.label_en(),
    };
    [
        lang.t(Key::StatusByteHeader).to_owned(),
        format!("{} {label}", lang.t(Key::StatusPrefix)),
    ]
}

pub(super) fn status_register_tooltip<'a>(
    cpu: &Cpu8080State,
    face: impl Into<Element<'a, Message>>,
    lang: Lang,
) -> Element<'a, Message> {
    let [description, status_line] = status_register_tooltip_body_lines(cpu, lang);
    let prefix = lang.t(Key::StatusPrefix);
    let prefix_with_space = format!("{prefix} ");
    let status_label = status_line
        .strip_prefix(&prefix_with_space)
        .unwrap_or(&status_line)
        .to_owned();

    let body = container(
        column![
            ui_text(description, 12, TOKYO_TEXT),
            Space::new().height(Length::Fixed(6.0)),
            row![
                ui_text(format!("{prefix} "), 12, TOKYO_TEXT),
                ui_text(status_label, 12, TOKYO_BLUE),
            ]
            .spacing(0)
            .align_y(alignment::Vertical::Center),
        ]
        .width(Length::Fill),
    )
    .padding(Padding {
        top: 4.0,
        right: 8.0,
        bottom: 4.0,
        left: 8.0,
    })
    .max_width(280.0)
    .style(status_tooltip_style);

    tooltip(face, body, tooltip::Position::Bottom)
        .gap(4.0)
        .padding(12.0)
        .delay(EXPLANATORY_TOOLTIP_DELAY)
        .snap_within_viewport(true)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cpu_with(opcode: u8, phase: Option<u8>) -> Cpu8080State {
        let mut cpu = Cpu8080State::default();
        cpu.last_fetched_opcode = opcode;
        cpu.last_completed_tact_phase = phase;
        cpu
    }

    #[test]
    fn cold_start_is_m1_fetch() {
        let cpu = Cpu8080State::default();
        assert_eq!(derive_status_kind(&cpu), MachineCycleKind::M1Fetch);
    }

    #[test]
    fn status_register_bits_uses_t1_status_byte() {
        let cpu = Cpu8080State::default();
        assert_eq!(status_register_bits(&cpu), "1010 0010");
    }

    #[test]
    fn tooltip_body_does_not_repeat_status_register_bits() {
        let cpu = Cpu8080State::default();
        let lines = status_register_tooltip_body_lines(&cpu, Lang::Ru);
        assert!(
            !lines
                .iter()
                .any(|line| line.contains("Регистр состояния 1010 0010"))
        );
        assert!(lines.iter().any(|line| line == "Статус: Загрузка опкода"));

        let en_lines = status_register_tooltip_body_lines(&cpu, Lang::En);
        assert!(en_lines.iter().any(|line| line == "Status: Opcode fetch"));
    }

    #[test]
    fn halt_overrides_table() {
        let mut cpu = cpu_with(0x76, Some(3));
        cpu.halted = true;
        assert_eq!(derive_status_kind(&cpu), MachineCycleKind::HaltAck);
    }

    #[test]
    fn interrupt_overrides_halt() {
        let mut cpu = cpu_with(0x76, Some(3));
        cpu.halted = true;
        cpu.interrupt_request_pending = true;
        cpu.interrupt_enable = true;
        assert_eq!(derive_status_kind(&cpu), MachineCycleKind::InterruptAck);
    }

    #[test]
    fn interrupt_pending_without_inte_uses_table() {
        let mut cpu = cpu_with(0x00, Some(0));
        cpu.interrupt_request_pending = true;
        cpu.interrupt_enable = false;
        assert_eq!(derive_status_kind(&cpu), MachineCycleKind::M1Fetch);
    }

    #[test]
    fn first_phase_of_any_opcode_is_m1_fetch() {
        let opcodes = [
            0x00, 0x40, 0x06, 0x01, 0x32, 0x3A, 0xC5, 0xC1, 0xCD, 0xC9, 0xDB, 0xD3,
        ];
        for &op in &opcodes {
            let cpu = cpu_with(op, Some(0));
            assert_eq!(
                derive_status_kind(&cpu),
                MachineCycleKind::M1Fetch,
                "opcode {:#04X} phase 0 must be M1Fetch",
                op,
            );
        }
    }

    #[test]
    fn sta_second_m_cycle_is_memory_read_third_is_memory_write() {
        let cpu = cpu_with(0x32, Some(4));
        assert_eq!(derive_status_kind(&cpu), MachineCycleKind::MemoryRead);
        let cpu = cpu_with(0x32, Some(7));
        assert_eq!(derive_status_kind(&cpu), MachineCycleKind::MemoryRead);
        let cpu = cpu_with(0x32, Some(10));
        assert_eq!(derive_status_kind(&cpu), MachineCycleKind::MemoryWrite);
    }

    #[test]
    fn out_second_m_cycle_is_io_write() {
        let cpu = cpu_with(0xD3, Some(7));
        assert_eq!(derive_status_kind(&cpu), MachineCycleKind::IoWrite);
    }

    #[test]
    fn in_second_m_cycle_is_io_read() {
        let cpu = cpu_with(0xDB, Some(7));
        assert_eq!(derive_status_kind(&cpu), MachineCycleKind::IoRead);
    }

    #[test]
    fn push_writes_to_stack() {
        let cpu = cpu_with(0xC5, Some(5));
        assert_eq!(derive_status_kind(&cpu), MachineCycleKind::StackWrite);
    }

    #[test]
    fn pop_reads_from_stack() {
        let cpu = cpu_with(0xC1, Some(4));
        assert_eq!(derive_status_kind(&cpu), MachineCycleKind::StackRead);
    }
}
