//! Reusable chip helpers for the left schematic plate.

use iced::widget::{Space, button, column, container, mouse_area, row, svg, text_input, tooltip};
use iced::{Background, Color, Element, Length, Padding, Theme, alignment};
use k580_core::Cpu8080State;
use std::time::Duration;

use super::styles::inline_value_input_style;
use super::styles::{action_button_style, schematic_block_style};
use super::theme::{
    MONO_FONT, TOKYO_MUTED, TOKYO_RED, TOKYO_SELECTION_BLUE, TOKYO_SURFACE, TOKYO_TEXT, mono_text,
    ui_text,
};
use super::tooltips::{hover_tooltip, long_tooltip_body};
use crate::app::{Message, REGISTER_INLINE_INPUT_ID, RegisterInlineTarget};

fn wrap_tooltip(face: Element<'static, Message>, hint: &'static str) -> Element<'static, Message> {
    tooltip(face, long_tooltip_body(hint), tooltip::Position::Bottom)
        .gap(4.0)
        .padding(12.0)
        .delay(super::tooltips::EXPLANATORY_TOOLTIP_DELAY)
        .snap_within_viewport(true)
        .into()
}

const FUNCTIONAL_BLOCK_VALUE_WIDTH: f32 = 54.0;
const FUNCTIONAL_BLOCK_VALUE_HEIGHT: f32 = 28.0;
const FUNCTIONAL_BLOCK_INPUT_PADDING: Padding = Padding {
    top: 4.0,
    right: 0.0,
    bottom: 0.0,
    left: 0.0,
};
const SCHEMATIC_READOUT_WIDTH: f32 = 134.0;
const SCHEMATIC_READOUT_HEIGHT: f32 = 60.0;
const SCHEMATIC_READOUT_PADDING: f32 = 8.0;
pub(super) const SCHEMATIC_WIDE_READOUT_HEIGHT: f32 = SCHEMATIC_READOUT_HEIGHT;
const SCHEMATIC_READOUT_VALUE_FONT_SIZE: u32 = 20;
const SCHEMATIC_READOUT_VALUE_SLOT_HEIGHT: f32 = 24.0;
const SCHEMATIC_MNEMONIC_MIN_FONT_SIZE: u32 = 16;
const SCHEMATIC_MNEMONIC_WIDTH_FACTOR: f32 = 0.62;
const SCHEMATIC_MNEMONIC_INNER_WIDTH: f32 =
    SCHEMATIC_READOUT_WIDTH - SCHEMATIC_READOUT_PADDING * 2.0;

#[derive(Clone, Copy)]
pub(super) struct FunctionalBlockState {
    pub(super) selected: bool,
    pub(super) editing: bool,
    pub(super) hovered: bool,
}

/// 134×60 fits the longest Russian flag-register label at 11 px
/// alongside a 20 px monospace value, matching `functional_block`
/// footprint so they line up pixel-for-pixel in the same row.
pub(super) fn schematic_readout(
    label: impl Into<String>,
    value: impl Into<String>,
    accent: Color,
    tooltip_hint: Option<&'static str>,
) -> Element<'static, Message> {
    let face = container(
        column![
            ui_text(label, 11, TOKYO_MUTED),
            readout_value(value, SCHEMATIC_READOUT_VALUE_FONT_SIZE, accent),
        ]
        .spacing(2)
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Center),
    )
    .padding(SCHEMATIC_READOUT_PADDING)
    .width(Length::Fixed(SCHEMATIC_READOUT_WIDTH))
    .height(Length::Fixed(SCHEMATIC_READOUT_HEIGHT))
    .align_x(alignment::Horizontal::Center)
    .style(schematic_block_style)
    .into();

    match tooltip_hint {
        Some(hint) => wrap_tooltip(face, hint),
        None => face,
    }
}

pub(super) fn schematic_wide_readout(
    label: impl Into<String>,
    value: impl Into<String>,
    accent: Color,
    tooltip_hint: Option<&'static str>,
) -> Element<'static, Message> {
    let face = container(
        column![
            ui_text(label, 11, TOKYO_MUTED),
            readout_value(value, SCHEMATIC_READOUT_VALUE_FONT_SIZE, accent),
        ]
        .spacing(2)
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Center),
    )
    .padding(SCHEMATIC_READOUT_PADDING)
    .width(Length::Fill)
    .height(Length::Fixed(SCHEMATIC_WIDE_READOUT_HEIGHT))
    .align_x(alignment::Horizontal::Center)
    .style(schematic_block_style)
    .into();

    match tooltip_hint {
        Some(hint) => wrap_tooltip(face, hint),
        None => face,
    }
}

