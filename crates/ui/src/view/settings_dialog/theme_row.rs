use iced::widget::{Column, Row, Space, button, container};
use iced::{Background, Border, Color, Element, Length, alignment};

use super::super::theme::{
    DARK_COLOR_SCHEMES, LIGHT_COLOR_SCHEMES, color_scheme_group_label, color_scheme_label,
    color_scheme_palette, tokyo_blue, tokyo_border, tokyo_muted, tokyo_selection_blue,
    tokyo_surface, tokyo_surface_2, tokyo_text, ui_text,
};
use crate::app::{ContentFocus, Message, SettingsDialog, SettingsSection};
use crate::i18n::Lang;
use crate::persistence::ColorScheme;

pub(super) fn theme_setting_row<'a>(
    dialog: &'a SettingsDialog,
    lang: Lang,
) -> Element<'a, Message> {
    let kb_focus = (dialog.section == SettingsSection::Content)
        .then_some(dialog.content_focus)
        .flatten();
    let keyboard_focused = kb_focus == Some(ContentFocus::Theme);

    let control = Column::new()
        .spacing(12)
        .push(theme_group(
            color_scheme_group_label(true, lang),
            &DARK_COLOR_SCHEMES,
            dialog.draft_color_scheme,
            keyboard_focused,
            lang,
        ))
        .push(theme_group(
            color_scheme_group_label(false, lang),
            &LIGHT_COLOR_SCHEMES,
            dialog.draft_color_scheme,
            keyboard_focused,
            lang,
        ));

    container(control).width(Length::Fill).into()
}

fn theme_group<'a>(
    label: &'static str,
    schemes: &'static [ColorScheme],
    active: ColorScheme,
    keyboard_focused: bool,
    lang: Lang,
) -> Element<'a, Message> {
    let mut column = Column::new()
        .spacing(6)
        .push(ui_text(label, 13, tokyo_muted()));
    for scheme in schemes {
        column = column.push(theme_option(*scheme, active, keyboard_focused, lang));
    }
    column.into()
}

fn theme_option<'a>(
    scheme: ColorScheme,
    active: ColorScheme,
    keyboard_focused: bool,
    lang: Lang,
) -> Element<'a, Message> {
    let selected = scheme == active;
    let content = Row::new()
        .align_y(alignment::Vertical::Center)
        .push(ui_text(color_scheme_label(scheme, lang), 15, tokyo_text()))
        .push(Space::new().width(Length::Fill))
        .push(palette_preview(scheme));

    button(container(content).padding([8, 12]))
        .on_press(Message::SettingsDraftColorSchemeChanged(scheme))
        .width(Length::Fill)
        .padding(0)
        .style(move |_theme, status| theme_option_style(status, selected, keyboard_focused))
        .into()
}

fn palette_preview<'a>(scheme: ColorScheme) -> Element<'a, Message> {
    let mut row = Row::new().spacing(6).align_y(alignment::Vertical::Center);
    for color in color_scheme_palette(scheme) {
        row = row.push(palette_square(color));
    }
    row.into()
}

pub(super) fn theme_search_matches(lang: Lang, lower_query: &str) -> bool {
    DARK_COLOR_SCHEMES
        .iter()
        .chain(LIGHT_COLOR_SCHEMES.iter())
        .any(|scheme| {
            color_scheme_label(*scheme, lang)
                .to_lowercase()
                .contains(lower_query)
        })
}

fn palette_square<'a>(color: Color) -> Element<'a, Message> {
    container(Space::new())
        .width(Length::Fixed(15.0))
        .height(Length::Fixed(15.0))
        .style(move |_theme| container::Style {
            background: Some(Background::Color(color)),
            border: Border {
                radius: 3.0.into(),
                width: 1.0,
                color: Color {
                    a: 0.45,
                    ..tokyo_border()
                },
            },
            ..container::Style::default()
        })
        .into()
}

fn theme_option_style(
    status: button::Status,
    selected: bool,
    keyboard_focused: bool,
) -> button::Style {
    let background = match (selected, status) {
        (true, _) => tokyo_selection_blue(),
        (false, button::Status::Pressed) => tokyo_surface_2(),
        (false, button::Status::Hovered) => tokyo_surface(),
        _ => Color::TRANSPARENT,
    };
    let border_color = if selected {
        tokyo_blue()
    } else if keyboard_focused {
        tokyo_text()
    } else {
        Color::TRANSPARENT
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: tokyo_text(),
        border: Border {
            radius: 6.0.into(),
            width: if selected || keyboard_focused {
                1.0
            } else {
                0.0
            },
            color: border_color,
        },
        ..button::Style::default()
    }
}
