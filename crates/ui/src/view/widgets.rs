//! Small reusable widgets that several panels share.
//!
//! Anything that lives here is purely visual sugar (a labelled frame, a
//! reusable button shape, …). Panel-level composition lives in the panel
//! modules themselves so this file does not turn into another monolith.

use iced::widget::{Space, button, column, container, row, stack, svg, text_input};
use iced::{Color, Element, Length, Padding, alignment};
use std::time::Duration;

use super::styles::{
    action_button_style, input_borderless_style, input_shell_style, legend_label_style,
    panel_style, schematic_block_style, step_button_style,
};
use super::theme::{MONO_FONT, TOKYO_GREEN, TOKYO_MUTED, TOKYO_TEXT, mono_text, ui_text};
use super::tooltips::hover_tooltip;
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
/// width – otherwise the focus glow gets clipped on the right edge.
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
    disabled: bool,
) -> Element<'a, Message> {
    const ARROW_RESERVED: f32 = 18.0;
    let padding_right = if disabled { 0.0 } else { ARROW_RESERVED };

    let input_style = if disabled {
        super::styles::disabled_input_borderless_style
            as fn(&iced::Theme, iced::widget::text_input::Status) -> iced::widget::text_input::Style
    } else {
        input_borderless_style
    };

    let mut input = text_input(placeholder, value)
        .id(id)
        .font(MONO_FONT)
        .size(16)
        .padding(Padding {
            top: 6.0,
            right: padding_right,
            bottom: 6.0,
            left: padding_right,
        })
        .align_x(alignment::Horizontal::Center)
        .width(Length::Fill)
        .style(input_style);

    if !disabled {
        input = input.on_input(on_input).on_submit(on_submit);
    }

    let input: Element<'a, Message> = input.into();

    let layered: Element<'a, Message> = if disabled {
        input
    } else {
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

        stack(vec![input, arrows])
            .width(Length::Fill)
            .height(Length::Shrink)
            .into()
    };

    container(layered)
        .width(width)
        .style(move |theme| input_shell_style(theme, focused && !disabled))
        .into()
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
    .width(Length::Fixed(34.0))
    .height(Length::Fixed(34.0))
    .style(move |_theme, status| super::styles::enter_button_style(status))
    .into()
}

pub(super) fn enter_button_disabled() -> Element<'static, Message> {
    const GLYPH_SIZE: f32 = 18.0;
    let glyph = svg(super::icons::chevrons_right())
        .width(Length::Fixed(GLYPH_SIZE))
        .height(Length::Fixed(GLYPH_SIZE))
        .style(|_theme, _status| svg::Style {
            color: Some(TOKYO_MUTED),
        });
    button(
        container(glyph)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center),
    )
    .padding(0)
    .width(Length::Fixed(34.0))
    .height(Length::Fixed(34.0))
    .style(move |_theme, status| super::styles::enter_button_style(status))
    .into()
}

pub(super) fn modal_icon_button(
    handle: svg::Handle,
    message: Message,
    tooltip_text: &'static str,
    size: f32,
) -> Element<'static, Message> {
    const GLYPH_SIZE: f32 = 18.0;

    let glyph = svg(handle)
        .width(Length::Fixed(GLYPH_SIZE))
        .height(Length::Fixed(GLYPH_SIZE))
        .style(|_theme, _status| svg::Style {
            color: Some(TOKYO_TEXT),
        });
    let face = button(
        container(glyph)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center),
    )
    .on_press(message)
    .padding(0)
    .width(Length::Fixed(size))
    .height(Length::Fixed(size))
    .style(move |_theme, status| super::styles::modal_field_button_style(status));

    hover_tooltip(
        face.into(),
        tooltip_text,
        None,
        iced::widget::tooltip::Position::Bottom,
        Duration::from_millis(350),
    )
}

pub(super) fn modal_footer_button(
    label_text: &'static str,
    message: Message,
    style: fn(button::Status) -> button::Style,
) -> Element<'static, Message> {
    button(
        container(ui_text(label_text, 14, TOKYO_TEXT))
            .padding([7, 22])
            .align_x(alignment::Horizontal::Center),
    )
    .on_press(message)
    .padding(0)
    .style(move |_theme, status| style(status))
    .into()
}

/// Square icon-only action button with hover tooltip. The SVG is
/// tinted with `accent`; passing `None` for `message` renders a
/// disabled chip (no `on_press`, faded glyph) – used by the post-HLT
/// latch to grey out execution chips while keeping resets clickable.
pub(super) fn icon_action_button(
    handle: svg::Handle,
    message: Option<Message>,
    accent: Color,
    hint: &'static str,
    shortcut: Option<&'static str>,
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

    hover_tooltip(
        face.into(),
        hint,
        shortcut,
        iced::widget::tooltip::Position::Top,
        Duration::from_millis(650),
    )
}
