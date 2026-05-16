//! Shared one-off helpers that do not deserve a dedicated module yet.

use iced::widget::{Space, container};
use iced::{Color, Element, Length};

use super::styles::solid_style;
use crate::app::Message;

/// 1-pixel horizontal divider used between rows in the memory list and
/// inside the opcode dropdown.
pub(super) fn row_separator() -> Element<'static, Message> {
    container(Space::new())
        .height(Length::Fixed(1.0))
        .width(Length::Fill)
        .style(|_theme| solid_style(Color::from_rgba8(0x41, 0x48, 0x68, 0.26), 0.0))
        .into()
}
