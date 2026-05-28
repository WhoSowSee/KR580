//! Small reusable chip helpers for the left schematic plate.
//!
//! Extracted from `view/schematic.rs` to keep that file under the
//! workspace's 400-line ceiling (see `AGENTS.md`). These are pure
//! widget builders: each returns an `Element<'static, Message>`,
//! takes only primitives or pre-tinted colours, and never reads
//! `DesktopApp` state. The boundary is "geometry / chrome of one
//! framed slot" — they are reused both inside `schematic.rs` and
//! by sibling modules that paint similar capsules on the plate.
//!
//! Public surface:
//! - `schematic_readout` — fixed-footprint label + 20 px hex value
//!   capsule (the "Регистр команд" / "Буфер данных" / "PSW" /
//!   "Регистр признаков" rows).
//! - `schematic_mnemonic_readout` — same chassis, 16 px value, used
//!   by the «Д/Ш команд» block where the value is a full mnemonic
//!   (`MVI M,d8`, `LXI SP,d16`, `JNZ a16`) instead of two hex
//!   digits.
//! - `flag_strip` / `flag_dot` — the Z/S/P/C/AC dot row that sits
//!   inside the top bus row.
//! - `device_chip` — peripheral chip on the bottom strip (tinted
//!   SVG glyph + tooltip).
//! - `functional_block` — clickable register chip (`Аккумулятор`,
//!   `Буферный регистр 1`, `Буферный регистр 2`).

use iced::widget::{Space, button, column, container, mouse_area, row, svg, text_input, tooltip};
use iced::{Background, Color, Element, Length, Padding, Theme, alignment};
use k580_core::Cpu8080State;
use std::time::Duration;

use super::styles::inline_value_input_style;
use super::styles::{action_button_style, inset_style, schematic_block_style};
use super::theme::{
    MONO_FONT, TOKYO_MUTED, TOKYO_RED, TOKYO_SELECTION_BLUE, TOKYO_SURFACE, TOKYO_TEXT, mono_text,
    ui_text,
};
use crate::app::{Message, REGISTER_INLINE_INPUT_ID, RegisterInlineTarget};

#[derive(Clone, Copy)]
pub(super) struct FunctionalBlockState {
    pub(super) selected: bool,
    pub(super) editing: bool,
    pub(super) hovered: bool,
}

pub(super) fn schematic_readout(
    label: impl Into<String>,
    value: impl Into<String>,
    accent: Color,
) -> Element<'static, Message> {
    // Width and height are fixed on purpose: with `Length::Fill` the row
    // stretches each readout to fill the schematic, so a 2-hex value floats
    // inside a half-panel-wide capsule. 134×60 fits the longest label we
    // render here («Регистр команд», «Буфер данных», «Регистр признаков»)
    // plus a 20 px monospace value, mirroring `functional_block`'s
    // footprint so the two helpers line up pixel-for-pixel when used in
    // the same row. Заголовок сжали с 12 до 11 px ровно по той же
    // причине, по которой расширили коробку — длиннее русские слова
    // («Регистр признаков» и сосед «Буферный регистр 1») едва влезали в
    // прежние 110 px при 12 px кеглем.
    container(
        column![
            ui_text(label, 11, TOKYO_MUTED),
            mono_text(value, 20, accent),
        ]
        .spacing(2)
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Center),
    )
    .padding(8)
    .width(Length::Fixed(134.0))
    .height(Length::Fixed(60.0))
    .align_x(alignment::Horizontal::Center)
    .style(schematic_block_style)
    .into()
}

pub(super) fn schematic_wide_readout(
    label: impl Into<String>,
    value: impl Into<String>,
    accent: Color,
) -> Element<'static, Message> {
    container(
        column![
            ui_text(label, 11, TOKYO_MUTED),
            mono_text(value, 20, accent),
        ]
        .spacing(2)
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Center),
    )
    .padding(8)
    .width(Length::Fill)
    .height(Length::Fixed(54.0))
    .align_x(alignment::Horizontal::Center)
    .style(schematic_block_style)
    .into()
}

