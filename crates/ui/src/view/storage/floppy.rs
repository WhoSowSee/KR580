use crate::backend::StorageState;
use iced::widget::{Space, row};
use iced::{Element, Length, alignment};

use super::super::icons;
use super::chrome::{icon_button, window_controls};
use super::{FLOPPY_KEYS, storage_window, storage_window_overlay};
use crate::app::{DeviceToolbar, Message};
use crate::i18n::{Key, Lang};

pub(in crate::view) fn floppy_window_overlay<'a>(
    state: &'a StorageState,
    show_image_contents: bool,
    image_contents: &'a [u8],
    image_error: Option<&'a str>,
    lang: Lang,
    toolbar: DeviceToolbar,
) -> Element<'a, Message> {
    storage_window_overlay(
        state,
        show_image_contents,
        image_contents,
        image_error,
        lang,
        Message::CloseFloppy,
        floppy_header(state, show_image_contents, lang, toolbar),
        FLOPPY_KEYS,
    )
}

pub(in crate::view) fn floppy_window<'a>(
    state: &'a StorageState,
    show_image_contents: bool,
    image_contents: &'a [u8],
    image_error: Option<&'a str>,
    lang: Lang,
    toolbar: DeviceToolbar,
) -> Element<'a, Message> {
    storage_window(
        state,
        show_image_contents,
        image_contents,
        image_error,
        lang,
        floppy_header(state, show_image_contents, lang, toolbar),
        FLOPPY_KEYS,
    )
}

fn floppy_header<'a>(
    state: &'a StorageState,
    show_image_contents: bool,
    lang: Lang,
    toolbar: DeviceToolbar,
) -> Element<'a, Message> {
    row![
        window_controls(toolbar, lang),
        icon_button(
            icons::hard_drive_download(),
            Some(Message::OpenFloppyImage),
            lang.t(Key::FloppyOpenImage),
            false,
            None,
            Some(toolbar),
        ),
        Space::new().width(Length::Fixed(6.0)),
        icon_button(
            icons::hard_drive_upload(),
            Some(Message::SaveFloppyBuffer),
            lang.t(Key::FloppySaveBuffer),
            false,
            None,
            Some(toolbar),
        ),
        Space::new().width(Length::Fixed(6.0)),
        icon_button(
            icons::hard_drive_x(),
            Some(Message::DetachFloppyImage),
            lang.t(Key::FloppyDetachImage),
            false,
            None,
            Some(toolbar),
        ),
        Space::new().width(Length::Fixed(6.0)),
        icon_button(
            icons::binary(),
            Some(Message::ToggleFloppyImageContents),
            lang.t(Key::FloppyShowImageContents),
            show_image_contents,
            None,
            Some(toolbar),
        ),
        Space::new().width(Length::Fixed(6.0)),
        icon_button(
            icons::bug(),
            Some(Message::ToggleFloppyDebugBuffer),
            lang.t(Key::FloppyDebugBuffer),
            state.debug_buffer,
            None,
            Some(toolbar),
        ),
        Space::new().width(Length::Fixed(6.0)),
        icon_button(
            icons::brush_cleaning(),
            Some(Message::ClearFloppyBuffer),
            lang.t(Key::FloppyClearBuffer),
            false,
            None,
            Some(toolbar),
        ),
        Space::new().width(Length::Fixed(6.0)),
        icon_button(
            icons::window_close(),
            Some(Message::CloseFloppy),
            lang.t(Key::MonitorClose),
            false,
            Some("Esc".to_owned()),
            Some(toolbar),
        ),
    ]
    .align_y(alignment::Vertical::Center)
    .into()
}
