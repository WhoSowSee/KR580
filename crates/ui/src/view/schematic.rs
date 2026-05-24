//! Left-hand "board" that imitates a CPU schematic with live readouts.
//!
//! Everything in this module is a pure function of the latest
//! `AppSnapshot`: status strip, registers, ALU block, multiplexer, control
//! lamps, buses, and the I/O device row.

use iced::widget::{Row, Space, button, column, container, mouse_area, row};
use iced::{Color, Element, Length, alignment};
use k580_core::{Cpu8080State, RegisterName};

use super::styles::{
    alu_style, capsule_button_style, inset_style, mux_button_style, mux_chip_style,
    mux_header_style, mux_panel_style, mux_register_chip_style, schematic_block_style,
    schematic_board_style, solid_style,
};
use super::theme::{
    MONO_FONT, TOKYO_BLUE, TOKYO_CYAN, TOKYO_GREEN, TOKYO_MAGENTA, TOKYO_MUTED, TOKYO_RED,
    TOKYO_TEXT, TOKYO_YELLOW, mono_text, ui_text,
};
use crate::app::{DesktopApp, Message, SpeedTier, register_name, tier_hz};

impl DesktopApp {
    pub(super) fn schematic_panel(&self) -> Element<'_, Message> {
        let cpu = &self.snapshot.cpu;

        let status_strip = row![
            mono_text(format!("PC {:04X}", cpu.pc), 13, TOKYO_BLUE),
            mono_text(format!("SP {:04X}", cpu.sp), 13, TOKYO_CYAN),
            mono_text(format!("T {}", cpu.cycle_count), 13, TOKYO_YELLOW),
            mono_text(
                // The label is `HLT` (the mnemonic of the Intel 8080
                // instruction that flips the halt flip-flop), not
                // `HALT` and not `HLDA`. `HALT` was a generic English
                // word for the internal state — fine in prose, but in
                // a strip of three-letter chip-style readouts next to
                // PC/SP/T it read as "name of a pin on the chip" and
                // the user pointed out the mismatch with both their
                // mental model and with `halt_notice`, which already
                // talks about "флаг HLT". `HLDA` is something else
                // entirely — that's the Hold Acknowledge pin (output
                // 21 on the 8080 corner), wired to the DMA arbiter
                // and unrelated to the halt flip-flop; it lives on
                // the control-lamp row at the bottom of the panel
                // where it actually belongs. Using `HLT` here keeps
                // the indicator, the halt-notice, and the register
                // editor all calling the same thing the same name.
                if cpu.halted { "HLT ON" } else { "HLT OFF" },
                13,
                if cpu.halted { TOKYO_RED } else { TOKYO_GREEN },
            ),
            Space::new().width(Length::Fill),
            ui_text("Статус", 12, TOKYO_MUTED),
            mono_text(&self.status, 13, TOKYO_TEXT),
        ]
        .spacing(14)
        .align_y(alignment::Vertical::Center);

        let top_bus_row = row![
            schematic_readout("PSW", format!("{:04X}", cpu.flags.to_psw()), TOKYO_GREEN),
            flag_strip(cpu),
            Space::new().width(Length::Fill),
            schematic_readout(
                "Data Buffer",
                format!("{:02X}", cpu.memory.read(cpu.pc)),
                TOKYO_GREEN,
            ),
        ]
        .spacing(18)
        .align_y(alignment::Vertical::Center);

        let alu = container(
            column![
                ui_text("ALU", 34, TOKYO_TEXT).font(MONO_FONT),
                row![
                    compact_value("A", format!("{:02X}", cpu.registers.a), TOKYO_GREEN),
                    compact_value("HL", format!("{:04X}", cpu.registers.hl()), TOKYO_CYAN),
                ]
                .spacing(8),
            ]
            .align_x(alignment::Horizontal::Center)
            .spacing(10),
        )
        .padding(12)
        .width(Length::Fill)
        .height(Length::Fixed(86.0))
        .style(alu_style);

        let core_left = column![
            row![
                functional_block(
                    "Accumulator",
                    format!("{:02X}", cpu.registers.a),
                    TOKYO_GREEN,
                    Message::RegisterSelected(RegisterName::A),
                ),
                functional_block(
                    "Buf. Reg 1",
                    format!("{:02X}", cpu.registers.b),
                    TOKYO_GREEN,
                    Message::RegisterSelected(RegisterName::B),
                ),
                schematic_readout(
                    "Reg. Flags",
                    format!("{:06b}", cpu.flags.to_psw()),
                    TOKYO_GREEN,
                ),
            ]
            .spacing(14),
            row![
                functional_block(
                    "Buf. Reg 2",
                    format!("{:02X}", cpu.registers.c),
                    TOKYO_GREEN,
                    Message::RegisterSelected(RegisterName::C),
                ),
                alu,
                schematic_readout(
                    "Instr. Reg",
                    format!("{:02X}", cpu.memory.read(cpu.pc)),
                    TOKYO_GREEN,
                ),
            ]
            .spacing(14),
        ]
        .spacing(14)
        .width(Length::FillPortion(3));

