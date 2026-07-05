use iced::widget::{button, column, container, row, svg, text_input};
use iced::{Element, Length, alignment};

use super::super::icons;
use super::super::styles::input_borderless_style;
use super::super::theme::{tokyo_muted, tokyo_text, ui_text};
use super::consts::{SEARCH_ICON_SIZE, SIDEBAR_WIDTH};
use super::styles::{section_input_style, sidebar_chip_style};
use crate::app::{
    Message, SETTINGS_SEARCH_INPUT_ID, SettingsCategory, SettingsDialog, SettingsSection,
};
use crate::i18n::{Key, Lang};

pub(super) fn settings_sidebar<'a>(dialog: &'a SettingsDialog, lang: Lang) -> Element<'a, Message> {
    let search_active = dialog.section == SettingsSection::Search;
    let sidebar_active = dialog.section == SettingsSection::Sidebar;

    let search_icon = svg(icons::search())
        .width(Length::Fixed(SEARCH_ICON_SIZE))
        .height(Length::Fixed(SEARCH_ICON_SIZE))
        .style(|_theme, _status| svg::Style {
            color: Some(tokyo_muted()),
        });

    let search_field = text_input(lang.t(Key::SettingsSearchPlaceholder), &dialog.search)
        .id(SETTINGS_SEARCH_INPUT_ID)
        .on_input(Message::SettingsSearchChanged)
        .padding(0)
        .size(13)
        .style(input_borderless_style);

    let search_row = row![search_icon, search_field]
        .spacing(8)
        .align_y(alignment::Vertical::Center);

    let search = container(search_row)
        .padding([6, 8])
        .style(move |_theme| section_input_style(search_active));

    let mut items: Vec<Element<'a, Message>> = Vec::with_capacity(SettingsCategory::ALL.len());
    for cat in SettingsCategory::ALL {
        items.push(category_chip(
            cat,
            dialog.category == cat,
            sidebar_active && dialog.category == cat,
            lang,
        ));
    }

    container(
        column![
            container(search).padding(iced::Padding {
                top: 16.0,
                right: 12.0,
                bottom: 8.0,
                left: 12.0,
            }),
            column(items).spacing(4).padding(iced::Padding {
                top: 4.0,
                right: 12.0,
                bottom: 12.0,
                left: 12.0,
            }),
        ]
        .width(Length::Fill),
    )
    .width(Length::Fixed(SIDEBAR_WIDTH))
    .height(Length::Fill)
    .into()
}

fn category_chip(
    category: SettingsCategory,
    active: bool,
    keyboard_focused: bool,
    lang: Lang,
) -> Element<'static, Message> {
    let label = ui_text(lang.t(category.label_key()), 13, tokyo_text());

    button(
        container(label)
            .padding([8, 12])
            .width(Length::Fill)
            .align_y(alignment::Vertical::Center),
    )
    .on_press(Message::SettingsCategorySelected(category))
    .padding(0)
    .width(Length::Fill)
    .style(move |_theme, status| sidebar_chip_style(status, active, keyboard_focused))
    .into()
}
