use iced::widget::{Space, button, column, container, mouse_area, opaque, row, stack};
use iced::{Background, Border, Color, Element, Length};

use super::theme::{TOKYO_BG, TOKYO_BORDER, TOKYO_MUTED, TOKYO_TEXT, ui_text};
use crate::app::{DiscardModalButton, Message, PendingAction};

/// Renders the "unsaved changes" confirmation modal. The layout is
/// three layers stacked together:
///
/// 1. **Backdrop** — a full-window dark fill (`TOKYO_BOARD` at 70%
///    alpha) wrapped in `mouse_area` + `opaque`. The mouse_area
///    catches *every* click that misses the dialog so the user can
///    dismiss with Esc-style "click outside" without needing the
///    button; the opaque wrapper guarantees those clicks do not
///    pass through to the application underneath. Together they
///    deliver the "ничего не было кликабельным" requirement: any
///    click anywhere on the page either dismisses the modal or
///    activates a button on the modal itself, never anything
///    behind it.
/// 2. **Centred dialog** — a column with the title, the body
///    paragraph, and the two action buttons. Wrapped in a second
///    `opaque` so pointer events on the dialog do not bubble back
///    up to the backdrop's `mouse_area` (otherwise clicking inside
///    the dialog would dismiss it). The focused button reuses the
///    hover fill; keyboard routing lives in `app::modal`.
/// 3. **Spacer rows** above and below + `Space::with(Length::Fill)`
///    flanks on either side push the dialog to the geometric centre
///    of the window without needing absolute coordinates.
///
/// `action` is the queued gesture, used only for the dialog title so
/// the user sees which gesture they are confirming ("Открыть файл" /
/// "Новый файл" / "Импорт" / "Закрыть приложение"). The body
/// paragraph is the same for every variant — the unsaved-changes
/// warning carries the actionable information.
pub(super) fn discard_modal_overlay(
    action: &PendingAction,
    focused: DiscardModalButton,
) -> Element<'_, Message> {
    let (title, title_note) = discard_modal_title(action);

    // Backdrop: wraps the whole window in a darkened fill. The
    // `mouse_area` swallows clicks landing outside the dialog and
    // routes them to `CancelDiscard` — same gesture as clicking
    // "Отменить", so a click on dead space behaves the same way as
    // pressing the cancel button. `opaque` then prevents that
    // mouse_area from passing the event further down the tree.
    let backdrop = mouse_area(
        container(Space::new())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(modal_backdrop_style),
    )
    .on_press(Message::CancelDiscard);

    let cancel_button = button(container(ui_text("Отменить", 13, TOKYO_TEXT)).padding([6, 16]))
        .on_press(Message::CancelDiscard)
        .style(move |theme, status| {
            modal_button_style(theme, status, focused == DiscardModalButton::Cancel)
        });

    let confirm_button =
        button(container(ui_text(discard_confirm_label(action), 13, TOKYO_TEXT)).padding([6, 16]))
            .on_press(Message::ConfirmDiscard)
            .style(move |theme, status| {
                modal_button_style(theme, status, focused == DiscardModalButton::Confirm)
            });

    let buttons = row![
        Space::new().width(Length::Fill),
        cancel_button,
        Space::new().width(Length::Fixed(8.0)),
        confirm_button,
    ]
    .width(Length::Fill);

    let title_note: Element<'_, Message> = match title_note {
        Some(note) => ui_text(note, 12, TOKYO_MUTED).into(),
        None => Space::new().width(Length::Shrink).into(),
    };
    let title_row = row![ui_text(title, 16, TOKYO_TEXT), title_note,]
        .spacing(10)
        .align_y(iced::alignment::Vertical::Center);

    let body = container(
        column![
            title_row,
            Space::new().height(Length::Fixed(8.0)),
            ui_text(
                "Несохранённые изменения будут потеряны.".to_owned(),
                13,
                TOKYO_TEXT,
            ),
            Space::new().height(Length::Fixed(16.0)),
            buttons,
        ]
        .width(Length::Fixed(360.0)),
    )
    .padding(16)
    .style(modal_dialog_style);

    // Centre the dialog. `Length::Fill` spacers above/below and on
    // both sides push the framed body to the middle of the
    // window — works for any window size without picking absolute
    // pixel coordinates.
    let centred = column![
        Space::new().height(Length::Fill),
        row![
            Space::new().width(Length::Fill),
            opaque(body),
            Space::new().width(Length::Fill),
        ]
        .width(Length::Fill),
        Space::new().height(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill);

    // The backdrop sits underneath the centred dialog — both stacked
    // together so the dark fill spans the whole window while the
    // dialog only takes its content size.
    stack![opaque(backdrop), centred]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn discard_modal_title(action: &PendingAction) -> (&'static str, Option<&'static str>) {
    match action {
        PendingAction::OpenSnapshot => ("Открыть файл", None),
        PendingAction::NewFile => ("Новый файл", None),
        PendingAction::Import => ("Импорт", None),
        PendingAction::OpenLegacySnapshot => ("Открыть файл", Some("старый формат")),
        PendingAction::CloseWindow => ("Закрыть приложение", None),
    }
}

fn discard_confirm_label(action: &PendingAction) -> &'static str {
    match action {
        PendingAction::OpenSnapshot | PendingAction::OpenLegacySnapshot => "Открыть",
        PendingAction::NewFile => "Создать",
        PendingAction::Import => "Импортировать",
        PendingAction::CloseWindow => "Закрыть",
    }
}