        let multiplexer = mux_panel(cpu, self.selected_register);

        let core_plane = row![core_left, multiplexer]
            .spacing(16)
            .height(Length::FillPortion(2));

        let low_control = row![
            cycle_tick_panel(cpu),
            Space::new().width(Length::Fill),
            speed_panel(self.speed_tier),
            Space::new().width(Length::Fill),
            schematic_readout("Sync & Control Block", "CTRL", TOKYO_TEXT),
        ]
        .spacing(20)
        .align_y(alignment::Vertical::Center);

        let devices_state = &self.snapshot.devices;
        let device_entries: [(&'static str, &dyn std::fmt::Debug); 5] = [
            ("MON", &devices_state.monitor.status),
            ("FDD", &devices_state.floppy.status),
            ("HDD", &devices_state.hdd.status),
            ("NET", &devices_state.network.status),
            ("PRN", &devices_state.printer.status),
        ];

        let devices = Row::with_children(
            device_entries
                .iter()
                .map(|(label, status)| square_device(label, &format!("{status:?}")))
                .chain(std::iter::once(Space::new().width(Length::Fill).into()))
                .chain(std::iter::once(schematic_readout(
                    "I/O Controller",
                    "I/O",
                    TOKYO_TEXT,
                ))),
        )
        .spacing(10)
        .align_y(alignment::Vertical::Center);

        let bottom = column![
            low_control,
            control_lamps(),
            bus_bar("Address Bus A0-A15", TOKYO_MUTED),
            bus_bar("Control Bus", TOKYO_MUTED),
            devices,
        ]
        .spacing(10);

        let content = column![
            status_strip,
            bus_bar("External Data Bus D7-D0", TOKYO_MUTED),
            top_bus_row,
            bus_bar("Internal Data Bus", TOKYO_MUTED),
            core_plane,
            bottom,
        ]
        .spacing(16)
        .height(Length::Fill);

        container(content)
            .padding(18)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(schematic_board_style)
            .into()
    }
}

// ---------------------------------------------------------------------------
// schematic blocks
// ---------------------------------------------------------------------------

fn schematic_readout(
    label: impl Into<String>,
    value: impl Into<String>,
    accent: Color,
) -> Element<'static, Message> {
    container(
        column![
            ui_text(label, 12, TOKYO_MUTED),
            mono_text(value, 20, accent),
        ]
        .spacing(4)
        .align_x(alignment::Horizontal::Center),
    )
    .padding(10)
    .width(Length::Fill)
    .height(Length::Fixed(70.0))
    .style(schematic_block_style)
    .into()
}

fn compact_value(
    label: impl Into<String>,
    value: impl Into<String>,
    accent: Color,
) -> Element<'static, Message> {
    container(
        column![
            ui_text(label, 11, TOKYO_MUTED),
            mono_text(value, 18, accent),
        ]
        .spacing(2),
    )
    .padding(8)
    .width(Length::Fill)
    .style(inset_style)
    .into()
}