/// Same chassis as `schematic_readout`, but with a 16 px monospace
/// value instead of 20 px so longer mnemonics fit. Used by the
/// «Д/Ш команд» block (instruction decoder), where the readout is
/// not a 2-hex byte but a full mnemonic with an operand —
/// `MVI M,d8`, `LXI SP,d16`, `JNZ a16` are all 7–10 characters,
/// and the 20 px font would push them past the 134 px capsule
/// width. The label and frame stay identical to `schematic_readout`
/// so the block visually rhymes with «Регистр команд» / «Регистр
/// флагов» — only the value column reads at a smaller size.
///
/// Кегль значения подняли с 14 → 16 px. На 14 px коротенькие
/// мнемоники вроде `NOP` (3 символа) выглядели заметно мельче
/// соседних значений при 20 px у `schematic_readout` («00» в РК,
/// в Буфере данных, в PSW), и пользователь это поймал на скрине.
/// 20 px напрямую поднять нельзя: самая длинная декодируемая
/// мнемоника — `LXI SP,d16` (10 символов) — на 20 px моноширинной
/// займёт ~120 px, а внутренняя ширина капсулы после padding'а
/// всего ~118 px, и текст вылезет за правую границу. 16 px — это
/// та же арифметика: 16 × 0.6 × 10 = 96 px, влезает с запасом, а
/// глаз уже не ловит ступеньку между «NOP» здесь и «00» рядом —
/// разница со соседним 20 px превратилась из «явно меньше» в
/// «чуть-чуть, но ритм один».
pub(super) fn schematic_mnemonic_readout(
    label: impl Into<String>,
    value: impl Into<String>,
    accent: Color,
) -> Element<'static, Message> {
    container(
        column![
            ui_text(label, 11, TOKYO_MUTED),
            mono_text(value, 16, accent),
        ]
        .spacing(2)
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Center),
    )
    .padding(8)
    .width(Length::Fixed(134.0))
    .height(Length::Fixed(60.0))
    .align_x(alignment::Horizontal::Center)
    .style(schematic_block_style)
    .into()
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

/// Single peripheral chip on the bottom row of the schematic plate.
/// Replaces the older two-line "MON / Ready" textual block with a tinted
/// SVG glyph inside the same neutral button chassis as the action chips, plus a
/// hover tooltip that reuses the editor `inset_style` so it visually
/// belongs to the same chrome family as the action-panel tooltips. The
/// chip intentionally dispatches an empty `MenuBatch`: it has no command
/// yet, but iced only exposes hover status for interactive buttons.
pub(super) fn device_chip(
    handle: svg::Handle,
    accent: Color,
    hint: &'static str,
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
    .on_press(Message::MenuBatch(Vec::new()))
    .padding(0)
    .width(Length::Fixed(CHIP_WIDTH))
    .height(Length::Fixed(CHIP_HEIGHT))
    .style(|_theme, status| action_button_style(status));

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

pub(super) fn functional_block<'a>(
    title: &'static str,
    value: String,
    accent: Color,
    target: RegisterInlineTarget,
    state: FunctionalBlockState,
    input_value: &'a str,
) -> Element<'a, Message> {
    // Same fixed footprint as `schematic_readout` so the two helpers visually
    // line up when used in the same row (Аккумулятор / Буферный регистр 1 /
    // Регистр признаков; Буферный регистр 2 / Регистр команд). 134×60 fits
    // the longest label rendered through this helper («Буферный регистр 1»
    // / «Буферный регистр 2» at 11 px) plus a 24 px monospace value with
    // breathing room. The inner column claims `Length::Fill` so the
    // centring directive actually has room to act on — without it the
    // column hugs the longest child and shorter labels slide left.
    //
    // The custom container style keeps the resting chip matched to the
    // outline-only readouts. Hover/editing only raise the fill tone while
    // the frame stays neutral.
    let value: Element<'_, Message> = if state.editing {
        container(
            text_input("00", input_value)
                .id(REGISTER_INLINE_INPUT_ID)
                .on_input(move |value| Message::InlineRegisterValueChanged(target, value))
                .on_submit(Message::ApplyInlineRegisterValue(target))
                .font(MONO_FONT)
                .size(24)
                .padding(0)
                .align_x(alignment::Horizontal::Center)
                .width(Length::Fill)
                .style(inline_value_input_style),
        )
        .width(Length::Fixed(54.0))
        .align_x(alignment::Horizontal::Center)
        .into()
    } else {
        mouse_area(
            container(mono_text(value, 24, accent).align_x(alignment::Horizontal::Center))
                .width(Length::Fixed(54.0))
                .align_x(alignment::Horizontal::Center),
        )
        .on_press(Message::RegisterEnter(target))
        .on_double_click(Message::RegisterEnter(target))
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
        area.into()
    } else {
        area.on_press(Message::RegisterSelected(target))
            .on_double_click(Message::RegisterEnter(target))
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
