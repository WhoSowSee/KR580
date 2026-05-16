//! Small reusable widgets that several panels share.
//!
//! Anything that lives here is purely visual sugar (a labelled frame, a
//! reusable button shape, …). Panel-level composition lives in the panel
//! modules themselves so this file does not turn into another monolith.

use iced::widget::{Space, button, column, container, row, stack, text_input};
use iced::{Element, Length, Padding, alignment};

use super::styles::{
    enter_button_style, input_borderless_style, input_shell_style, legend_label_style, panel_style,
    step_button_style,
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
/// for the spinner buttons and reports focus state through the border
/// colour. `id` makes the inner input addressable for focus operations.
///
/// The arrows live in a stacked overlay rather than next to the input in
/// a row, so the input gets the full inner width of the shell. That keeps
/// the focus glow from being clipped on the right edge by a sibling
/// column, which used to leave a visible artefact behind the buttons.
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
) -> Element<'a, Message> {
    // Width reserved for the arrow stack on the right side of the shell.
    // The same amount is added as left padding so the entered text stays
    // centred horizontally relative to the visible border.
    const ARROW_RESERVED: f32 = 18.0;

    let input: Element<'a, Message> = text_input(placeholder, value)
        .id(id)
        .on_input(on_input)
        .on_submit(on_submit)
        .font(MONO_FONT)
        .size(16)
        .padding(Padding {
            top: 6.0,
            right: ARROW_RESERVED,
            bottom: 6.0,
            left: ARROW_RESERVED,
        })
        .align_x(alignment::Horizontal::Center)
        .width(Length::Fill)
        .style(input_borderless_style)
        .into();

    let arrows: Element<'a, Message> =
        container(column![step_button("▲", up), step_button("▼", down)].spacing(0))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Padding {
                top: 0.0,
                right: 4.0,
                bottom: 0.0,
                left: 0.0,
            })
            .align_x(alignment::Horizontal::Right)
            .align_y(alignment::Vertical::Center)
            .into();

    let layered: Element<'a, Message> = stack(vec![input, arrows])
        .width(Length::Fill)
        .height(Length::Shrink)
        .into();

    let shell: Element<'a, Message> = container(layered)
        .width(width)
        .style(move |theme| input_shell_style(theme, focused))
        .into();

    shell
}

pub(super) fn step_button(label: &'static str, message: Message) -> Element<'static, Message> {
    button(mono_text(label, 8, TOKYO_TEXT).align_x(alignment::Horizontal::Center))
        .on_press(message)
        .padding(0)
        .width(Length::Fixed(14.0))
        .height(Length::Fixed(13.0))
        .style(move |_theme, status| step_button_style(status))
        .into()
}

pub(super) fn enter_button(message: Message) -> Element<'static, Message> {
    button(
        mono_text("↵", 14, TOKYO_GREEN)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center),
    )
    .on_press(message)
    .padding(0)
    .width(Length::Fixed(28.0))
    .height(Length::Fixed(28.0))
    .style(move |_theme, status| enter_button_style(status))
    .into()
}
