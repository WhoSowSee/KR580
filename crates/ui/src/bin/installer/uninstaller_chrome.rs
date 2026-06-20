use super::super::style;
use super::Message;
use iced::widget::{Space, button, column, container, mouse_area, row, svg, text};
use iced::{Alignment, Element, Length, alignment};
use std::sync::LazyLock;

const CAPTION_ICON_SIZE: f32 = 14.0;
const CAPTION_CLOSE_ICON_SIZE: f32 = 16.0;
const CAPTION_BUTTON_WIDTH: f32 = 32.0;
const CAPTION_BUTTON_HEIGHT: f32 = 24.0;

macro_rules! action_icon_bytes {
    ($name:literal) => {
        include_bytes!(concat!(
            "../../../../../assets/icons/actions/",
            $name,
            ".svg"
        ))
    };
}

static WINDOW_MINIMIZE: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("window-minimize").as_slice()));
static WINDOW_CLOSE: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("window-close").as_slice()));

pub(super) fn title_bar(uninstalling: bool, closing: bool) -> Element<'static, Message> {
    let title = text("KR580 Setup")
        .font(style::FONT_BOLD)
        .size(14)
        .color(style::TEXT);

    let drag_handle: Element<'static, Message> = mouse_area(
        container(Space::new())
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .on_press(Message::WindowDragStart)
    .into();

    let body = row![
        container(title)
            .padding(iced::Padding::ZERO.left(11))
            .height(Length::Fill)
            .align_y(alignment::Vertical::Center),
        drag_handle,
        caption_button(
            WINDOW_MINIMIZE.clone(),
            Some(Message::WindowMinimize),
            false
        ),
        caption_button(
            WINDOW_CLOSE.clone(),
            (!uninstalling && !closing).then_some(Message::WindowClose),
            true,
        ),
    ]
    .spacing(2)
    .align_y(Alignment::Center);

    column![
        container(body)
            .height(Length::Fixed(34.0))
            .width(Length::Fill)
            .style(style::titlebar),
        container(Space::new())
            .height(Length::Fixed(1.0))
            .width(Length::Fill)
            .style(style::titlebar_divider),
    ]
    .into()
}

fn caption_button(
    icon: svg::Handle,
    message: Option<Message>,
    close: bool,
) -> Element<'static, Message> {
    let glyph_size = if close {
        CAPTION_CLOSE_ICON_SIZE
    } else {
        CAPTION_ICON_SIZE
    };
    let glyph = svg(icon)
        .width(Length::Fixed(glyph_size))
        .height(Length::Fixed(glyph_size))
        .style(|_theme, _status| svg::Style {
            color: Some(style::TEXT),
        });

    let body = container(glyph)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center);

    button(body)
        .padding(0)
        .width(Length::Fixed(CAPTION_BUTTON_WIDTH))
        .height(Length::Fixed(CAPTION_BUTTON_HEIGHT))
        .style(move |_theme, status| {
            if close {
                style::close_caption_button(status)
            } else {
                style::caption_button(status)
            }
        })
        .on_press_maybe(message)
        .into()
}
