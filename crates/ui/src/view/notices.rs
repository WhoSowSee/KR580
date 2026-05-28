use iced::widget::{Space, column, container, mouse_area, opaque, row};
use iced::{Element, Length, alignment};

use super::styles::{error_inset_style, info_inset_style};
use super::theme::{TOKYO_TEXT, ui_text};
use crate::app::Message;

/// Vertical offset of the halt-blocked notice overlay from the top of
/// the app root. Sits comfortably below the menu bar (34 px tall + 1 px
/// hairline) with a small gap so the framed message reads as a separate
/// floating element rather than glued to the bar.
const HALT_NOTICE_TOP: f32 = 48.0;

/// Vertical offset of the file-error notice overlay. Sits a touch
/// further below the menu bar than the halt notice so that, in the
/// (rare) case both are visible together — a halted CPU plus a
/// failed save dialog — the two messages stack rather than overlap.
/// The halt notice is the longer-lived of the two (it persists until
/// the CPU is reset), so the error notice rides on top of it; that
/// way the new, actionable error reads first.
const ERROR_NOTICE_TOP: f32 = 88.0;

/// Vertical offset of the info-notice overlay (legacy-format heads
/// up). Sits below the error notice so the three stack predictably
/// when more than one is visible at once: halt (persistent, top),
/// error (8 s, middle), info (5 s, bottom).
const INFO_NOTICE_TOP: f32 = 128.0;

/// Floating notice anchored to the top centre of the window. Used for
/// the halt-blocked Variant A message — see `docs/ui_app.md` and the
/// `halt_notice` field on `DesktopApp`.
pub(super) fn halt_notice_overlay(notice: &str) -> Element<'_, Message> {
    notice_overlay(
        notice,
        HALT_NOTICE_TOP,
        error_inset_style,
        Message::DismissHaltNotice,
    )
}

/// Floating notice for file-system errors (failed open / save /
/// import / export). Routed through `error_notice` rather than the
/// status bar because the status bar is too quiet for this failure.
pub(super) fn error_notice_overlay(notice: &str) -> Element<'_, Message> {
    notice_overlay(
        notice,
        ERROR_NOTICE_TOP,
        error_inset_style,
        Message::DismissErrorNotice,
    )
}

/// Floating notice for the legacy-format heads-up ("Открыт старый
/// формат файла"). Mirrors the error notice except for the yellow
/// frame, which signals "heads up, not an error".
pub(super) fn info_notice_overlay(notice: &str) -> Element<'_, Message> {
    notice_overlay(
        notice,
        INFO_NOTICE_TOP,
        info_inset_style,
        Message::DismissInfoNotice,
    )
}

fn notice_overlay(
    notice: &str,
    top: f32,
    style: fn(&iced::Theme) -> iced::widget::container::Style,
    dismiss: Message,
) -> Element<'_, Message> {
    let body = container(
        ui_text(notice.to_owned(), 15, TOKYO_TEXT).align_x(alignment::Horizontal::Center),
    )
    .padding([12, 22])
    .style(style);
    let dismissible = mouse_area(opaque(body)).on_press(dismiss);
    column![
        Space::new().height(Length::Fixed(top)),
        row![
            Space::new().width(Length::Fill),
            dismissible,
            Space::new().width(Length::Fill),
        ]
        .width(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
