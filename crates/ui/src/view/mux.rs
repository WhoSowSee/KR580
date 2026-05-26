//! Мультиплексор panel — the right column of the schematic plate.
//!
//! Carries the W/Z scratch pair, the РОН register grid (B/C, D/E,
//! H/L), and the SP/PC footer. Lives in its own module because the
//! panel is ~270 lines on its own and `schematic.rs` was running over
//! the workspace's 400-line ceiling.
//!
//! Public surface is just `mux_panel(cpu, selected)` — the rest of
//! the helpers (`mux_section_caption`, `mux_static`, `mux_readout`,
//! `mux_register`) are private to this module.

use iced::widget::{Space, button, column, container, row};
use iced::{Element, Length, alignment};
use k580_core::{Cpu8080State, RegisterName, decode_opcode};

use super::styles::{
    mux_chip_style, mux_header_style, mux_panel_style, schematic_select_button_style,
};
use super::theme::{TOKYO_BLUE, TOKYO_GREEN, TOKYO_MUTED, mono_text, ui_text};
use crate::app::{Message, register_name};

/// Builds the "Мультиплексор" panel that mirrors the layout from the
/// reference KR-580 schematic the user is studying:
///
/// ```text
/// ┌─────────── Мультиплексор ───────────┐
/// │      Регистры временного хранения   │
/// │   W  00              Z  00          │
/// │   Регистры общего назначения (РОН)  │
/// │   B  00              C  00          │
/// │   D  00              E  00          │
/// │   H  00              L  00          │
/// │  ─────────────────────────────────  │
/// │   Указатель стека (УС)         FFFF │
/// │   Счётчик команд (СК)          0000 │
/// │   Схема инкремента-декремента    +1 │
/// └─────────────────────────────────────┘
/// ```
///
/// Key visual choices the user converged on over a few rounds:
/// 1. **Russian labels everywhere**: header is "Мультиплексор", the
///    two register groups carry the schoolbook 8080 captions
///    "Регистры временного хранения" / "Регистры общего назначения
///    (РОН)" — the user is teaching from a Russian-language reference,
///    so matching the captions verbatim is what makes the panel
///    readable as "the same diagram". The captions are centred
///    horizontally so the eye reads them as section titles rather
///    than left-flush list rows.
/// 2. **Inline name + value**: each register chip puts its name on
///    the left and its value on the right of the same row, instead
///    of stacking them vertically. Same reading rhythm as the
///    schematic notation, half the vertical footprint.
/// 3. **Outline-only chrome**: the panel and every chip inside it
///    are transparent in the resting state — the panel reads as a
///    bordered cut-out on the plate rather than a lifted card.
///    Borders carry the structure on their own.
/// 4. **A is gone from the РОН block**: the accumulator already has
///    its own dedicated chip in the status strip above the schematic
///    plate, so listing A here was duplicating the same readout in
///    two places. Click-target for selecting A still works through
///    the register editor's name input. The РОН grid now holds B/C,
///    D/E, H/L — three pairs in three rows, no orphan trailing
///    register.
/// 5. **SP and PC inline**: "Указатель стека (УС)" / "Счётчик команд
///    (СК)" each render as a single-row readout (label left, value
///    right) instead of label-above-value. Mirrors the rhythm of the
///    register chips above so the footer reads as a continuation of
///    the same column.
pub(super) fn mux_panel(cpu: &Cpu8080State, selected: RegisterName) -> Element<'static, Message> {
    container(
        column![
            // Header: centred 14 px text, same size as the legend
            // titles on the right-hand panels ("Содержимое ячеек
            // ОЗУ", "Ячейка ОЗУ и её значение"). The earlier 16 px
            // pass made the strip read as a much heavier title than
            // its siblings — the user wanted "Мультиплексор" to
            // visually rhyme with the other section headers, not
            // tower over them. Centring on both the inner text and
            // the wrapping container keeps the caption pinned to
            // the middle regardless of how iced rounds the inner
            // bounding box at this width. The header tone is
            // `TOKYO_MUTED` (same swatch the section captions use)
            // so the caption sits in the same grey weight class as
            // every other label on the schematic plate — the user
            // explicitly asked the white-bright variant to come
            // back to the same muted register the rest of the
            // chrome lives in.
            container(
                ui_text("Мультиплексор", 14, TOKYO_MUTED).align_x(alignment::Horizontal::Center),
            )
            .padding([4, 7])
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .style(mux_header_style),
            mux_section_caption("Регистры временного хранения"),
            // W and Z are the 8080's internal scratch pair. They are
            // programmatically invisible — no instruction can read or
            // write them directly — so we render them through
            // `mux_static`: same chrome as `mux_register`, but without
            // the `mouse_area`/`RegisterSelected` wiring (clicking
            // them would advertise an interaction the architecture
            // does not have).
            //
            // What they DO carry is a real value: the microsequencer
            // parks the address operand of `STA`/`LDA`/`LHLD`/`SHLD`/
            // `JMP*`/`CALL*`/`RET*`/`RST`/`XCHG`/`XTHL`/`PCHL`/`SPHL`/
            // `LXI` in W/Z on its way to the final destination. The
            // reference emulator we match against shows that residue,
            // and now we do too — `cpu.registers.w` / `.z` are
            // populated by the matching `set_wz` calls in `ops/data`
            // and `ops/control`. So after `STA 2000`, W reads `20`,
            // Z reads `00`, exactly like the school-grade emulator.
            row![
                mux_static("W", cpu.registers.w),
                mux_static("Z", cpu.registers.z),
            ]
            .spacing(0),
            mux_section_caption("Регистры общего назначения (РОН)"),
            // Three register pairs in three rows, no A. The user
            // explicitly removed A from this block because it lives
            // in the status strip's `compact_value("A", …)` slot
            // above the schematic plate; duplicating the readout
            // here would have it tracking the same byte from two
            // different chips. B/C / D/E / H/L are exactly the
            // three register pairs the chip exposes as 16-bit
            // operands (BC, DE, HL), which is also how every 8080
            // schoolbook diagram pairs them.
            row![
                mux_register(RegisterName::B, cpu.registers.b, selected),
                mux_register(RegisterName::C, cpu.registers.c, selected),
            ]
            .spacing(0),
            row![
                mux_register(RegisterName::D, cpu.registers.d, selected),
                mux_register(RegisterName::E, cpu.registers.e, selected),
            ]
            .spacing(0),
            row![
                mux_register(RegisterName::H, cpu.registers.h, selected),
                mux_register(RegisterName::L, cpu.registers.l, selected),
            ]
            .spacing(0),
            // SP and PC live below the register grid as a single
            // column of inline readouts. They are not "registers" in
            // the РОН sense — the user cannot click them to slot
            // into the register editor — so painting them as
            // `mux_readout` (label on the left, mono value on the
            // right) is what telegraphs the difference. Full Russian
            // names mirror the reference schematic; the parenthesised
            // abbreviations after them ("УС", "СК") are how the
            // textbook labels the same readouts on the bus diagram.
            mux_readout("Указатель стека (УС)", format!("{:04X}", cpu.sp)),
            mux_readout("Счётчик команд (СК)", format!("{:04X}", cpu.pc)),
            // "Схема инкремента-декремента" — the dedicated +1
            // increment block that the reference schematic paints
            // right under the SP/PC pair. On the real chip this is
            // the auxiliary adder that walks PC forward by the
            // length of the current instruction during the fetch
            // (and is also the path SP/HL go through for INX/DCX).
            // We surface the **PC step** here because that is what
            // the reference panel shows: the value reads `+1` for a
            // single-byte opcode, `+2` for opcodes with one operand
            // byte (`MVI r, d8`, `IN`, `OUT`, immediate ALU), `+3`
            // for those with a 16-bit operand (`LXI`, `JMP`,
            // `CALL`, `LDA`, `STA`, `LHLD`, `SHLD`). The byte at
            // `cpu.pc` is the next opcode about to be fetched, so
            // `decode_opcode(byte).size` is exactly the step the
            // increment circuit will commit on the next M1 cycle.
            // Undocumented bytes fall back to `+1` because that is
            // what the reference emulator paints for them too — it
            // never lets the readout go blank, and our
            // `decode_opcode` returning `Err` only happens on the
            // formally-undefined slots from `opcode_dispatch.md`.
            // Лейбл укорочен с «Схема инкремента-декремента» до
            // «Инкремент-декремент»: полная фраза не помещалась в
            // chip-ширину `FillPortion(1)` мультиплексора на дефолтном
            // окне — «декремента» вылезало за правую границу chip'а
            // прямо на value-колонку с `+N`. Слово «Схема» тут
            // избыточно: блок и так нарисован как chip в схематической
            // плите, читатель и так видит что это схема. Семантика та
            // же — это auxiliary adder, который шагает PC вперёд на
            // длину текущей инструкции (см. ниже про +1/+2/+3).
            mux_readout(
                "Инкремент-декремент",
                format!(
                    "+{}",
                    decode_opcode(cpu.memory.read(cpu.pc))
                        .map(|info| info.size)
                        .unwrap_or(1)
                ),
            ),
        ]
        .spacing(0),
    )
    .width(Length::FillPortion(1))
    .style(mux_panel_style)
    .into()
}

