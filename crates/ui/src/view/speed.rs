//! Four-tier speed stepper for the paced `Run` loop.
//!
//! The control keeps the same `SpeedTier` model as the previous
//! segmented text buttons, but presents it as a compact instrument:
//! left/right chevrons move to the neighbouring tier, the centre gauge
//! shows how much of the 4-tier range is active, and the current
//! resolved speed is printed below the gauge.

use iced::widget::{Row, Space, button, column, container, row, svg};
use iced::{Background, Border, Color, Element, Length, alignment};

use super::icons;
use super::styles::solid_style;
use super::theme::{
    TOKYO_BOARD, TOKYO_BORDER, TOKYO_MAGENTA, TOKYO_SURFACE, TOKYO_SURFACE_2, TOKYO_TEXT, ui_text,
};
use super::widgets::legend_panel_left;
use crate::app::{Message, SpeedTier, tier_hz};
use crate::i18n::{Key, Lang};

const CONTROL_BUTTON_WIDTH: f32 = 36.0;
const CONTROL_BUTTON_HEIGHT: f32 = 36.0;
const CONTROL_ICON_SIZE: f32 = 18.0;
const CONTROL_ROW_GAP: f32 = 8.0;
const SPEED_PANEL_WIDTH: f32 = 328.0;
#[cfg(test)]
const SPEED_PANEL_HORIZONTAL_PADDING: f32 = 20.0;
const GAUGE_SEGMENTS: usize = 20;
const GAUGE_SEGMENT_WIDTH: f32 = 4.0;
const GAUGE_SEGMENT_HEIGHT: f32 = 18.0;
const GAUGE_SEGMENT_GAP: f32 = 5.0;
const GAUGE_READOUT_SPACING: f32 = 4.0;
const GAUGE_WAVE_EDGE_HEIGHT: f32 = 0.55;
const GAUGE_WAVE_CENTER_HEIGHT: f32 = 0.78;
const GAUGE_HALO_SEGMENTS: usize = 4;

/// Four-tier speed switch for the paced `Run` loop. Lives in the
/// lower-left strip next to the quick-access panel. The title remains
/// in the framed legend; the body uses chevron buttons and a segmented
/// gauge so the control reads like the reference speed instrument
/// while still dispatching the same `Message::SpeedTierChanged`.
pub(super) fn speed_panel(active: SpeedTier, lang: Lang) -> Element<'static, Message> {
    let gauge = column![
        gauge_row(active),
        ui_text(speed_readout(active, lang), 13, TOKYO_TEXT).align_x(alignment::Horizontal::Center),
    ]
    .spacing(GAUGE_READOUT_SPACING)
    .align_x(alignment::Horizontal::Center);

    let control = row![
        speed_step_button(icons::chevrons_left(), previous_tier(active)),
        Space::new().width(Length::Fill),
        gauge,
        Space::new().width(Length::Fill),
        speed_step_button(icons::chevrons_right(), next_tier(active)),
    ]
    .spacing(CONTROL_ROW_GAP)
    .align_y(alignment::Vertical::Center)
    .width(Length::Fill);

    container(legend_panel_left(
        lang.t(Key::SpeedTitle),
        container(control)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center),
        Length::Shrink,
    ))
    .width(Length::Fixed(SPEED_PANEL_WIDTH))
    .into()
}

fn speed_step_button(handle: svg::Handle, tier: SpeedTier) -> Element<'static, Message> {
    let glyph = svg(handle)
        .width(Length::Fixed(CONTROL_ICON_SIZE))
        .height(Length::Fixed(CONTROL_ICON_SIZE))
        .style(|_theme, _status| svg::Style {
            color: Some(TOKYO_TEXT),
        });

    button(
        container(glyph)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center),
    )
    .on_press(Message::SpeedTierChanged(tier))
    .padding(0)
    .width(Length::Fixed(CONTROL_BUTTON_WIDTH))
    .height(Length::Fixed(CONTROL_BUTTON_HEIGHT))
    .style(|_theme, status| speed_step_button_style(status))
    .into()
}

fn speed_step_button_style(status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Pressed => TOKYO_SURFACE_2,
        button::Status::Hovered => TOKYO_SURFACE,
        _ => TOKYO_BOARD,
    };
    let border_color = match status {
        button::Status::Disabled => Color {
            a: 0.35,
            ..TOKYO_BORDER
        },
        _ => TOKYO_BORDER,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: TOKYO_TEXT,
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: border_color,
        },
        ..button::Style::default()
    }
}

fn gauge_row(active: SpeedTier) -> Element<'static, Message> {
    Row::with_children((0..GAUGE_SEGMENTS).map(move |idx| gauge_segment(idx, active)))
        .spacing(GAUGE_SEGMENT_GAP)
        .align_y(alignment::Vertical::Center)
        .into()
}

