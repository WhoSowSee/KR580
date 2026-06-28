use crate::backend::StorageState;
use iced::widget::{Space, row};
use iced::{Element, Length, alignment};

use super::super::icons;
use super::chrome::{icon_button, window_controls};
use super::{HDD_KEYS, storage_window, storage_window_overlay};
use crate::app::{Message, ToolWindowKind};
use crate::i18n::{Key, Lang};

pub(in crate::view) fn hdd_window_overlay<'a>(
    state: &'a StorageState,
    hdd_file_exists: bool,
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
        Message::CloseHdd,
        move |state, show, detached, always_on_top, lang| {
            hdd_header(state, hdd_file_exists, show, detached, always_on_top, lang)
        },
        HDD_KEYS,
    )
}

pub(in crate::view) fn hdd_window<'a>(
    state: &'a StorageState,
    hdd_file_exists: bool,
    show_image_contents: bool,
    image_contents: &'a [u8],
    image_error: Option<&'a str>,
    always_on_top: bool,
    lang: Lang,
) -> Element<'a, Message> {
    storage_window(
        state,
        show_image_contents,
        image_contents,
        image_error,
        always_on_top,
        lang,
        move |state, show, detached, always_on_top, lang| {
            hdd_header(state, hdd_file_exists, show, detached, always_on_top, lang)
        },
        HDD_KEYS,
    )
}

fn hdd_header<'a>(
    state: &'a StorageState,
    hdd_file_exists: bool,
    show_image_contents: bool,
    detached: bool,
    always_on_top: bool,
    lang: Lang,
) -> Element<'a, Message> {
    row![
        window_controls(ToolWindowKind::Hdd, detached, always_on_top, lang),
        icon_button(
            icons::folder_open(),
            Some(Message::ChooseHddDirectory),
            lang.t(Key::HddChooseDirectory),
            false,
            None,
        ),
        Space::new().width(Length::Fixed(6.0)),
        icon_button(
            icons::binary(),
            Some(Message::ToggleHddImageContents),
            lang.t(Key::HddShowImageContents),
            show_image_contents,
            None,
        ),
        Space::new().width(Length::Fixed(6.0)),
        icon_button(
            icons::bug(),
            Some(Message::ToggleHddDebugBuffer),
            lang.t(Key::HddDebugBuffer),
            state.debug_buffer,
            None,
        ),
        Space::new().width(Length::Fixed(6.0)),
        icon_button(
            icons::brush_cleaning(),
            Some(Message::ClearHddBuffer),
            lang.t(Key::HddClearBuffer),
            false,
            None,
        ),
        Space::new().width(Length::Fixed(6.0)),
        icon_button(
            icons::trash_2(),
            if hdd_file_exists {
                Some(Message::DeleteHddFile)
            } else {
                None
            },
            lang.t(Key::HddDeleteFile),
            false,
            None,
        ),
        Space::new().width(Length::Fixed(6.0)),
        icon_button(
            icons::file_plus_corner(),
            if hdd_file_exists {
                None
            } else {
                Some(Message::CreateHddFile)
            },
            lang.t(Key::HddCreateFile),
            false,
            None,
        ),
        Space::new().width(Length::Fixed(6.0)),
        icon_button(
            icons::window_close(),
            Some(Message::CloseHdd),
            lang.t(Key::HddClose),
            false,
            Some("Esc".to_owned()),
        ),
    ]
    .align_y(alignment::Vertical::Center)
    .into()
}
