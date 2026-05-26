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
    // The glyph is `chevrons-right` from the Lucide set: twin
    // right-pointing chevrons, tinted green to mirror the action-panel
    // accent palette ("commit and move on"). Sized to 18 px so it sits
    // visually centred inside the 28 px square chrome.
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

/// A square icon-only action button with a hover tooltip. Used to
/// compose the "Управление" panel as a single horizontal row of glyph
/// chips, mirroring the toolbar look of the reference KR-580 emulator.
/// The SVG glyph is tinted with `accent` so each button is unambiguously
/// identified by its colour; the surrounding chrome stays neutral and
/// only the surface tone shifts on hover/press, matching the editor `↵`
/// button so the chips read as part of the same family. The tooltip
/// body uses the editor `inset_style` so it visually belongs to the
/// same chrome family as the rest of the side panel.
///
/// `message` is `Option<Message>` rather than `Message` so the caller
/// can render a *disabled* chip without juggling a parallel "is it
/// enabled" branch in the layout: passing `None` skips the
/// `.on_press(...)` call, and iced 0.14's `button` widget treats a
/// button with no `on_press` as non-interactive (no hover highlight,
/// no click). The tooltip still appears on hover so the user can see
/// the hint even while the button is locked. This is the chokepoint
/// that the post-HLT run-block (`run_blocked_after_halt`) uses to
/// grey out the four execution chips while leaving the reset chips
/// alive — the resets are the way out of the latch, so they must
/// stay clickable.
pub(super) fn icon_action_button(
    handle: svg::Handle,
    message: Option<Message>,
    accent: Color,
    hint: &'static str,
) -> Element<'static, Message> {
    const BUTTON_SIZE: f32 = 38.0;
    const GLYPH_SIZE: f32 = 20.0;

    // Tint the SVG geometry by overriding `currentColor` with the
    // accent — the source files are authored with `stroke="currentColor"`
    // so the same handle can be reused at any tone. When the chip is
    // locked (`message` is `None`), we collapse the glyph to a low-alpha
    // `TOKYO_MUTED` grey instead. iced 0.14's `svg::Style` callback
    // also receives the host widget's status, but for this widget the
    // host is a `container`, not the `button` itself, so the status is
    // always `Idle` and we cannot key off it. The enabled/disabled
    // decision is therefore baked into the closure at construction
    // time — same lifetime as the `Option<Message>` we already branch
    // on for `.on_press(...)`. This is what makes the chip read as
    // "out of service" at a glance: the frame fades via
    // `action_button_style`'s `Disabled` arm, the glyph fades here,
    // and together they outvoter the per-chip accent colour the chip
    // wears when alive.
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