fn flag_strip(cpu: &Cpu8080State) -> Element<'static, Message> {
    let dots = [
        ("Z", cpu.flags.zero),
        ("S", cpu.flags.sign),
        ("P", cpu.flags.parity),
        ("C", cpu.flags.carry),
        ("AC", cpu.flags.auxiliary_carry),
    ];

    Row::with_children(
        dots.into_iter()
            .map(|(label, active)| flag_dot(label, active)),
    )
    .spacing(8)
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
    .width(Length::Fixed(28.0))
    .into()
}

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
/// 3. **Plate-coloured chrome with borders only**: the panel and
///    every chip inside it use `TOKYO_BOARD` (the schematic plate
///    fill) — the panel reads as a bordered cut-out on the plate
///    rather than a lifted card. Borders carry the structure on
///    their own.
/// 4. **A is gone from the РОН block**: the accumulator already has
///    its own dedicated chip in the status strip above the schematic
///    plate (line 74 in this file: `compact_value("A", …)`),
///    so listing A here was duplicating the same readout in two
///    places. Click-target for selecting A still works through the
///    register editor's name input. The РОН grid now holds B/C, D/E,
///    H/L — three pairs in three rows, no orphan trailing register.
/// 5. **SP and PC inline**: "Указатель стека (УС)" / "Счётчик команд
///    (СК)" each render as a single-row readout (label left, value
///    right) instead of label-above-value. Mirrors the rhythm of the
///    register chips above so the footer reads as a continuation of
///    the same column.
fn mux_panel(cpu: &Cpu8080State, selected: RegisterName) -> Element<'static, Message> {
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
            // bounding box at this width.
            container(
                ui_text("Мультиплексор", 14, TOKYO_TEXT)
                    .align_x(alignment::Horizontal::Center),
            )
            .padding([8, 7])
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
/// reads as a sibling of the panel's main header (same `TOKYO_BOARD`
/// surface, same hairline border) only with smaller, muted text. The
/// `align_x(Center)` is applied to both the inner `Text` and the
/// surrounding `container` so the caption sits centred regardless of
/// how iced rounds the inner-text bounding box against the outer
/// width — without the container-level alignment the centring drifts
/// a few pixels left at certain sizes.
fn mux_section_caption(label: &'static str) -> Element<'static, Message> {
    container(ui_text(label, 11, TOKYO_MUTED).align_x(alignment::Horizontal::Center))
        .padding([4, 8])
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
    .padding(10)
    .width(Length::Fill)
    .height(Length::Fixed(38.0))
    .style(mux_chip_style)
    .into()
}

/// Inline readout used for the SP / PC footer rows — same single-row
/// layout as `mux_static` (label left, mono value right) so the
/// footer reads as a continuation of the chip column above instead
/// of a stacked block with the value on a second line. We don't
/// reuse `compact_value` here because that helper is the
/// vertical-stack variant the status strip relies on: changing it
/// would also flip A and HL upstairs into a horizontal layout the
/// user did not ask for. Background sticks to `mux_chip_style` for
/// the same plate-coloured-with-border look every chip in the panel
/// wears.
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
    .padding(10)
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
    let label_color = if is_selected {
        TOKYO_BLUE
    } else {
        TOKYO_MUTED
    };

    // Inline layout: register name on the left, value pushed to the
    // right by a flexible spacer. Replaces the previous vertical
    // stack (name above, value below) so each chip occupies a single
    // row in the multiplexer panel — same reading rhythm as the
    // reference KR-580 schematic. Height drops from 58 px (which the
    // old vertical layout needed for two text lines plus padding) to
    // 38 px so the panel still fits the W/Z + РОН + SP/PC stack
    // without growing taller than the schematic plate it sits on.
    //
    // The chip is now a `mouse_area`-wrapped `container`, not a
    // `button`. iced's `button::on_press` only fires on
    // `Status::Released`, so between mouse-down and mouse-up the
    // user perceives the gap between the click and the highlight
    // moving as input lag. `mouse_area::on_press` fires on the
    // press edge (matching `address_cell` / `command_cell` in the
    // memory list, which are also `mouse_area`-driven), so the
    // selection visibly snaps in the same frame the click lands.
    // The `Pointer` interaction hint replaces the cursor change
    // that `button` did for free, keeping the affordance honest.
    let body = container(
        row![
            ui_text(register_name(register), 13, label_color),
            Space::new().width(Length::Fill),
            mono_text(format!("{value:02X}"), 16, TOKYO_GREEN),
        ]
        .align_y(alignment::Vertical::Center)
        .spacing(8),
    )
    .padding(10)
    .width(Length::Fill)
    .height(Length::Fixed(38.0))
    .style(move |_theme| mux_register_chip_style(is_selected));

    mouse_area(body)
        .on_press(Message::RegisterSelected(register))
        .interaction(iced::mouse::Interaction::Pointer)
        .into()
}

fn cycle_tick_panel(cpu: &Cpu8080State) -> Element<'static, Message> {
    container(
        column![
            row![
                ui_text("Cycle", 12, TOKYO_MUTED),
                mono_text(cpu.cycle_count.to_string(), 14, TOKYO_GREEN),
            ]
            .spacing(10),
            row![
                ui_text("Tick", 12, TOKYO_MUTED),
                mono_text(
                    cpu.tact_phase
                        .map(|phase| phase.to_string())
                        .unwrap_or_else(|| "1".to_owned()),
                    14,
                    TOKYO_GREEN,
                ),
            ]
            .spacing(18),
        ]
        .spacing(6),
    )
    .padding(10)
    .style(schematic_block_style)
    .into()
}

