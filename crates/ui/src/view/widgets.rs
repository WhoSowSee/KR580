//! Small reusable widgets that several panels share.
//!
//! Anything that lives here is purely visual sugar (a labelled frame, a
//! reusable button shape, …). Panel-level composition lives in the panel
//! modules themselves so this file does not turn into another monolith.

use iced::widget::{Space, button, column, container, mouse_area, row, stack, text_input};
use iced::{Color, Element, Length, Padding, alignment};

use super::styles::{
    capsule_button_style, input_borderless_style, input_shell_style, legend_label_style,
    panel_style, step_button_style,
};
use super::theme::{MONO_FONT, TOKYO_GREEN, TOKYO_TEXT, mono_text, ui_text};
use crate::app::Message;

/// Frames `content` with a border that has a centred title cut into it.
/// `height` controls how much vertical space the framed area takes.
pub(super) fn legend_panel<'a>(
    title: impl Into<String>,
    content: impl Into<Element<'a, Message>>,
    height: Length,
) -> Element<'a, Message> {
    const LEGEND_LINE_OFFSET: f32 = 9.0;

    let panel: Element<'a, Message> = container(content)
        .padding(Padding {
            top: 18.0,
            right: 10.0,
            bottom: 10.0,
            left: 10.0,
        })
        .width(Length::Fill)
        .height(height)
        .style(panel_style)
        .into();
    let framed_panel: Element<'a, Message> = column![
        Space::new().height(Length::Fixed(LEGEND_LINE_OFFSET)),
        panel,
    ]
    .spacing(0)
    .width(Length::Fill)
    .height(height)
    .into();
    let legend: Element<'a, Message> = row![
        Space::new().width(Length::Fill),
        container(ui_text(title, 14, TOKYO_TEXT))
            .padding([0, 5])
            .style(legend_label_style),
        Space::new().width(Length::Fill),
    ]
    .width(Length::Fill)
    .into();

    stack(vec![framed_panel, legend])
        .width(Length::Fill)
        .height(height)
        .into()
}

/// Builds a `<text_input> + ▲▼` shell that publishes `up`/`down` messages
/// for the spinner buttons and reports hover/focus state through the
/// border colour. `id` makes the inner input addressable for focus
/// operations.
#[allow(clippy::too_many_arguments)]
pub(super) fn spinner_text_input<'a>(
    placeholder: &'static str,
    value: &'a str,
    on_input: fn(String) -> Message,
    up: Message,
    down: Message,
    width: Length,
    on_submit: Message,
    id: &'static str,
    focused: bool,
    hovered: bool,
) -> Element<'a, Message> {
    let shell: Element<'a, Message> = container(
        row![
            text_input(placeholder, value)
                .id(id)
                .on_input(on_input)
                .on_submit(on_submit)
                .font(MONO_FONT)
                .size(16)
                .padding(6)
                .align_x(alignment::Horizontal::Center)
                .width(Length::Fill)
                .style(input_borderless_style),
            column![step_button("▲", up), step_button("▼", down)].spacing(0),
        ]
        .spacing(0)
        .align_y(alignment::Vertical::Center),
    )
    .width(width)
    .style(move |theme| input_shell_style(theme, focused, hovered))
    .into();

    mouse_area(shell)
        .on_enter(Message::SpinnerHovered { id, hovered: true })
        .on_exit(Message::SpinnerHovered { id, hovered: false })
        .into()
}

pub(super) fn step_button(label: &'static str, message: Message) -> Element<'static, Message> {
    button(mono_text(label, 9, TOKYO_TEXT).align_x(alignment::Horizontal::Center))
        .on_press(message)
        .padding(1)
        .width(Length::Fixed(18.0))
        .height(Length::Fixed(17.0))
        .style(move |_theme, status| step_button_style(status))
        .into()
}

pub(super) fn enter_button(message: Message) -> Element<'static, Message> {
    icon_button("↵", message, TOKYO_GREEN)
}

fn icon_button(label: &'static str, message: Message, accent: Color) -> Element<'static, Message> {
    button(mono_text(label, 14, accent).align_x(alignment::Horizontal::Center))
        .on_press(message)
        .padding(6)
        .style(move |_theme, status| capsule_button_style(status, accent, false))
        .into()
}
