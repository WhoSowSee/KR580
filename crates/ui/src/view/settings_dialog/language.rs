use iced::widget::{Space, button, column, container, row, svg};
use iced::{Element, Length, alignment};

use super::super::icons;
use super::super::theme::{tokyo_muted, tokyo_text, ui_text};
use super::consts::{DROPDOWN_CHEVRON_SIZE, LANGUAGE_PICKER_WIDTH};
use super::setting_row::setting_row;
use super::styles::{dropdown_anchor_style, dropdown_option_style, dropdown_panel_style};
use crate::app::{ContentFocus, Message, SettingsDialog};
use crate::i18n::{Key, Lang};

pub(super) fn language_setting_row<'a>(
    dialog: &'a SettingsDialog,
    lang: Lang,
) -> Element<'a, Message> {
    let active_label = lang.t(language_label_key(dialog.draft_lang));
    let keyboard_focused = dialog.content_focus_is_visible(ContentFocus::LanguageAnchor);

    let chevron = svg(icons::chevron_down())
        .width(Length::Fixed(DROPDOWN_CHEVRON_SIZE))
        .height(Length::Fixed(DROPDOWN_CHEVRON_SIZE))
        .style(|_theme, _status| svg::Style {
            color: Some(tokyo_muted()),
        });

    let anchor = button(
        container(
            row![
                ui_text(active_label, 13, tokyo_text()),
                Space::new().width(Length::Fill),
                chevron,
            ]
            .align_y(alignment::Vertical::Center)
            .spacing(10),
        )
        .padding([8, 12])
        .width(Length::Fixed(LANGUAGE_PICKER_WIDTH)),
    )
    .on_press(Message::SettingsLanguageDropdownToggled)
    .padding(0)
    .style(move |_theme, status| {
        dropdown_anchor_style(status, dialog.language_dropdown_open, keyboard_focused)
    });

    setting_row(
        lang.t(Key::SettingsLanguageLabel),
        lang.t(Key::SettingsLanguageHint),
        anchor.into(),
    )
}

pub(super) fn language_dropdown_list(
    selected: Option<Lang>,
    highlighted: Lang,
    lang: Lang,
) -> Element<'static, Message> {
    let options = column![
        language_dropdown_option(
            Lang::Ru,
            selected == Some(Lang::Ru),
            highlighted == Lang::Ru,
            lang
        ),
        language_dropdown_option(
            Lang::En,
            selected == Some(Lang::En),
            highlighted == Lang::En,
            lang
        ),
    ]
    .spacing(0);

    container(options)
        .padding(4)
        .width(Length::Fixed(LANGUAGE_PICKER_WIDTH))
        .style(dropdown_panel_style)
        .into()
}

fn language_dropdown_option(
    target: Lang,
    selected: bool,
    highlighted: bool,
    lang: Lang,
) -> Element<'static, Message> {
    let label = lang.t(language_label_key(target));
    button(
        container(ui_text(label, 13, tokyo_text()))
            .padding([6, 10])
            .width(Length::Fill),
    )
    .on_press(Message::SettingsDraftLanguageChanged(target))
    .padding(0)
    .width(Length::Fill)
    .style(move |_theme, status| dropdown_option_style(status, selected, highlighted))
    .into()
}

pub(super) fn language_label_key(target: Lang) -> Key {
    match target {
        Lang::Ru => Key::LangRussian,
        Lang::En => Key::LangEnglish,
    }
}