pub(super) fn schematic_mnemonic_readout(
    label: impl Into<String>,
    value: impl Into<String>,
    accent: Color,
    tooltip_hint: Option<&'static str>,
) -> Element<'static, Message> {
    let value = value.into();
    let value_size = schematic_mnemonic_font_size(&value);
    let face = container(
        column![
            ui_text(label, 11, TOKYO_MUTED),
            readout_value(value, value_size, accent),
        ]
        .spacing(2)
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Center),
    )
    .padding(SCHEMATIC_READOUT_PADDING)
    .width(Length::Fixed(SCHEMATIC_READOUT_WIDTH))
    .height(Length::Fixed(SCHEMATIC_READOUT_HEIGHT))
    .align_x(alignment::Horizontal::Center)
    .style(schematic_block_style)
    .into();

    match tooltip_hint {
        Some(hint) => wrap_tooltip(face, hint),
        None => face,
    }
}

fn readout_value(
    value: impl Into<String>,
    font_size: u32,
    accent: Color,
) -> Element<'static, Message> {
    container(
        mono_text(value, font_size, accent)
            .align_x(alignment::Horizontal::Center)
            .wrapping(iced::widget::text::Wrapping::None),
    )
    .width(Length::Fill)
    .height(Length::Fixed(SCHEMATIC_READOUT_VALUE_SLOT_HEIGHT))
    .align_x(alignment::Horizontal::Center)
    .align_y(alignment::Vertical::Center)
    .into()
}

fn schematic_mnemonic_font_size(value: &str) -> u32 {
    let glyph_count = value.chars().count();
    let visual_size = match glyph_count {
        0..=5 => SCHEMATIC_READOUT_VALUE_FONT_SIZE,
        6..=7 => 19,
        8..=9 => 18,
        10..=11 => 17,
        _ => SCHEMATIC_MNEMONIC_MIN_FONT_SIZE,
    };
    let fitted = (SCHEMATIC_MNEMONIC_INNER_WIDTH
        / (glyph_count.max(1) as f32 * SCHEMATIC_MNEMONIC_WIDTH_FACTOR))
        .floor() as u32;
    fitted.min(visual_size).clamp(
        SCHEMATIC_MNEMONIC_MIN_FONT_SIZE,
        SCHEMATIC_READOUT_VALUE_FONT_SIZE,
    )
}

pub(super) fn flag_strip(cpu: &Cpu8080State) -> Element<'static, Message> {
    const FLAG_GAP: f32 = 18.0;

    row![
        Space::new().width(Length::Fill),
        flag_dot("Z", cpu.flags.zero),
        Space::new().width(Length::Fixed(FLAG_GAP)),
        flag_dot("S", cpu.flags.sign),
        Space::new().width(Length::Fixed(FLAG_GAP)),
        flag_dot("P", cpu.flags.parity),
        Space::new().width(Length::Fixed(FLAG_GAP)),
        flag_dot("C", cpu.flags.carry),
        Space::new().width(Length::Fixed(FLAG_GAP)),
        flag_dot("AC", cpu.flags.auxiliary_carry),
        Space::new().width(Length::Fill),
    ]
    .width(Length::Fill)
    .align_y(alignment::Vertical::Center)
    .into()
}

fn flag_dot(label: &'static str, active: bool) -> Element<'static, Message> {
    column![
        mono_text("●", 18, if active { TOKYO_RED } else { TOKYO_TEXT })
            .align_x(alignment::Horizontal::Center),
        ui_text(label, 10, TOKYO_TEXT).align_x(alignment::Horizontal::Center),
    ]
    .spacing(2)
    .width(Length::Fixed(32.0))
    .into()
}

/// `on_press` is `None` for chips whose target window is not wired up
/// yet – the chip stays interactive (hover/tooltip) but its click is a
/// no-op so half-finished slots don't dispatch stale messages.
pub(super) fn device_chip(
    handle: svg::Handle,
    accent: Color,
    hint: &'static str,
    on_press: Option<Message>,
    shortcut: Option<&'static str>,
) -> Element<'static, Message> {
    const CHIP_WIDTH: f32 = 38.0;
    const CHIP_HEIGHT: f32 = 38.0;
    const GLYPH_SIZE: f32 = 20.0;

    let glyph = svg(handle)
        .width(Length::Fixed(GLYPH_SIZE))
        .height(Length::Fixed(GLYPH_SIZE))
        .style(move |_theme, _status| svg::Style {
            color: Some(accent),
        });

    let face = button(
        container(glyph)
            .padding(0)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center),
    )
    .on_press(on_press.unwrap_or(Message::MenuBatch(Vec::new())))
    .padding(0)
    .width(Length::Fixed(CHIP_WIDTH))
    .height(Length::Fixed(CHIP_HEIGHT))
    .style(|_theme, status| action_button_style(status));

    hover_tooltip(
        face.into(),
        hint,
        shortcut,
        iced::widget::tooltip::Position::Top,
        Duration::from_millis(650),
    )
}

