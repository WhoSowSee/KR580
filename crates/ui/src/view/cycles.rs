//! Two blocks at the bottom of the schematic: the school-convention
//! M-cycle/T-phase view on the left and the datasheet linear-T-states
//! view on the right. They diverge on HLT (4 vs 7) and on multi-M-cycle
//! instructions where Block 1 resets the tact counter each cycle while
//! Block 2's phase grows linearly.

use iced::widget::{Space, column, container, row, tooltip};
use iced::{Element, Length, Padding, alignment};
use k580_core::{
    Cpu8080State, MachineCycleLayout, MachineCyclePosition, decode_opcode, layout_for, position_for,
};

use super::styles::inset_style;
use super::theme::{TOKYO_GREEN, TOKYO_MUTED, TOKYO_TEXT, mono_text, ui_text};
use super::widgets::legend_panel_left;
use crate::app::Message;
use crate::i18n::{Key, Lang};

const CYCLE_BLOCK_WIDTH: f32 = 200.0;
const TIMING_BLOCK_WIDTH: f32 = 200.0;
const CYCLE_BLOCK_BALANCE_SPACER_HEIGHT: f32 = 6.0;

/// HLT layout (M-cycle view) is `[4]` — only the visible M1; the
/// datasheet view glues M1+M2 into `[7]` so the linear T-phase counts
/// halt-ack too.
fn full_duration_layout(opcode: u8) -> MachineCycleLayout {
    if opcode == 0x76 {
        return MachineCycleLayout::fixed(&[7]);
    }
    layout_for(opcode)
}

/// Phase is clamped to `total - 1` so HLT's `last_completed_tact_phase = 6`
/// against an M-cycle layout of `[4]` freezes on T4 instead of dropping
/// to "-".
fn position_at(
    cpu: &Cpu8080State,
    phase_source: Option<u8>,
    use_full_duration: bool,
) -> Option<MachineCyclePosition> {
    let opcode = cpu.last_fetched_opcode;
    decode_opcode(opcode).ok()?;
    let layout = if use_full_duration {
        full_duration_layout(opcode)
    } else {
        layout_for(opcode)
    };
    let phase = phase_source?;
    let taken_total = layout.total_t_states(true);
    let not_taken_total = layout.total_t_states(false);
    let clamped_taken = phase.min(taken_total.saturating_sub(1));
    let clamped_not_taken = phase.min(not_taken_total.saturating_sub(1));
    position_for(layout, true, clamped_taken)
        .or_else(|| position_for(layout, false, clamped_not_taken))
}

fn labeled_row_with_tooltip(
    label_short: &str,
    value_text: String,
    hint: &str,
) -> Element<'static, Message> {
    use std::time::Duration;

    let face = row![
        ui_text(label_short.to_owned(), 12, TOKYO_MUTED),
        Space::new().width(Length::Fill),
        mono_text(value_text, 14, TOKYO_GREEN),
    ]
    .spacing(10)
    .align_y(alignment::Vertical::Center);

    let face_container = container(face).width(Length::Fill);

    let body = container(ui_text(hint.to_owned(), 12, TOKYO_TEXT))
        .padding(Padding {
            top: 4.0,
            right: 8.0,
            bottom: 4.0,
            left: 8.0,
        })
        .max_width(230.0)
        .style(inset_style);

    tooltip(face_container, body, tooltip::Position::Top)
        .gap(4.0)
        .padding(0.0)
        .delay(Duration::from_millis(600))
        .snap_within_viewport(true)
        .into()
}

fn total_tacts_text(cpu: &Cpu8080State) -> String {
    if cpu.cycle_count == 0 {
        "-".to_owned()
    } else {
        cpu.cycle_count.to_string()
    }
}

fn cycle_timing_spacer_width() -> Length {
    Length::Fill
}

pub(super) fn cycle_panels(cpu: &Cpu8080State, lang: Lang) -> Element<'_, Message> {
    let active_phase = cpu.tact_phase.or(cpu.last_completed_tact_phase);

    let cycle_active = position_at(cpu, active_phase, false);
    let tact_last_completed = position_at(cpu, cpu.last_completed_tact_phase, false);
    let full_duration_active = position_at(cpu, active_phase, true);

    let cycle_text = match cycle_active {
        Some(pos) => pos.m_cycle.to_string(),
        None => "-".to_owned(),
    };
    let tact_text = match tact_last_completed {
        Some(pos) => pos.t_in_cycle.to_string(),
        None => "-".to_owned(),
    };
    let tact_full_text = match full_duration_active {
        Some(pos) => pos.t_in_cycle.to_string(),
        None => "-".to_owned(),
    };

    let cycle_block = container(legend_panel_left(
        lang.t(Key::CyclesAndTacts),
        column![
            Space::new().height(Length::Fixed(CYCLE_BLOCK_BALANCE_SPACER_HEIGHT)),
            labeled_row_with_tooltip(
                lang.t(Key::CycleLabel),
                cycle_text,
                lang.t(Key::CycleTooltip),
            ),
            labeled_row_with_tooltip(lang.t(Key::TactLabel), tact_text, lang.t(Key::TactTooltip),),
            Space::new().height(Length::Fixed(CYCLE_BLOCK_BALANCE_SPACER_HEIGHT)),
        ]
        .spacing(6),
        Length::Shrink,
    ))
    .width(Length::Fixed(CYCLE_BLOCK_WIDTH));

    // `*` = "instruction finished, last recorded value shown".
    let linear_phase_text = match (cpu.tact_phase, cpu.last_completed_tact_phase) {
        (Some(phase), _) => phase.to_string(),
        (None, Some(last)) => format!("{last}*"),
        (None, None) => "-".to_owned(),
    };

    let our_block = container(legend_panel_left(
        lang.t(Key::InternalTimings),
        column![
            labeled_row_with_tooltip(
                lang.t(Key::TotalTacts),
                total_tacts_text(cpu),
                lang.t(Key::TotalTactsTooltip),
            ),
            labeled_row_with_tooltip(
                lang.t(Key::InstructionTact),
                tact_full_text,
                lang.t(Key::InstructionTactTooltip),
            ),
            labeled_row_with_tooltip(
                lang.t(Key::PhaseLabel),
                linear_phase_text,
                lang.t(Key::PhaseTooltip),
            ),
        ]
        .spacing(6),
        Length::Shrink,
    ))
    .width(Length::Fixed(TIMING_BLOCK_WIDTH));

    row![
        cycle_block,
        Space::new().width(cycle_timing_spacer_width()),
        our_block
    ]
    .width(Length::Fill)
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_tacts_text_is_dash_at_cold_start() {
        let cpu = Cpu8080State::default();
        assert_eq!(total_tacts_text(&cpu), "-");
    }

    #[test]
    fn cycle_and_timing_blocks_share_width() {
        assert_eq!(CYCLE_BLOCK_WIDTH, TIMING_BLOCK_WIDTH);
    }

    #[test]
    fn cycle_and_timing_blocks_are_pushed_to_opposite_edges() {
        assert!(matches!(cycle_timing_spacer_width(), Length::Fill));
    }

    #[test]
    fn cycle_block_has_height_balance_spacer() {
        assert!(std::hint::black_box(CYCLE_BLOCK_BALANCE_SPACER_HEIGHT) > 0.0);
    }
}
