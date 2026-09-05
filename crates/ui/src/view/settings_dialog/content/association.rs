use iced::widget::{Space, row};
use iced::{Element, Length, alignment};

use super::super::setting_row::setting_row;
use super::rows::settings_browse_button;
use crate::app::{ContentFocus, Message, SettingsDialog};
use crate::i18n::{Key, Lang};

pub(super) fn file_association_row<'a>(
    dialog: &'a SettingsDialog,
    lang: Lang,
) -> Element<'a, Message> {
    let focused = dialog.content_focus_is_visible(ContentFocus::FileAssociation);

    #[cfg(target_os = "windows")]
    let (label, message) = if k580_ui::file_assoc::is_registered() {
        (
            Key::SettingsFileAssociationRemove,
            Message::SettingsFileAssociationUnregister,
        )
    } else {
        (
            Key::SettingsFileAssociationAdd,
            Message::SettingsFileAssociationRegister,
        )
    };
    #[cfg(not(target_os = "windows"))]
    let (label, message) = (
        Key::SettingsFileAssociationAdd,
        Message::SettingsFileAssociationRegister,
    );

    let button = settings_browse_button(lang.t(label), message, focused);
    let control =
        row![Space::new().width(Length::Fill), button].align_y(alignment::Vertical::Center);

    setting_row(
        lang.t(Key::SettingsFileAssociationLabel),
        lang.t(Key::SettingsFileAssociationHint),
        control.into(),
    )
}
