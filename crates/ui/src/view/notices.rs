use std::time::Instant;

use iced::widget::{Space, column, container, mouse_area, opaque, row, stack};
use iced::{Background, Border, Color, Element, Length, alignment};

use super::styles::error_inset_style;
use super::theme::{tokyo_board, tokyo_border, tokyo_text, ui_text};
use crate::app::{Message, SettingsSavedNotice, SettingsSavedNoticePresentation};
use crate::i18n::{Key, Lang};

const NOTICE_TOP: f32 = 96.0;
const SETTINGS_SAVED_NOTICE_TOP: f32 = 48.0;

pub(super) fn halt_notice_overlay(notice: &str) -> Element<'_, Message> {
    notice_overlay(
        notice,
        NOTICE_TOP,
        error_inset_style,
        Message::DismissHaltNotice,
    )
}

pub(super) fn error_notice_overlay(notice: &str) -> Element<'_, Message> {
    notice_overlay(
        notice,
        NOTICE_TOP,
        error_inset_style,
        Message::DismissErrorNotice,
    )
}

pub(super) fn with_settings_saved_notice<'a>(
    base: Element<'a, Message>,
    notice: Option<SettingsSavedNotice>,
    lang: Lang,
) -> Element<'a, Message> {
    let Some(notice) = notice else {
        return base;
    };
    stack![
        base,
        settings_saved_notice_overlay(
            lang.t(Key::SettingsSavedNotice),
            notice.presentation(Instant::now()),
        )
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn settings_saved_notice_overlay(
    notice: &'static str,
    presentation: SettingsSavedNoticePresentation,
) -> Element<'static, Message> {
    let opacity = presentation.opacity;
    let body = container(
        ui_text(notice, 15, faded(tokyo_text(), opacity)).align_x(alignment::Horizontal::Center),
    )
    .padding([12, 22])
    .style(move |_| settings_saved_style(opacity));
    let dismissible = mouse_area(opaque(body)).on_press(Message::DismissSettingsSavedNotice);
    column![
        Space::new().height(Length::Fixed(
            SETTINGS_SAVED_NOTICE_TOP + presentation.offset_y
        )),
        row![
            Space::new().width(Length::Fill),
            dismissible,
            Space::new().width(Length::Fill)
        ]
        .width(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn settings_saved_style(opacity: f32) -> iced::widget::container::Style {
    iced::widget::container::Style {
        text_color: Some(faded(tokyo_text(), opacity)),
        background: Some(Background::Color(faded(tokyo_board(), opacity))),
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: faded(tokyo_border(), opacity),
        },
        ..iced::widget::container::Style::default()
    }
}

fn faded(mut color: Color, opacity: f32) -> Color {
    color.a *= opacity;
    color
}

fn notice_overlay(
    notice: &str,
    top: f32,
    style: fn(&iced::Theme) -> iced::widget::container::Style,
    dismiss: Message,
) -> Element<'_, Message> {
    let body = container(
        ui_text(notice.to_owned(), 15, tokyo_text()).align_x(alignment::Horizontal::Center),
    )
    .padding([12, 22])
    .style(style);
    let dismissible = mouse_area(opaque(body)).on_press(dismiss);
    column![
        Space::new().height(Length::Fixed(top)),
        row![
            Space::new().width(Length::Fill),
            dismissible,
            Space::new().width(Length::Fill),
        ]
        .width(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

#[cfg(test)]
mod tests {
    use super::{NOTICE_TOP, SETTINGS_SAVED_NOTICE_TOP};

    #[test]
    fn settings_saved_notice_uses_the_higher_notice_lane() {
        assert_eq!(SETTINGS_SAVED_NOTICE_TOP, 48.0);
        assert_eq!(NOTICE_TOP - SETTINGS_SAVED_NOTICE_TOP, 48.0);
    }
}
