//! Shared one-off helpers that do not deserve a dedicated module yet.

use iced::widget::{Space, container};
use iced::{Element, Length};

use super::styles::solid_style;
use super::theme::tokyo_subtle_line;
use crate::app::Message;

/// 1-pixel horizontal divider shared by compact row groups.
pub(super) fn row_separator() -> Element<'static, Message> {
    container(Space::new())
        .height(Length::Fixed(1.0))
        .width(Length::Fill)
        .style(|_theme| solid_style(tokyo_subtle_line(), 0.0))
        .into()
}
