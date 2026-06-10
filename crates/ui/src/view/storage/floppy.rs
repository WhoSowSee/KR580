use iced::widget::{Space, row};
use iced::{Element, Length, alignment};
use k580_app::StorageState;

use super::super::icons;
use super::super::tooltips::shortcut_hint;
use super::{FLOPPY_KEYS, icon_button, storage_window_overlay};
use crate::app::Message;
use crate::i18n::{Key, Lang};

pub(in crate::view) fn floppy_window_overlay<'a>(
    state: &'a StorageState,
    show_image_contents: bool,
    image_contents: &'a [u8],
    image_error: Option<&'a str>,
    lang: Lang,
) -> Element<'a, Message> {
    storage_window_overlay(
        state,
        show_image_contents,
        image_contents,
        image_error,
        lang,
        Message::CloseFloppy,
        |state, show, lang| floppy_header(state, show, lang),
        FLOPPY_KEYS,
    )
}

fn floppy_header<'a>(
    state: &'a StorageState,
    show_image_contents: bool,
    lang: Lang,
) -> Element<'a, Message> {
    row![
        Space::new().width(Length::Fill),
        icon_button(
            icons::hard_drive_download(),
            Some(Message::OpenFloppyImage),
            lang.t(Key::FloppyOpenImage),
            false,
            None,
        ),
        Space::new().width(Length::Fixed(6.0)),
        icon_button(
            icons::hard_drive_upload(),
            Some(Message::SaveFloppyBuffer),
            lang.t(Key::FloppySaveBuffer),
            false,
            None,
        ),
        Space::new().width(Length::Fixed(6.0)),
        icon_button(
            icons::hard_drive_x(),
            Some(Message::DetachFloppyImage),
            lang.t(Key::FloppyDetachImage),
            false,
            None,
        ),
        Space::new().width(Length::Fixed(6.0)),
        icon_button(
            icons::binary(),
            Some(Message::ToggleFloppyImageContents),
            lang.t(Key::FloppyShowImageContents),
            show_image_contents,
            None,
        ),
        Space::new().width(Length::Fixed(6.0)),
        icon_button(
            icons::bug(),
            Some(Message::ToggleFloppyDebugBuffer),
            lang.t(Key::FloppyDebugBuffer),
            state.debug_buffer,
            None,
        ),
        Space::new().width(Length::Fixed(6.0)),
        icon_button(
            icons::brush_cleaning(),
            Some(Message::ClearFloppyBuffer),
            lang.t(Key::FloppyClearBuffer),
            false,
            None,
        ),
        Space::new().width(Length::Fixed(6.0)),
        icon_button(
            icons::window_close(),
            Some(Message::CloseFloppy),
            lang.t(Key::MonitorClose),
            false,
            shortcut_hint(&Message::CloseFloppy),
        ),
    ]
    .align_y(alignment::Vertical::Center)
    .into()
}