/// Section caption inside the multiplexer panel — paints the centred
/// muted-text divider that splits the chip group into "временного
/// хранения" / "РОН" / footer. Reuses `mux_header_style` so the strip
/// reads as a sibling of the panel's main header (same plate surface,
/// same hairline border) only with smaller, muted text. The
/// `align_x(Center)` is applied to both the inner `Text` and the
/// surrounding `container` so the caption sits centred regardless of
/// how iced rounds the inner-text bounding box against the outer
/// width — without the container-level alignment the centring drifts
/// a few pixels left at certain sizes.
fn mux_section_caption(label: &'static str) -> Element<'static, Message> {
    container(ui_text(label, 11, TOKYO_MUTED).align_x(alignment::Horizontal::Center))
        .padding([2, 8])
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .style(mux_header_style)
        .into()
}

/// Read-only chip used for the W/Z scratch pair. Same chrome as
/// `mux_register` (so the row visually matches the РОН rows
/// underneath it) but without the `mouse_area` wrapper or the
/// `RegisterSelected` press handler — W/Z are NOT programmer-visible
/// on the 8080. No 8080 instruction can read or write them directly,
/// so making them clickable would advertise an interaction the
/// architecture does not have.
///
/// They DO carry a real value: the microsequencer parks the address
/// operand of `STA`/`LDA`/`LHLD`/`SHLD`/`JMP*`/`CALL*`/`RET*`/`RST`/
/// `XCHG`/`XTHL`/`PCHL`/`SPHL`/`LXI` in W/Z on its way to the final
/// destination. The school-grade reference emulator we match against
/// shows that residue, and so do we — value text is `TOKYO_GREEN`
/// (same intensity as every other live numeric readout in the panel)
/// and the label drops to `TOKYO_MUTED` to keep telegraphing "this
/// row is special, you cannot click it".
fn mux_static(label: &'static str, value: u8) -> Element<'static, Message> {
    container(
        row![
            ui_text(label, 13, TOKYO_MUTED),
            Space::new().width(Length::Fill),
            mono_text(format!("{value:02X}"), 16, TOKYO_GREEN),
        ]
        .align_y(alignment::Vertical::Center)
        .spacing(8),
    )
    .padding([4, 10])
    .width(Length::Fill)
    .height(Length::Fixed(30.0))
    .style(mux_chip_style)
    .into()
}

/// Inline readout used for the SP / PC footer rows — same single-row
/// layout as `mux_static` (label left, mono value right) so the
/// footer reads as a continuation of the chip column above instead
/// of a stacked block with the value on a second line. Background
/// sticks to `mux_chip_style` for the same plate-coloured-with-border
/// look every chip in the panel wears.
fn mux_readout(label: &'static str, value: String) -> Element<'static, Message> {
    container(
        row![
            ui_text(label, 12, TOKYO_MUTED),
            Space::new().width(Length::Fill),
            mono_text(value, 16, TOKYO_GREEN),
        ]
        .align_y(alignment::Vertical::Center)
        .spacing(8),
    )
    .padding([4, 10])
    .width(Length::Fill)
    .style(mux_chip_style)
    .into()
}

fn mux_register(
    register: RegisterName,
    value: u8,
    selected: RegisterName,
) -> Element<'static, Message> {
    let is_selected = register == selected;

    // Register name colour mirrors the memory-row address column:
    // `TOKYO_BLUE` when the chip is the active selection,
    // `TOKYO_MUTED` otherwise. The user explicitly asked the
    // multiplexer to reuse the same idiom — "selected → blue,
    // unselected → grey" — so the eye does not have to learn a
    // separate visual language for the two panels. The byte stays
    // green (`TOKYO_GREEN`) at all times, same as the value column
    // on memory rows: it answers "what's stored here right now",
    // independent of which row holds the cursor.
    let label_color = if is_selected { TOKYO_BLUE } else { TOKYO_MUTED };

    // Inline layout: register name on the left, value pushed to the
    // right by a flexible spacer. Each chip occupies a single row in
    // the multiplexer panel — same reading rhythm as the reference
    // KR-580 schematic.
    //
    button(
        row![
            ui_text(register_name(register), 13, label_color),
            Space::new().width(Length::Fill),
            mono_text(format!("{value:02X}"), 16, TOKYO_GREEN),
        ]
        .align_y(alignment::Vertical::Center)
        .spacing(8)
        .width(Length::Fill),
    )
    .on_press(Message::RegisterSelected(register))
    .padding([4, 10])
    .width(Length::Fill)
    .height(Length::Fixed(30.0))
    .style(move |_theme, status| schematic_select_button_style(status, is_selected))
    .into()
}