/// Four-tier speed switch for the paced `Run` loop. Lives in the
/// lower-left strip next to the Cycle/Tick panel: same vertical band,
/// same "control surface" semantics. Replaces the freeform slider —
/// past the monitor refresh rate the slider only made the program
/// finish faster, never visibly faster, so a continuous control was
/// inviting the user to chase a sweet spot that didn't exist. Named
/// tiers communicate intent honestly:
///
/// - "Медленно" — `SLOW_TIER_HZ` (5 Hz), one instruction every
///   200 ms. The pace at which a human can read every memory row as
///   PC walks across it.
/// - "Средне" — `MEDIUM_TIER_HZ` (20 Hz), the default. Visibly "the
///   program is running" while the eye still keeps up.
/// - "Высоко" — primary monitor's refresh rate (with a 60 Hz
///   fallback). One instruction per frame; finishes as fast as the
///   screen can paint without skipping rows.
/// - "Максимум" — `MAX_TIER_HZ` (1000 Hz), uncoupled from the
///   monitor. The worker churns at its hard ceiling
///   (`MIN_STEP_INTERVAL = 1ms`) and the highlighted row jumps
///   instead of walking. The opt-in for "просто доведи программу до
///   конца, мне не нужно смотреть на каждый шаг".
///
/// The active tier is highlighted with the same `mux_button_style`
/// the multiplexer panel uses for its own selected/unselected
/// distinction, so the switch reads as part of the schematic's
/// control surface rather than a foreign widget.
fn speed_panel(active: SpeedTier) -> Element<'static, Message> {
    let caption = format!("Скорость: {} шаг/сек", tier_hz(active));

    let switch = row![
        tier_button("Медленно", SpeedTier::Slow, active),
        tier_button("Средне", SpeedTier::Medium, active),
        tier_button("Высоко", SpeedTier::High, active),
        tier_button("Максимум", SpeedTier::Max, active),
    ]
    .spacing(4);

    container(
        column![ui_text(caption, 12, TOKYO_MUTED), switch].spacing(6),
    )
    .padding(10)
    .width(Length::Fixed(340.0))
    .style(schematic_block_style)
    .into()
}

fn tier_button(
    label: &'static str,
    tier: SpeedTier,
    active: SpeedTier,
) -> Element<'static, Message> {
    let is_selected = tier == active;
    let accent = if is_selected {
        TOKYO_MAGENTA
    } else {
        TOKYO_BLUE
    };

    button(
        ui_text(label, 11, TOKYO_TEXT)
            .align_x(alignment::Horizontal::Center)
            .width(Length::Fill),
    )
    .on_press(Message::SpeedTierChanged(tier))
    .padding(6)
    .width(Length::Fill)
    .style(move |_theme, status| mux_button_style(status, accent, is_selected))
    .into()
}

fn square_device(label: &'static str, value: &str) -> Element<'static, Message> {
    container(
        column![
            mono_text(label, 12, TOKYO_TEXT),
            ui_text(value.to_owned(), 10, TOKYO_MUTED),
        ]
        .align_x(alignment::Horizontal::Center)
        .spacing(2),
    )
    .padding(7)
    .width(Length::Fixed(52.0))
    .height(Length::Fixed(44.0))
    .style(schematic_block_style)
    .into()
}

fn control_lamps() -> Element<'static, Message> {
    const LAMPS: [&str; 11] = [
        "F2", "F1", "SYNC", "READY", "WAIT", "HOLD", "INT", "INTE", "DBIN", "WR", "HLDA",
    ];

    Row::with_children(LAMPS.into_iter().map(control_lamp))
        .spacing(7)
        .align_y(alignment::Vertical::Center)
        .into()
}

fn control_lamp(label: &'static str) -> Element<'static, Message> {
    column![
        ui_text(label, 8, TOKYO_MUTED).align_x(alignment::Horizontal::Center),
        mono_text("●", 14, TOKYO_RED).align_x(alignment::Horizontal::Center),
    ]
    .width(Length::Fixed(34.0))
    .spacing(2)
    .into()
}

fn functional_block(
    title: impl Into<String>,
    value: impl Into<String>,
    accent: Color,
    message: Message,
) -> Element<'static, Message> {
    button(
        column![
            ui_text(title, 12, TOKYO_MUTED),
            mono_text(value, 24, accent),
        ]
        .align_x(alignment::Horizontal::Center)
        .spacing(4),
    )
    .on_press(message)
    .padding(10)
    .width(Length::Fill)
    .style(move |_theme, status| capsule_button_style(status, accent, false))
    .into()
}

fn bus_bar(label: impl Into<String>, accent: Color) -> Element<'static, Message> {
    row![
        ui_text(label, 12, TOKYO_MUTED),
        container(Space::new())
            .height(Length::Fixed(4.0))
            .width(Length::Fill)
            .style(move |_theme| solid_style(accent, 0.0)),
    ]
    .spacing(10)
    .align_y(alignment::Vertical::Center)
    .width(Length::Fill)
    .into()
}
