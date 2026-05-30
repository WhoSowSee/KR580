use iced::Element;
use iced::widget::container;

use super::super::theme::{TOKYO_MUTED, ui_text};
use super::setting_row::setting_row;
use super::styles::placeholder_style;
use crate::app::{Message, SettingsDialog};
use crate::i18n::{Key, Lang};

pub(super) fn theme_setting_row(_dialog: &SettingsDialog, lang: Lang) -> Element<'static, Message> {
    let placeholder = container(ui_text(
        lang.t(Key::SettingsThemePlaceholder),
        13,
        TOKYO_MUTED,
    ))
    .padding([8, 14])
    .style(placeholder_style);

    setting_row(
        lang.t(Key::SettingsThemeLabel),
        lang.t(Key::SettingsThemeHint),
        placeholder.into(),
    )
}
