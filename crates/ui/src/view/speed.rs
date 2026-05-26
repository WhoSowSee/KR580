//! Four-tier speed switch for the paced `Run` loop.
//!
//! Lives in the lower-left strip of the schematic next to the
//! Cycle/Tick panel. Pulled out of `schematic.rs` so the parent file
//! stays under the workspace's 400-line ceiling — the switch is its
//! own self-contained widget with no other consumers, and the named
//! tiers carry enough rationale (see the doc comment on `speed_panel`)
//! to deserve their own home.

use iced::widget::{button, container, row};
use iced::{Element, Length, alignment};

use super::styles::schematic_select_button_style;
use super::theme::{TOKYO_TEXT, ui_text};
use super::widgets::legend_panel_left;
use crate::app::{Message, SpeedTier};

const TIER_LABEL_SIZE: u32 = 12;
const TIER_BUTTON_HEIGHT: f32 = 38.0;

/// Four-tier speed switch for the paced `Run` loop. Lives in the
/// lower-left strip next to the Cycle/Tick panel: same vertical band,
/// same "control surface" semantics. Replaces the freeform slider —
/// past the monitor refresh rate the slider only made the program
/// finish faster, never visibly faster, so a continuous control was
/// inviting the user to chase a sweet spot that didn't exist. Named
/// tiers communicate intent honestly:
///
/// - "Медленно" — `SLOW_TIER_HZ` (5 Hz), one instruction every
///   200 ms. The pace at which a human can read every memory row as
///   PC walks across it.
/// - "Средне" — `MEDIUM_TIER_HZ` (20 Hz), the default. Visibly "the
///   program is running" while the eye still keeps up.
/// - "Высоко" — primary monitor's refresh rate (with a 60 Hz
///   fallback). One instruction per frame; finishes as fast as the
///   screen can paint without skipping rows.
/// - "Максимум" — `MAX_TIER_HZ` (1000 Hz), uncoupled from the
///   monitor. The worker churns at its hard ceiling
///   (`MIN_STEP_INTERVAL = 1ms`) and the highlighted row jumps
///   instead of walking. The opt-in for "просто доведи программу до
///   конца, мне не нужно смотреть на каждый шаг".
///
/// The active tier uses the same blue fill as the selected memory row;
/// hover/press use a neutral surface fill with no coloured frame.
pub(super) fn speed_panel(active: SpeedTier) -> Element<'static, Message> {
    let switch = row![
        tier_button("Медленно", SpeedTier::Slow, active),
        tier_button("Средне", SpeedTier::Medium, active),
        tier_button("Высоко", SpeedTier::High, active),
        tier_button("Максимум", SpeedTier::Max, active),
    ]
    .spacing(8);

    container(legend_panel_left(
        "Скорость",
        container(switch)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center),
        Length::Shrink,
    ))
    .width(Length::Fixed(328.0))
    .into()
}

fn tier_button(
    label: &'static str,
    tier: SpeedTier,
    active: SpeedTier,
) -> Element<'static, Message> {
    let is_selected = tier == active;

    let label = container(
        ui_text(label, TIER_LABEL_SIZE, TOKYO_TEXT)
            .align_x(alignment::Horizontal::Center)
            .width(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(alignment::Horizontal::Center)
    .align_y(alignment::Vertical::Center);

    button(label)
        .on_press(Message::SpeedTierChanged(tier))
        .padding(0)
        .width(Length::Fill)
        .height(Length::Fixed(TIER_BUTTON_HEIGHT))
        .style(move |_theme, status| schematic_select_button_style(status, is_selected))
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_label_size_is_larger_than_previous_compact_size() {
        assert!(std::hint::black_box(TIER_LABEL_SIZE) > 11);
    }
}
