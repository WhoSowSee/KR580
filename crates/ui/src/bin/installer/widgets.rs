use super::style;
use iced::widget::{Space, button, checkbox as iced_checkbox, container, text};
use iced::{Element, Length, alignment};

const ACTION_WIDTH: f32 = 176.0;
const ACTION_HEIGHT: f32 = 40.0;

pub(super) fn command_bar<M: Clone + 'static>(
    label: &'static str,
    message: Option<M>,
) -> Element<'static, M> {
    container(primary_action(label, message))
        .padding([10, 16])
        .height(Length::Fixed(64.0))
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Right)
        .style(style::command_bar)
        .into()
}

fn primary_action<M: Clone + 'static>(
    label: &'static str,
    message: Option<M>,
) -> Element<'static, M> {
    button(
        container(
            text(label)
                .font(style::FONT_BOLD)
                .size(14)
                .align_x(alignment::Horizontal::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center),
    )
    .padding(0)
    .width(Length::Fixed(ACTION_WIDTH))
    .height(Length::Fixed(ACTION_HEIGHT))
    .style(style::primary_button)
    .on_press_maybe(message)
    .into()
}

pub(super) fn checkbox<'a, M: Clone + 'a>(
    checked: bool,
    label: &'a str,
    message: fn(bool) -> M,
) -> Element<'a, M> {
    iced_checkbox(checked)
        .label(label)
        .on_toggle(message)
        .font(style::FONT)
        .text_size(14)
        .size(style::CHECKBOX_SIZE)
        .spacing(10)
        .style(style::check)
        .into()
}

pub(super) fn horizontal_rule<M: 'static>() -> Element<'static, M> {
    container(Space::new())
        .height(Length::Fixed(1.0))
        .width(Length::Fill)
        .style(|_| style::divider())
        .into()
}

pub(super) fn vertical_rule<M: 'static>() -> Element<'static, M> {
    container(Space::new())
        .width(Length::Fixed(1.0))
        .height(Length::Fill)
        .style(|_| style::divider())
        .into()
}
