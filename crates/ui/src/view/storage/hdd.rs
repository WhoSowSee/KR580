use iced::widget::{Space, row};
use iced::{Element, Length, alignment};
use k580_app::StorageState;

use super::super::icons;
use super::super::tooltips::shortcut_hint;
use super::{HDD_KEYS, icon_button, storage_window_overlay};
use crate::app::Message;
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
        move |state, show, lang| hdd_header(state, hdd_file_exists, show, lang),
        HDD_KEYS,
    )
}

fn hdd_header<'a>(
    state: &'a StorageState,
    hdd_file_exists: bool,
    show_image_contents: bool,
    lang: Lang,
) -> Element<'a, Message> {
    row![
        Space::new().width(Length::Fill),
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
            shortcut_hint(&Message::CloseHdd),
        ),
    ]
    .align_y(alignment::Vertical::Center)
    .into()
}
