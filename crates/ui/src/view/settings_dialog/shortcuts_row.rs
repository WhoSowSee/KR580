use iced::widget::{Space, button, column, container, row};
use iced::{Background, Border, Color, Element, Length, alignment};

use super::super::theme::{
    TOKYO_BORDER, TOKYO_SURFACE, TOKYO_SURFACE_2, TOKYO_TEXT, mono_text, ui_text,
};
use crate::app::shortcuts::shortcut_action_label;
use crate::app::{ContentFocus, Message, SettingsDialog, SettingsSection};
use crate::i18n::Lang;
use crate::persistence::ShortcutAction;

const ACTION_COLUMN_WIDTH: f32 = 260.0;
const BINDING_COLUMN_WIDTH: f32 = 172.0;
const ROW_HEIGHT: f32 = 34.0;

pub(super) fn shortcuts_setting_row<'a>(
    dialog: &'a SettingsDialog,
    lang: Lang,
) -> Element<'a, Message> {
    let mut table = column![].spacing(6);
    for action in ShortcutAction::ALL {
        table = table.push(shortcut_row(dialog, lang, action));
    }

    container(table).width(Length::Fill).into()
}

fn shortcut_row<'a>(
    dialog: &'a SettingsDialog,
    lang: Lang,
    action: ShortcutAction,
) -> Element<'a, Message> {
    let focused = dialog.section == SettingsSection::Content
        && dialog.content_focus == Some(ContentFocus::Shortcut(action));
    let recording = dialog.recording_shortcut == Some(action);
    let label = if recording {
        capture_prompt(lang).to_owned()
    } else {
        dialog
            .draft_shortcuts
            .binding(action)
            .map(|binding| binding.label())
            .unwrap_or_else(|| unassigned_label(lang).to_owned())
    };
    let action_label = ui_text(shortcut_action_label(action, lang), 13, TOKYO_TEXT);

    let capture_button = button(
        container(mono_text(label, 12, TOKYO_TEXT))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center),
    )
    .on_press(Message::SettingsShortcutCaptureStarted(action))
    .padding(0)
    .width(Length::Fixed(BINDING_COLUMN_WIDTH))
    .height(Length::Fixed(ROW_HEIGHT - 4.0))
    .style(move |_theme, status| shortcut_button_style(status, focused, recording));

    container(
        row![
            container(action_label)
                .width(Length::Fixed(ACTION_COLUMN_WIDTH))
                .height(Length::Fixed(ROW_HEIGHT))
                .align_y(alignment::Vertical::Center),
            Space::new().width(Length::Fill),
            capture_button,
        ]
        .spacing(10)
        .align_y(alignment::Vertical::Center),
    )
    .width(Length::Fill)
    .height(Length::Fixed(ROW_HEIGHT))
    .into()
}

fn capture_prompt(lang: Lang) -> &'static str {
    match lang {
        Lang::Ru => "Нажмите сочетание",
        Lang::En => "Press shortcut",
    }
}

fn unassigned_label(lang: Lang) -> &'static str {
    match lang {
        Lang::Ru => "Не назначено",
        Lang::En => "Unassigned",
    }
}

fn shortcut_button_style(status: button::Status, focused: bool, recording: bool) -> button::Style {
    let background = match (recording, focused, status) {
        (true, _, button::Status::Pressed) => TOKYO_SURFACE_2,
        (true, _, _) => TOKYO_SURFACE,
        (false, true, _) => TOKYO_SURFACE,
        (false, false, button::Status::Hovered) => Color {
            a: 0.65,
            ..TOKYO_SURFACE
        },
        (false, false, button::Status::Pressed) => TOKYO_SURFACE_2,
        _ => Color::TRANSPARENT,
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: TOKYO_TEXT,
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: TOKYO_BORDER,
        },
        ..button::Style::default()
    }
}
