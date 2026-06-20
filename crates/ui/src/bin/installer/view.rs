mod chrome;
mod content;
mod finish;

use super::{Installer, Message};
use iced::widget::column;
use iced::{Element, Length};

pub fn view(app: &Installer) -> Element<'_, Message> {
    column![chrome::title_bar(app), content::content(app)]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
