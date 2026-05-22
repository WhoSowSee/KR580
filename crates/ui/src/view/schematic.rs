//! Left-hand "board" that imitates a CPU schematic with live readouts.
//!
//! Everything in this module is a pure function of the latest
//! `AppSnapshot`: status strip, registers, ALU block, multiplexer, control
//! lamps, buses, and the I/O device row.

use iced::widget::{Row, Space, button, column, container, row, slider};
use iced::{Color, Element, Length, alignment};
use k580_core::{Cpu8080State, RegisterName};

use super::styles::{
    alu_style, board_style, capsule_button_style, inset_style, mux_button_style, mux_header_style,
    mux_panel_style, schematic_block_style, solid_style,
};
use super::theme::{
    MONO_FONT, TOKYO_BLUE, TOKYO_CYAN, TOKYO_GREEN, TOKYO_MAGENTA, TOKYO_MUTED, TOKYO_RED,
    TOKYO_TEXT, TOKYO_YELLOW, mono_text, ui_text,
};
use crate::app::{DesktopApp, MAX_STEP_HZ, MIN_STEP_HZ, Message, register_name};

impl DesktopApp {
    pub(super) fn schematic_panel(&self) -> Element<'_, Message> {
        let cpu = &self.snapshot.cpu;

        let status_strip = row![
            mono_text(format!("PC {:04X}", cpu.pc), 13, TOKYO_BLUE),
            mono_text(format!("SP {:04X}", cpu.sp), 13, TOKYO_CYAN),
            mono_text(format!("T {}", cpu.cycle_count), 13, TOKYO_YELLOW),
            mono_text(
                if cpu.halted { "HALT ON" } else { "HALT OFF" },
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
            speed_panel(self.step_hz),
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
            .style(board_style)
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

fn mux_panel(cpu: &Cpu8080State, selected: RegisterName) -> Element<'static, Message> {
    let pair = |a: RegisterName, b: RegisterName| {
        row![
            mux_register(a, cpu.registers.get(a), selected),
            mux_register(b, cpu.registers.get(b), selected),
        ]
        .spacing(0)
    };

    container(
        column![
            container(ui_text("Multiplexer", 12, TOKYO_TEXT))
                .padding(7)
                .width(Length::Fill)
                .style(mux_header_style),
            pair(RegisterName::A, RegisterName::B),
            pair(RegisterName::C, RegisterName::D),
            pair(RegisterName::E, RegisterName::H),
            row![
                mux_register(RegisterName::L, cpu.registers.l, selected),
                compact_value("SP", format!("{:04X}", cpu.sp), TOKYO_GREEN),
            ]
            .spacing(0),
            compact_value("PC", format!("{:04X}", cpu.pc), TOKYO_GREEN),
        ]
        .spacing(0),
    )
    .width(Length::FillPortion(1))
    .height(Length::Fill)
    .style(mux_panel_style)
    .into()
}

fn mux_register(
    register: RegisterName,
    value: u8,
    selected: RegisterName,
) -> Element<'static, Message> {
    let is_selected = register == selected;
    let accent = if is_selected {
        TOKYO_MAGENTA
    } else {
        TOKYO_BLUE
    };

    button(column![
        ui_text(register_name(register), 11, TOKYO_BLUE),
        mono_text(format!("{value:02X}"), 16, TOKYO_GREEN),
    ])
    .on_press(Message::RegisterSelected(register))
    .padding(10)
    .width(Length::Fill)
    .height(Length::Fixed(58.0))
    .style(move |_theme, status| mux_button_style(status, accent, is_selected))
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

/// Speed slider for the paced `Run` loop. Lives in the lower-left
/// strip next to the Cycle/Tick panel: same vertical band, same
/// "control surface" semantics. Drag the slider, the worker thread
/// picks up the new instructions-per-second budget on the next
/// command tick.
///
/// Built as a fresh element on every redraw because slider state is
/// derived from `step_hz`, which lives on `DesktopApp`. iced's
/// `slider` is a thin wrapper around `f64` internally; the
/// conversion at both ends keeps the message type honest as `u32`.
fn speed_panel(step_hz: u32) -> Element<'static, Message> {
    let label = format!("Скорость: {step_hz} шаг/сек");
    container(
        column![
            ui_text(label, 12, TOKYO_MUTED),
            slider(MIN_STEP_HZ..=MAX_STEP_HZ, step_hz, Message::SpeedChanged).width(Length::Fill),
        ]
        .spacing(6),
    )
    .padding(10)
    .width(Length::Fixed(220.0))
    .style(schematic_block_style)
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
