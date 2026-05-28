//! Small reusable widgets that several panels share.
//!
//! Anything that lives here is purely visual sugar (a labelled frame, a
//! reusable button shape, …). Panel-level composition lives in the panel
//! modules themselves so this file does not turn into another monolith.

use iced::widget::{Space, button, column, container, row, stack, svg, text_input, tooltip};
use iced::{Color, Element, Length, Padding, alignment};
use std::time::Duration;

use super::styles::{
    action_button_style, enter_button_style, input_borderless_style, input_shell_style,
    inset_style, legend_label_style, panel_style, schematic_block_style, step_button_style,
};
use super::theme::{MONO_FONT, TOKYO_GREEN, TOKYO_MUTED, TOKYO_TEXT, mono_text, ui_text};
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

pub(super) fn legend_panel_left<'a>(
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
        .style(schematic_block_style)
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

/// `<text_input> + ▲▼` shell. Arrows are stacked over the input
/// rather than placed in a row so the input keeps the full inner
/// width — otherwise the focus glow gets clipped on the right edge.
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
    // Reserved for the right-side arrow stack; mirrored as left
    // padding so the typed text stays centred.
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
    const GLYPH_SIZE: f32 = 18.0;
    let glyph = svg(super::icons::chevrons_right())
        .width(Length::Fixed(GLYPH_SIZE))
        .height(Length::Fixed(GLYPH_SIZE))
        .style(|_theme, _status| svg::Style {
            color: Some(TOKYO_GREEN),
        });
    button(
        container(glyph)
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

/// Square icon-only action button with hover tooltip. The SVG is
/// tinted with `accent`; passing `None` for `message` renders a
/// disabled chip (no `on_press`, faded glyph) — used by the post-HLT
/// latch to grey out execution chips while keeping resets clickable.
pub(super) fn icon_action_button(
    handle: svg::Handle,
    message: Option<Message>,
    accent: Color,
    hint: &'static str,
) -> Element<'static, Message> {
    const BUTTON_SIZE: f32 = 38.0;
    const GLYPH_SIZE: f32 = 20.0;

    // Disabled chip → low-alpha muted grey. The style callback's host
    // is a `container`, not the `button`, so its status stays `Idle`
    // and we have to bake the enabled/disabled choice in at build time.
    let enabled = message.is_some();
    let glyph_color = if enabled {
        accent
    } else {
        Color {
            a: 0.45,
            ..TOKYO_MUTED
        }
    };
    let glyph = svg(handle)
        .width(Length::Fixed(GLYPH_SIZE))
        .height(Length::Fixed(GLYPH_SIZE))
        .style(move |_theme, _status| svg::Style {
            color: Some(glyph_color),
        });

    let mut face = button(
        container(glyph)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center),
    )
    .padding(0)
    .width(Length::Fixed(BUTTON_SIZE))
    .height(Length::Fixed(BUTTON_SIZE))
    .style(move |_theme, status| action_button_style(status));
    if let Some(action) = message {
        face = face.on_press(action);
    }

    let body = container(ui_text(hint, 12, TOKYO_TEXT))
        .padding(Padding {
            top: 4.0,
            right: 8.0,
            bottom: 4.0,
            left: 8.0,
        })
        .style(inset_style);

    tooltip(face, body, tooltip::Position::Top)
        .gap(4.0)
        .padding(0.0)
        .delay(Duration::from_millis(650))
        .snap_within_viewport(true)
        .into()
}