fn gauge_segment(index: usize, active: SpeedTier) -> Element<'static, Message> {
    let color = gauge_segment_color(index, active);
    let height = gauge_segment_height(index);

    container(Space::new())
        .width(Length::Fixed(GAUGE_SEGMENT_WIDTH))
        .height(Length::Fixed(height))
        .style(move |_theme| solid_style(color, 3.0))
        .into()
}

fn gauge_segment_color(index: usize, active: SpeedTier) -> Color {
    let (start, end) = active_segment_range(active);

    if (start..end).contains(&index) {
        let strength = active_segment_strength(index, start, end);
        Color {
            a: 0.66 + 0.24 * strength,
            ..TOKYO_MAGENTA
        }
    } else {
        let halo = inactive_segment_halo(index, start, end);
        if halo > 0.0 {
            return blend_color(
                Color {
                    a: 0.50,
                    ..TOKYO_SURFACE_2
                },
                Color {
                    a: 0.50,
                    ..TOKYO_MAGENTA
                },
                halo,
            );
        }

        Color {
            a: 0.48,
            ..TOKYO_SURFACE_2
        }
    }
}

fn gauge_segment_height(index: usize) -> f32 {
    let strength = whole_gauge_strength(index);
    let factor =
        GAUGE_WAVE_EDGE_HEIGHT + (GAUGE_WAVE_CENTER_HEIGHT - GAUGE_WAVE_EDGE_HEIGHT) * strength;

    GAUGE_SEGMENT_HEIGHT * factor
}

fn active_segment_range(tier: SpeedTier) -> (usize, usize) {
    let active_count = active_segment_count(tier);
    let start = (GAUGE_SEGMENTS - active_count) / 2;

    (start, start + active_count)
}

fn active_segment_strength(index: usize, start: usize, end: usize) -> f32 {
    let center = (start + end - 1) as f32 / 2.0;
    let radius = ((end - start - 1) as f32 / 2.0).max(1.0);
    let distance = (index as f32 - center).abs();

    (1.0 - distance / radius).clamp(0.0, 1.0)
}

fn whole_gauge_strength(index: usize) -> f32 {
    let center = (GAUGE_SEGMENTS - 1) as f32 / 2.0;
    let radius = center.max(1.0);
    let distance = (index as f32 - center).abs();

    (1.0 - distance / radius).clamp(0.0, 1.0)
}

fn inactive_segment_halo(index: usize, start: usize, end: usize) -> f32 {
    let distance = if index < start {
        start - index
    } else if index >= end {
        index - end + 1
    } else {
        return 0.0;
    };

    if distance > GAUGE_HALO_SEGMENTS {
        return 0.0;
    }

    0.04 + 0.14 * (GAUGE_HALO_SEGMENTS + 1 - distance) as f32 / GAUGE_HALO_SEGMENTS as f32
}

fn blend_color(base: Color, accent: Color, amount: f32) -> Color {
    let amount = amount.clamp(0.0, 1.0);

    Color {
        r: base.r + (accent.r - base.r) * amount,
        g: base.g + (accent.g - base.g) * amount,
        b: base.b + (accent.b - base.b) * amount,
        a: base.a + (accent.a - base.a) * amount,
    }
}

fn speed_readout(tier: SpeedTier, lang: Lang) -> String {
    format!("{} {}", tier_hz(tier), lang.t(Key::SpeedUnit))
}

fn previous_tier(tier: SpeedTier) -> SpeedTier {
    match tier {
        SpeedTier::Slow => SpeedTier::Slow,
        SpeedTier::Medium => SpeedTier::Slow,
        SpeedTier::High => SpeedTier::Medium,
        SpeedTier::Max => SpeedTier::High,
    }
}

fn next_tier(tier: SpeedTier) -> SpeedTier {
    match tier {
        SpeedTier::Slow => SpeedTier::Medium,
        SpeedTier::Medium => SpeedTier::High,
        SpeedTier::High => SpeedTier::Max,
        SpeedTier::Max => SpeedTier::Max,
    }
}

fn active_segment_count(tier: SpeedTier) -> usize {
    match tier {
        SpeedTier::Slow => 5,
        SpeedTier::Medium => 10,
        SpeedTier::High => 15,
        SpeedTier::Max => 20,
    }
}

#[cfg(test)]
mod tests {
    use super::super::theme::TOKYO_BORDER;
    use super::*;

    #[test]
    fn speed_buttons_step_between_adjacent_tiers() {
        assert_eq!(previous_tier(SpeedTier::Medium), SpeedTier::Slow);
        assert_eq!(next_tier(SpeedTier::Medium), SpeedTier::High);
    }

    #[test]
    fn speed_buttons_clamp_at_edges() {
        assert_eq!(previous_tier(SpeedTier::Slow), SpeedTier::Slow);
        assert_eq!(next_tier(SpeedTier::Max), SpeedTier::Max);
    }

