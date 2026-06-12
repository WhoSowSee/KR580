use iced::widget::{Space, button, column, container, mouse_area, opaque, row, stack};
use iced::{Background, Border, Element, Length};

use super::styles::{modal_backdrop_style, panel_style as modal_dialog_style};
use super::theme::{TOKYO_BOARD, TOKYO_BORDER, TOKYO_MUTED, TOKYO_TEXT, ui_text};
use crate::app::{DiscardModalButton, Message, PendingAction};
use crate::i18n::{Key, Lang};

/// Renders the "unsaved changes" confirmation modal as three layers:
///
/// 1. **Backdrop** – full-window 70%-alpha dark fill in `mouse_area` + `opaque`.
///    The `mouse_area` catches every click that misses the dialog and routes it
///    to `CancelDiscard` (click-outside = cancel); `opaque` blocks those clicks
///    from reaching the app behind. Together they enforce that nothing behind
///    the modal is clickable.
/// 2. **Centred dialog** – title, body paragraph, two action buttons. A second
///    `opaque` keeps clicks inside the dialog from bubbling back to the
///    backdrop's `mouse_area`. Focused button reuses the hover fill; keyboard
///    routing lives in `app::modal`.
/// 3. **Spacer rows** above/below + side spacers centre the dialog without
///    absolute coordinates.
///
/// `action` only affects the title (so the user knows which gesture they are
/// confirming); the body paragraph is shared.
pub(super) fn discard_modal_overlay(
    action: &PendingAction,
    focused: DiscardModalButton,
    lang: Lang,
) -> Element<'_, Message> {
    let (title_key, title_note_key, body_key) = discard_modal_keys(action);
    let title = lang.t(title_key);
    let title_note = title_note_key.map(|k| lang.t(k));

    // Backdrop click → `CancelDiscard`, same as the cancel button.
    // `opaque` keeps the event from passing further down the tree.
    let backdrop = mouse_area(
        container(Space::new())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(modal_backdrop_style),
    )
    .on_press(Message::CancelDiscard);

    let cancel_button =
        button(container(ui_text(lang.t(Key::DiscardCancel), 13, TOKYO_TEXT)).padding([6, 16]))
            .on_press(Message::CancelDiscard)
            .style(move |theme, status| {
                modal_button_style(theme, status, focused == DiscardModalButton::Cancel)
            });

    let confirm_button = button(
        container(ui_text(
            lang.t(discard_confirm_label_key(action)),
            13,
            TOKYO_TEXT,
        ))
        .padding([6, 16]),
    )
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
            ui_text(lang.t(body_key), 13, TOKYO_TEXT,),
            Space::new().height(Length::Fixed(16.0)),
            buttons,
        ]
        .width(Length::Fixed(360.0)),
    )
    .padding(16)
    .style(modal_dialog_style);

    // `Length::Fill` spacers on all four sides centre the dialog
    // without picking absolute pixel coordinates.
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

    // Backdrop spans the whole window; the dialog only takes its
    // content size – both stacked together.
    stack![opaque(backdrop), centred]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn discard_modal_keys(action: &PendingAction) -> (Key, Option<Key>, Key) {
    match action {
        PendingAction::OpenSnapshot => (Key::DiscardTitleOpen, None, Key::DiscardBody),
        PendingAction::NewFile => (Key::DiscardTitleNew, None, Key::DiscardBody),
        PendingAction::Import => (Key::DiscardTitleImport, None, Key::DiscardBody),
        PendingAction::CloseWindow => (Key::DiscardTitleClose, None, Key::DiscardBody),
        PendingAction::DeleteHdd => (Key::DiscardTitleDeleteHdd, None, Key::DiscardBodyDeleteHdd),
    }
}

fn discard_confirm_label_key(action: &PendingAction) -> Key {
    match action {
        PendingAction::OpenSnapshot => Key::DiscardConfirmOpen,
        PendingAction::NewFile => Key::DiscardConfirmNew,
        PendingAction::Import => Key::DiscardConfirmImport,
        PendingAction::CloseWindow => Key::DiscardConfirmClose,
        PendingAction::DeleteHdd => Key::DiscardConfirmDeleteHdd,
    }
}

/// Cancel/confirm share neutral chrome. The focused twin reuses the
/// hover fill so keyboard users see what Enter will activate.
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
        _ => TOKYO_BOARD,
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
    use super::{discard_confirm_label_key, modal_button_style};
    use crate::app::PendingAction;
    use crate::i18n::Key;
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
    fn discard_confirm_label_matches_pending_action() {
        assert_eq!(
            discard_confirm_label_key(&PendingAction::OpenSnapshot),
            Key::DiscardConfirmOpen
        );
        assert_eq!(
            discard_confirm_label_key(&PendingAction::NewFile),
            Key::DiscardConfirmNew
        );
        assert_eq!(
            discard_confirm_label_key(&PendingAction::CloseWindow),
            Key::DiscardConfirmClose
        );
    }
}
