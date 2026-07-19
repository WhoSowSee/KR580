use iced::widget::{container, row, text::Wrapping, tooltip};
use iced::{Element, Length, Padding, alignment};
use std::time::Duration;

use super::styles::inset_style;
use super::theme::{tokyo_muted, tokyo_text, ui_text};
use crate::app::Message;
use crate::persistence::ShortcutSettings;

const LONG_TOOLTIP_WIDTH: f32 = 220.0;
pub(super) const TOOLTIP_BODY_FONT_SIZE: u32 = 12;

pub(super) fn long_tooltip_body(hint: &'static str) -> Element<'static, Message> {
    container(
        ui_text(hint, TOOLTIP_BODY_FONT_SIZE, tokyo_text())
            .width(Length::Fixed(LONG_TOOLTIP_WIDTH))
            .wrapping(Wrapping::Word),
    )
    .padding(Padding::from([4, 8]))
    .style(inset_style)
    .into()
}

pub(super) const VIEWPORT_PADDING: f32 = 12.0;
pub(super) const VISIBLE_GAP: f32 = 6.0;
pub(super) const SNAPPED_TOOLTIP_GAP: f32 = VISIBLE_GAP - VIEWPORT_PADDING;

/// Slightly longer delay for explanatory readout/indicator tooltips so
/// they don't pop up while the user is casually moving the mouse across
/// the schematic plate, while keeping button/shortcut tooltips snappy.
pub(super) const EXPLANATORY_TOOLTIP_DELAY: Duration = Duration::from_millis(1200);

pub(super) fn shortcut_hint(settings: &ShortcutSettings, message: &Message) -> Option<String> {
    match message {
        Message::CloseMonitor
        | Message::CloseFloppy
        | Message::CloseHdd
        | Message::CloseNetwork => Some("Esc".to_owned()),
        _ => crate::app::shortcuts::shortcut_label(settings, message),
    }
}

pub(super) fn hover_tooltip(
    face: Element<'static, Message>,
    hint: &'static str,
    shortcut: Option<String>,
    position: tooltip::Position,
    delay: Duration,
) -> Element<'static, Message> {
    let title =
        ui_text(hint, TOOLTIP_BODY_FONT_SIZE, tokyo_text()).align_x(alignment::Horizontal::Center);
    let content: Element<'static, Message> = match shortcut.filter(|value| !value.is_empty()) {
        Some(shortcut) => row![
            title,
            ui_text(shortcut, 11, tokyo_muted()).align_x(alignment::Horizontal::Center),
        ]
        .spacing(8)
        .align_y(alignment::Vertical::Center)
        .into(),
        None => title.into(),
    };

    let body = container(content)
        .padding(Padding {
            top: 4.0,
            right: 8.0,
            bottom: 4.0,
            left: 8.0,
        })
        .style(inset_style);

    tooltip(face, body, position)
        .gap(SNAPPED_TOOLTIP_GAP)
        .padding(VIEWPORT_PADDING)
        .delay(delay)
        .snap_within_viewport(true)
        .into()
}

#[cfg(test)]
mod tests {
    use super::{
        SNAPPED_TOOLTIP_GAP, TOOLTIP_BODY_FONT_SIZE, VIEWPORT_PADDING, VISIBLE_GAP, shortcut_hint,
    };
    use crate::app::Message;
    use crate::persistence::{ShortcutAction, ShortcutBinding, ShortcutKey, ShortcutSettings};

    #[test]
    fn shortcut_hints_cover_icon_buttons_with_global_shortcuts() {
        let settings = ShortcutSettings::default();
        assert_eq!(
            shortcut_hint(&settings, &Message::ToggleRun),
            Some("Ctrl+R".to_owned())
        );
        assert_eq!(
            shortcut_hint(&settings, &Message::StepInstruction),
            Some("Ctrl+T".to_owned())
        );
        assert_eq!(
            shortcut_hint(&settings, &Message::ResetCpu),
            Some("Ctrl+Shift+G".to_owned())
        );
        assert_eq!(
            shortcut_hint(&settings, &Message::OpenMonitor),
            Some("Ctrl+M".to_owned())
        );
        assert_eq!(
            shortcut_hint(&settings, &Message::OpenFloppy),
            Some("Ctrl+F".to_owned())
        );
        assert_eq!(
            shortcut_hint(&settings, &Message::OpenNetwork),
            Some("Ctrl+A".to_owned())
        );
        assert_eq!(
            shortcut_hint(&settings, &Message::OpenPrinter),
            Some("Ctrl+P".to_owned())
        );
        assert_eq!(
            shortcut_hint(&settings, &Message::ToggleStackView),
            Some("Ctrl+Shift+C".to_owned())
        );
        assert_eq!(shortcut_hint(&settings, &Message::RestartProgram), None);
    }

    #[test]
    fn shortcut_hints_track_custom_shortcut_settings() {
        let mut settings = ShortcutSettings::default();
        settings.assign(
            ShortcutAction::OpenMonitor,
            ShortcutBinding::new(true, true, true, ShortcutKey::M),
        );

        assert_eq!(
            shortcut_hint(&settings, &Message::OpenMonitor),
            Some("Ctrl+Shift+Alt+M".to_owned())
        );
    }

    #[test]
    fn tooltips_keep_distance_from_viewport_edges() {
        assert_eq!(VIEWPORT_PADDING, 12.0);
    }

    #[test]
    fn tooltip_gap_keeps_visible_offset_close_to_trigger() {
        assert_eq!(VISIBLE_GAP, 6.0);
        assert_eq!(SNAPPED_TOOLTIP_GAP, -6.0);
    }

    #[test]
    fn tooltip_body_font_size_stays_readable() {
        assert_eq!(TOOLTIP_BODY_FONT_SIZE, 12);
    }
}
