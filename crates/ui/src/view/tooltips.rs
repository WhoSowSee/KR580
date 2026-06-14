use iced::widget::{container, row, tooltip};
use iced::{Element, Padding, alignment};
use std::time::Duration;

use super::styles::inset_style;
use super::theme::{TOKYO_MUTED, TOKYO_TEXT, ui_text};
use crate::app::Message;

pub(super) const VIEWPORT_PADDING: f32 = 12.0;
pub(super) const VISIBLE_GAP: f32 = 6.0;
pub(super) const SNAPPED_TOOLTIP_GAP: f32 = VISIBLE_GAP - VIEWPORT_PADDING;

pub(super) fn shortcut_hint(message: &Message) -> Option<&'static str> {
    match message {
        Message::NewFile => Some("Ctrl+N"),
        Message::OpenSnapshot => Some("Ctrl+O"),
        Message::SaveSnapshot => Some("Ctrl+S"),
        Message::SaveSnapshotAs => Some("Ctrl+Shift+S"),
        Message::Import => Some("Ctrl+I"),
        Message::Export => Some("Ctrl+E"),
        Message::OpenFloppy => Some("Ctrl+F"),
        Message::ToggleRun => Some("Ctrl+R"),
        Message::StepInstruction => Some("Ctrl+T"),
        Message::StepTact => Some("Ctrl+Y"),
        Message::ResetRam => Some("Ctrl+Shift+R"),
        Message::ResetCpu => Some("Ctrl+Shift+G"),
        Message::ClearHalt => Some("Ctrl+Shift+H"),
        Message::OpenHelp => Some("Ctrl+H"),
        Message::OpenMonitor => Some("Ctrl+M"),
        Message::OpenHdd => Some("Ctrl+D"),
        Message::OpenNetwork => Some("Ctrl+A"),
        Message::OpenPrinter => Some("Ctrl+P"),
        Message::ToggleStackView => Some("Ctrl+Shift+C"),
        Message::OpenSettings => Some("Ctrl+,"),
        Message::Undo => Some("Ctrl+Z"),
        Message::Redo => Some("Ctrl+Shift+Z"),
        Message::OpenOpcodePicker => Some("E"),
        Message::CloseMonitor
        | Message::CloseFloppy
        | Message::CloseHdd
        | Message::CloseNetwork => Some("Esc"),
        _ => None,
    }
}

pub(super) fn hover_tooltip(
    face: Element<'static, Message>,
    hint: &'static str,
    shortcut: Option<&'static str>,
    position: tooltip::Position,
    delay: Duration,
) -> Element<'static, Message> {
    let title = ui_text(hint, 12, TOKYO_TEXT).align_x(alignment::Horizontal::Center);
    let content: Element<'static, Message> = match shortcut.filter(|value| !value.is_empty()) {
        Some(shortcut) => row![
            title,
            ui_text(shortcut, 11, TOKYO_MUTED).align_x(alignment::Horizontal::Center),
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
    use super::{SNAPPED_TOOLTIP_GAP, VIEWPORT_PADDING, VISIBLE_GAP, shortcut_hint};
    use crate::app::Message;

    #[test]
    fn shortcut_hints_cover_icon_buttons_with_global_shortcuts() {
        assert_eq!(shortcut_hint(&Message::ToggleRun), Some("Ctrl+R"));
        assert_eq!(shortcut_hint(&Message::StepInstruction), Some("Ctrl+T"));
        assert_eq!(shortcut_hint(&Message::ResetCpu), Some("Ctrl+Shift+G"));
        assert_eq!(shortcut_hint(&Message::OpenMonitor), Some("Ctrl+M"));
        assert_eq!(shortcut_hint(&Message::OpenFloppy), Some("Ctrl+F"));
        assert_eq!(shortcut_hint(&Message::OpenNetwork), Some("Ctrl+A"));
        assert_eq!(shortcut_hint(&Message::OpenPrinter), Some("Ctrl+P"));
        assert_eq!(
            shortcut_hint(&Message::ToggleStackView),
            Some("Ctrl+Shift+C")
        );
        assert_eq!(shortcut_hint(&Message::RestartProgram), None);
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
}
