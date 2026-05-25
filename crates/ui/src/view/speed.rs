//! Four-tier speed switch for the paced `Run` loop.
//!
//! Lives in the lower-left strip of the schematic next to the
//! Cycle/Tick panel. Pulled out of `schematic.rs` so the parent file
//! stays under the workspace's 400-line ceiling — the switch is its
//! own self-contained widget with no other consumers, and the named
//! tiers carry enough rationale (see the doc comment on `speed_panel`)
//! to deserve their own home.

use iced::widget::{button, column, container, row};
use iced::{Element, Length, alignment};

use super::styles::{mux_button_style, schematic_block_style};
use super::theme::{TOKYO_BLUE, TOKYO_MAGENTA, TOKYO_MUTED, TOKYO_TEXT, ui_text};
use crate::app::{Message, SpeedTier, tier_hz};

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
/// The active tier is highlighted with the same `mux_button_style`
/// the multiplexer panel uses for its own selected/unselected
/// distinction, so the switch reads as part of the schematic's
/// control surface rather than a foreign widget.
pub(super) fn speed_panel(active: SpeedTier) -> Element<'static, Message> {
    let caption = format!("Скорость: {} шаг/сек", tier_hz(active));

    let switch = row![
        tier_button("Медленно", SpeedTier::Slow, active),
        tier_button("Средне", SpeedTier::Medium, active),
        tier_button("Высоко", SpeedTier::High, active),
        tier_button("Максимум", SpeedTier::Max, active),
    ]
    .spacing(4);

    container(column![ui_text(caption, 12, TOKYO_MUTED), switch].spacing(6))
        .padding(10)
        .width(Length::Fixed(340.0))
        .style(schematic_block_style)
        .into()
}

fn tier_button(
    label: &'static str,
    tier: SpeedTier,
    active: SpeedTier,
) -> Element<'static, Message> {
    let is_selected = tier == active;
    let accent = if is_selected {
        TOKYO_MAGENTA
    } else {
        TOKYO_BLUE
    };

    button(
        ui_text(label, 11, TOKYO_TEXT)
            .align_x(alignment::Horizontal::Center)
            .width(Length::Fill),
    )
    .on_press(Message::SpeedTierChanged(tier))
    .padding(6)
    .width(Length::Fill)
    .style(move |_theme, status| mux_button_style(status, accent, is_selected))
    .into()
}