    #[test]
    fn speed_gauge_has_one_quarter_per_tier() {
        assert_eq!(active_segment_count(SpeedTier::Slow), 5);
        assert_eq!(active_segment_count(SpeedTier::Medium), 10);
        assert_eq!(active_segment_count(SpeedTier::High), 15);
        assert_eq!(active_segment_count(SpeedTier::Max), 20);
    }

    #[test]
    fn speed_control_fixed_width_fits_panel_inner_width() {
        let gauge_width = (GAUGE_SEGMENTS as f32 * GAUGE_SEGMENT_WIDTH) + 19.0 * GAUGE_SEGMENT_GAP;
        let fixed_width = 2.0 * CONTROL_BUTTON_WIDTH + gauge_width + 4.0 * CONTROL_ROW_GAP;
        let inner_width = SPEED_PANEL_WIDTH - SPEED_PANEL_HORIZONTAL_PADDING;

        assert!(std::hint::black_box(fixed_width <= inner_width));
    }

    #[test]
    fn speed_step_buttons_are_square() {
        assert_eq!(
            std::hint::black_box(CONTROL_BUTTON_WIDTH),
            CONTROL_BUTTON_HEIGHT
        );
    }

    #[test]
    fn speed_step_button_resting_background_matches_app_plate() {
        let style = speed_step_button_style(button::Status::Active);

        assert_eq!(style.background, Some(Background::Color(TOKYO_BOARD)));
    }

    #[test]
    fn speed_step_button_hover_uses_shared_hover_surface() {
        let style = speed_step_button_style(button::Status::Hovered);

        assert_eq!(style.background, Some(Background::Color(TOKYO_SURFACE)));
    }

    #[test]
    fn speed_step_button_border_matches_panel_border() {
        let active = speed_step_button_style(button::Status::Active);
        let hovered = speed_step_button_style(button::Status::Hovered);

        assert_eq!(active.border.color, TOKYO_BORDER);
        assert_eq!(hovered.border.color, TOKYO_BORDER);
    }

    #[test]
    fn gauge_wave_tapers_toward_edges() {
        let center = gauge_segment_height(9);
        let edge = gauge_segment_height(5);

        assert!(std::hint::black_box(center > edge));
        assert!(std::hint::black_box(edge > GAUGE_SEGMENT_HEIGHT * 0.6));
    }

    #[test]
    fn gauge_wave_keeps_outer_segments_shorter() {
        let near_active = gauge_segment_height(4);
        let outer_edge = gauge_segment_height(0);

        assert!(std::hint::black_box(near_active > outer_edge));
        assert!(std::hint::black_box(near_active < GAUGE_SEGMENT_HEIGHT));
        assert!(std::hint::black_box(outer_edge < GAUGE_SEGMENT_HEIGHT));
    }

    #[test]
    fn lighting_gauge_segment_changes_color_without_changing_height() {
        let segment = 5;
        let slow_range = active_segment_range(SpeedTier::Slow);
        let medium_range = active_segment_range(SpeedTier::Medium);
        let inactive_height = gauge_segment_height(segment);
        let inactive_color = gauge_segment_color(segment, SpeedTier::Slow);
        let active_height = gauge_segment_height(segment);
        let active_color = gauge_segment_color(segment, SpeedTier::Medium);

        assert!(!std::hint::black_box(slow_range.0..slow_range.1).contains(&segment));
        assert!(std::hint::black_box(medium_range.0..medium_range.1).contains(&segment));
        assert_eq!(std::hint::black_box(active_height), inactive_height);
        assert_ne!(std::hint::black_box(active_color), inactive_color);
    }

    #[test]
    fn max_gauge_wave_still_tapers_toward_panel_edges() {
        let center = gauge_segment_height(9);
        let outer_edge = gauge_segment_height(0);

        assert!(std::hint::black_box(center > outer_edge));
    }

    #[test]
    fn inactive_gauge_halo_is_pink_near_active_wave_and_fades_outward() {
        let near = gauge_segment_color(4, SpeedTier::Medium);
        let next = gauge_segment_color(3, SpeedTier::Medium);
        let far = gauge_segment_color(0, SpeedTier::Medium);

        assert!(std::hint::black_box(near.r > next.r));
        assert!(std::hint::black_box(next.r > far.r));
        assert!(std::hint::black_box(near.b > next.b));
        assert!(std::hint::black_box(next.b > far.b));
    }

    #[test]
    fn speed_readout_uses_instruction_rate_units() {
        assert_eq!(
            speed_readout(SpeedTier::Slow, crate::i18n::Lang::Ru),
            "5 инстр/сек"
        );
        assert_eq!(
            speed_readout(SpeedTier::Slow, crate::i18n::Lang::En),
            "5 instr/sec"
        );
    }
}
