use iced::widget::{Space, button, container, row};
use iced::{Element, Length, alignment};

use super::super::theme::{TOKYO_TEXT, ui_text};
use super::consts::FOOTER_HEIGHT;
use super::styles::footer_button_style;
use crate::app::{FooterFocus, Message, SettingsDialog, SettingsSection};
use crate::i18n::{Key, Lang};

pub(super) fn settings_footer(dialog: &SettingsDialog, lang: Lang) -> Element<'static, Message> {
    let focus = dialog.footer_focus;
    let footer_active = dialog.section == SettingsSection::Footer;

    let reset = footer_button(
        lang.t(Key::SettingsReset),
        Message::SettingsResetRequested,
        footer_active && focus == FooterFocus::Reset,
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

    container(
        row![
            reset,
            Space::new().width(Length::Fill),
            cancel,
            Space::new().width(Length::Fixed(8.0)),
            save,
        ]
        .align_y(alignment::Vertical::Center),
    )
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

fn footer_button(label: &'static str, action: Message, focused: bool) -> Element<'static, Message> {
    button(container(ui_text(label, 13, TOKYO_TEXT)).padding([6, 16]))
        .on_press(action)
        .padding(0)
        .style(move |_theme, status| footer_button_style(status, focused))
        .into()
}
