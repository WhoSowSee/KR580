use iced::widget::{Space, column, container, mouse_area, opaque, row};
use iced::{Element, Length, alignment};

use super::styles::error_inset_style;
use super::theme::{TOKYO_TEXT, ui_text};
use crate::app::Message;

const NOTICE_TOP: f32 = 96.0;

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

fn notice_overlay(
    notice: &str,
    top: f32,
    style: fn(&iced::Theme) -> iced::widget::container::Style,
    dismiss: Message,
) -> Element<'_, Message> {
    let body = container(
        ui_text(notice.to_owned(), 15, TOKYO_TEXT).align_x(alignment::Horizontal::Center),
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
