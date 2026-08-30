mod content;
mod controls;
mod finish;
mod rail;

use super::locale::Text as T;
use super::{Installer, Message, window_chrome};
use iced::widget::column;
use iced::{Element, Length};

pub(super) fn view(app: &Installer) -> Element<'_, Message> {
    column![
        window_chrome::title_bar(
            app.locale.t(T::WindowTitleInstaller),
            app.window_maximized,
            window_chrome::CaptionMessages {
                drag: Message::WindowDragStart,
                minimize: Message::WindowMinimize,
                toggle_maximize: Message::WindowToggleMaximize,
                close: Some(Message::WindowClose),
            },
        ),
        content::content(app)
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
