use iced::widget::{Space, button, container, row};
use iced::{Element, Length, alignment};

use super::super::theme::{tokyo_text, ui_text};
use super::consts::FOOTER_HEIGHT;
use super::styles::footer_button_style;
use crate::app::{FooterFocus, Message, SettingsCategory, SettingsDialog, SettingsSection};
use crate::i18n::{Key, Lang};

pub(super) fn settings_footer(dialog: &SettingsDialog, lang: Lang) -> Element<'static, Message> {
    let focus = dialog.footer_focus;
    let footer_active = dialog.section_focus_is_visible(SettingsSection::Footer);

    let reset = footer_button(
        lang.t(Key::SettingsReset),
        Message::SettingsResetRequested,
        footer_active && focus == FooterFocus::Reset,
    );
    let reset_shortcuts = footer_button(
        reset_shortcuts_label(lang),
        Message::SettingsShortcutsReset,
        footer_active && focus == FooterFocus::ShortcutReset,
    );
    let cancel = footer_button(
        lang.t(Key::DiscardCancel),
        Message::CloseSettings,
        footer_active && focus == FooterFocus::Cancel,
    );
    let save = footer_button(
        lang.t(Key::FileSave),
        Message::SaveSettings,
        footer_active && focus == FooterFocus::Save,
    );

    let mut buttons = row![reset].align_y(alignment::Vertical::Center);
    if dialog.category == SettingsCategory::Shortcuts {
        buttons = buttons
            .push(Space::new().width(Length::Fixed(8.0)))
            .push(reset_shortcuts);
    }
    buttons = buttons
        .push(Space::new().width(Length::Fill))
        .push(cancel)
        .push(Space::new().width(Length::Fixed(8.0)))
        .push(save);

    container(buttons)
        .padding(iced::Padding {
            top: 0.0,
            right: 16.0,
            bottom: 0.0,
            left: 16.0,
        })
        .height(Length::Fixed(FOOTER_HEIGHT))
        .align_y(alignment::Vertical::Center)
        .into()
}

fn reset_shortcuts_label(lang: Lang) -> &'static str {
    match lang {
        Lang::Ru => "Сбросить сочетания",
        Lang::En => "Reset shortcuts",
    }
}

fn footer_button(label: &'static str, action: Message, focused: bool) -> Element<'static, Message> {
    button(container(ui_text(label, 13, tokyo_text())).padding([6, 16]))
        .on_press(action)
        .padding(0)
        .style(move |_theme, status| footer_button_style(status, focused))
        .into()
}