/// Semi-transparent dark overlay for the modal backdrop. Iced 0.14
/// has no native gaussian blur primitive, so we approximate the
/// "blur the background" intent with a darkening fill — the standard
/// pattern modal dialogs use across desktop UI when blur is not
/// available. 70% alpha on `TOKYO_BOARD` lets just enough of the
/// schematic bleed through that the user remembers what they were
/// doing while still reading the surrounding chrome as suppressed.
fn modal_backdrop_style(_theme: &iced::Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(Background::Color(Color {
            r: 0.07,
            g: 0.07,
            b: 0.13,
            a: 0.70,
        })),
        border: Border {
            radius: 0.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        ..iced::widget::container::Style::default()
    }
}

/// Framed body of the modal dialog. Solid surface (no transparency
/// — the backdrop already provides the contrast) with a 1-px border
/// so the dialog reads as a discrete element floating above the
/// suppressed background. 8 px corner radius matches the rest of
/// the bubble chrome.
fn modal_dialog_style(_theme: &iced::Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        text_color: Some(TOKYO_TEXT),
        background: Some(Background::Color(TOKYO_BG)),
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: TOKYO_BORDER,
        },
        ..iced::widget::container::Style::default()
    }
}

/// The cancel and confirm buttons share the same neutral chrome. The
/// focused twin reuses the hover fill so keyboard users can see what
/// Enter will activate without introducing a second border language.
fn modal_button_style(
    _theme: &iced::Theme,
    status: iced::widget::button::Status,
    focused: bool,
) -> iced::widget::button::Style {
    use crate::view::theme::{TOKYO_SURFACE, TOKYO_SURFACE_2};
    use iced::widget::button;
    let background = match status {
        button::Status::Pressed => TOKYO_SURFACE_2,
        button::Status::Hovered => TOKYO_SURFACE,
        _ if focused => TOKYO_SURFACE,
        _ => TOKYO_BG,
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: TOKYO_TEXT,
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: TOKYO_BORDER,
        },
        ..button::Style::default()
    }
}

#[cfg(test)]
mod tests {
    use super::{discard_confirm_label, discard_modal_title, modal_button_style};
    use crate::app::PendingAction;
    use crate::view::theme::{TOKYO_BORDER, TOKYO_SURFACE};
    use iced::widget::button;
    use iced::{Background, Theme};

    #[test]
    fn focused_modal_button_uses_hover_fill_without_focus_border() {
        let style = modal_button_style(&Theme::TokyoNight, button::Status::Active, true);

        assert_eq!(style.background, Some(Background::Color(TOKYO_SURFACE)));
        assert_eq!(style.border.color, TOKYO_BORDER);
    }

    #[test]
    fn legacy_open_modal_uses_muted_format_note_without_parentheses() {
        assert_eq!(
            discard_modal_title(&PendingAction::OpenLegacySnapshot),
            ("Открыть файл", Some("старый формат"))
        );
    }

    #[test]
    fn discard_confirm_label_matches_pending_action() {
        assert_eq!(
            discard_confirm_label(&PendingAction::OpenLegacySnapshot),
            "Открыть"
        );
        assert_eq!(
            discard_confirm_label(&PendingAction::OpenSnapshot),
            "Открыть"
        );
        assert_eq!(discard_confirm_label(&PendingAction::NewFile), "Создать");
        assert_eq!(
            discard_confirm_label(&PendingAction::CloseWindow),
            "Закрыть"
        );
    }
}