pub(super) fn functional_block<'a>(
    title: &'static str,
    value: String,
    accent: Color,
    target: RegisterInlineTarget,
    state: FunctionalBlockState,
    input_value: &'a str,
    input_placeholder: &'a str,
) -> Element<'a, Message> {
    let value: Element<'_, Message> = if state.editing {
        container(
            text_input(input_placeholder, input_value)
                .id(REGISTER_INLINE_INPUT_ID)
                .on_input(move |value| Message::InlineRegisterValueChanged(target, value))
                .on_submit(Message::ApplyInlineRegisterValue(target))
                .font(MONO_FONT)
                .size(24)
                .padding(FUNCTIONAL_BLOCK_INPUT_PADDING)
                .align_x(alignment::Horizontal::Center)
                .width(Length::Fill)
                .style(inline_value_input_style),
        )
        .width(Length::Fixed(FUNCTIONAL_BLOCK_VALUE_WIDTH))
        .height(Length::Fixed(FUNCTIONAL_BLOCK_VALUE_HEIGHT))
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .into()
    } else {
        mouse_area(
            container(mono_text(value, 24, accent).align_x(alignment::Horizontal::Center))
                .width(Length::Fixed(FUNCTIONAL_BLOCK_VALUE_WIDTH))
                .height(Length::Fixed(FUNCTIONAL_BLOCK_VALUE_HEIGHT))
                .align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Center),
        )
        .on_press(Message::RegisterEnter(target))
        .on_double_click(Message::RegisterReplace(target))
        .interaction(iced::mouse::Interaction::Pointer)
        .into()
    };

    let face = container(
        column![ui_text(title, 11, TOKYO_MUTED), value,]
            .align_x(alignment::Horizontal::Center)
            .width(Length::Fill)
            .spacing(2),
    )
    .padding(8)
    .width(Length::Fixed(134.0))
    .height(Length::Fixed(60.0))
    .align_x(alignment::Horizontal::Center)
    .style(move |theme| {
        functional_block_style(theme, state.selected, state.hovered || state.editing)
    });

    let area = mouse_area(face)
        .on_enter(Message::RegisterHoverStarted(target))
        .on_exit(Message::RegisterHoverEnded(target))
        .interaction(iced::mouse::Interaction::Pointer);

    if state.editing {
        area.on_press(Message::RegisterEnter(target)).into()
    } else {
        area.on_press(Message::RegisterSelected(target))
            .on_double_click(Message::RegisterReplace(target))
            .into()
    }
}

fn functional_block_style(theme: &Theme, selected: bool, active: bool) -> container::Style {
    let mut style = schematic_block_style(theme);
    if selected {
        style.background = Some(Background::Color(TOKYO_SELECTION_BLUE));
    } else if active {
        style.background = Some(Background::Color(TOKYO_SURFACE));
    }
    style
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn functional_block_value_slot_keeps_edit_and_readout_metrics_stable() {
        assert!((FUNCTIONAL_BLOCK_VALUE_WIDTH - 54.0).abs() < f32::EPSILON);
        assert!((FUNCTIONAL_BLOCK_VALUE_HEIGHT - 28.0).abs() < f32::EPSILON);
        assert!((FUNCTIONAL_BLOCK_INPUT_PADDING.top - 4.0).abs() < f32::EPSILON);
        assert_eq!(FUNCTIONAL_BLOCK_INPUT_PADDING.bottom, 0.0);
    }

    #[test]
    fn wide_readout_keeps_same_vertical_metrics_as_standard_readout() {
        assert_eq!(SCHEMATIC_WIDE_READOUT_HEIGHT, SCHEMATIC_READOUT_HEIGHT);
    }

    #[test]
    fn mnemonic_readout_keeps_base_size_until_text_needs_shrink() {
        assert_eq!(
            schematic_mnemonic_font_size("NOP"),
            SCHEMATIC_READOUT_VALUE_FONT_SIZE
        );
        assert_eq!(
            schematic_mnemonic_font_size("INR A"),
            SCHEMATIC_READOUT_VALUE_FONT_SIZE
        );
        assert_eq!(
            schematic_mnemonic_font_size("SUB B"),
            SCHEMATIC_READOUT_VALUE_FONT_SIZE
        );
        assert_eq!(schematic_mnemonic_font_size("MVI D,d8"), 18);
        assert_eq!(schematic_mnemonic_font_size("LXI SP,d16"), 17);
    }

    #[test]
    fn mnemonic_readout_has_minimum_size_floor() {
        assert_eq!(
            schematic_mnemonic_font_size("0123456789ABCDEF"),
            SCHEMATIC_MNEMONIC_MIN_FONT_SIZE
        );
    }
}
