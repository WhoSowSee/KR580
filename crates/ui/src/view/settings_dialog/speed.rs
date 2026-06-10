use iced::widget::{button, container, row};
use iced::{Element, Length, alignment};

use super::super::theme::{TOKYO_TEXT, ui_text};
use super::consts::SPEED_SEGMENT_WIDTH;
use super::setting_row::setting_row;
use super::styles::segmented_button_style;
use crate::app::{ContentFocus, Message, SettingsDialog, SettingsSection, SpeedTier};
use crate::i18n::{Key, Lang};

pub(super) fn speed_setting_row<'a>(
    dialog: &'a SettingsDialog,
    lang: Lang,
) -> Element<'a, Message> {
    let kb_focus = (dialog.section == SettingsSection::Content)
        .then_some(dialog.content_focus)
        .flatten();
    let kb_focused_for = |c: ContentFocus| kb_focus == Some(c);

    let segments = row![
        segmented_button(
            lang.t(Key::SpeedSlow),
            dialog.draft_speed == SpeedTier::Slow,
            kb_focused_for(ContentFocus::SpeedSlow),
            Message::SettingsDraftSpeedChanged(SpeedTier::Slow),
        ),
        segmented_button(
            lang.t(Key::SpeedMedium),
            dialog.draft_speed == SpeedTier::Medium,
            kb_focused_for(ContentFocus::SpeedMedium),
            Message::SettingsDraftSpeedChanged(SpeedTier::Medium),
        ),
        segmented_button(
            lang.t(Key::SpeedHigh),
            dialog.draft_speed == SpeedTier::High,
            kb_focused_for(ContentFocus::SpeedFast),
            Message::SettingsDraftSpeedChanged(SpeedTier::High),
        ),
        segmented_button(
            lang.t(Key::SpeedMax),
            dialog.draft_speed == SpeedTier::Max,
            kb_focused_for(ContentFocus::SpeedMax),
            Message::SettingsDraftSpeedChanged(SpeedTier::Max),
        ),
    ]
    .spacing(6);

    setting_row(
        lang.t(Key::SettingsSpeedLabel),
        lang.t(Key::SettingsSpeedHint),
        segments.into(),
    )
}

pub(super) fn segmented_button(
    label: &'static str,
    active: bool,
    keyboard_focused: bool,
    action: Message,
) -> Element<'static, Message> {
    segmented_button_width(label, active, keyboard_focused, action, SPEED_SEGMENT_WIDTH)
}

pub(super) fn segmented_button_width(
    label: &'static str,
    active: bool,
    keyboard_focused: bool,
    action: Message,
    width: f32,
) -> Element<'static, Message> {
    button(
        container(ui_text(label, 13, TOKYO_TEXT))
            .padding([6, 0])
            .width(Length::Fixed(width))
            .align_x(alignment::Horizontal::Center),
    )
    .on_press(action)
    .padding(0)
    .style(move |_theme, status| segmented_button_style(status, active, keyboard_focused))
    .into()
}
