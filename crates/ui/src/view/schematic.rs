//! Left-hand "board" that imitates a CPU schematic with live readouts.
//!
//! Everything in this module is a pure function of the latest
//! `AppSnapshot`: status strip, registers, multiplexer, control lamps,
//! and the I/O device row. The multiplexer panel lives in `mux.rs` and
//! the lamp strip in `lamps.rs` so this file stays focused on the
//! framing logic that ties the panels together.

use iced::widget::{Space, column, container, mouse_area, row};
use iced::{Element, Length, Padding, alignment};
use k580_core::{Cpu8080State, RegisterName, decode_opcode};

use super::chips::{
    FunctionalBlockState, device_chip, flag_strip, functional_block, schematic_mnemonic_readout,
    schematic_readout, schematic_wide_readout,
};
use super::icons;
use super::lamps::control_lamps;
use super::mux::MuxRegisterValues;
use super::styles::{schematic_block_style, schematic_board_style};
use super::theme::{
    TOKYO_BLUE, TOKYO_CYAN, TOKYO_GREEN, TOKYO_MAGENTA, TOKYO_MUTED, TOKYO_RED, TOKYO_TEXT,
    TOKYO_YELLOW, mono_text, ui_text,
};
use super::widgets::legend_panel_left;
use crate::app::{DesktopApp, Message, RegisterInlineTarget, SpeedTier};

const MAIN_TO_BOTTOM_SPACING: f32 = 8.0;

impl DesktopApp {
    pub(super) fn schematic_panel(&self) -> Element<'_, Message> {
        let cpu = &self.snapshot.cpu;

        let halt_indicator = mouse_area(mono_text(
            if cpu.halted {
                "HLT ВКЛ"
            } else {
                "HLT ВЫКЛ"
            },
            13,
            if cpu.halted { TOKYO_RED } else { TOKYO_GREEN },
        ))
        .on_press(Message::ToggleHalt)
        .interaction(iced::mouse::Interaction::Pointer);

        let status_row = row![
            mono_text(format!("PC {:04X}", cpu.pc), 13, TOKYO_BLUE),
            mono_text(format!("SP {:04X}", cpu.sp), 13, TOKYO_CYAN),
            mono_text(format!("T {}", cpu.cycle_count), 13, TOKYO_YELLOW),
            halt_indicator,
        ]
        .spacing(14)
        .align_y(alignment::Vertical::Center);

        let flag_panel = container(flag_strip(cpu))
            .padding([10, 20])
            .width(Length::Fill)
            .height(Length::Fixed(60.0))
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
            .style(schematic_block_style);

        let psw_row = row![
            schematic_readout(
                "PSW",
                format!(
                    "{:04X}",
                    ((cpu.registers.a as u16) << 8) | cpu.flags.to_psw() as u16,
                ),
                TOKYO_GREEN,
            ),
            flag_panel,
        ]
        .spacing(18)
        .align_y(alignment::Vertical::Center);

        let accumulator_target = RegisterInlineTarget::Schematic(RegisterName::A);
        let buffer1_target = RegisterInlineTarget::Schematic(RegisterName::B);
        let buffer2_target = RegisterInlineTarget::Schematic(RegisterName::C);
        let active_target = self.active_register_target;
        let inline_target = self.inline_register_target;
        let hovered_target = self.hovered_register_target;

        let registers_grid = column![
            row![
                functional_block(
                    "Аккумулятор",
                    self.display_register_value(RegisterName::A),
                    TOKYO_GREEN,
                    accumulator_target,
                    FunctionalBlockState {
                        selected: active_target == Some(accumulator_target),
                        editing: inline_target == Some(accumulator_target),
                        hovered: hovered_target == Some(accumulator_target),
                    },
                    &self.register_value_input,
                ),
                Space::new().width(Length::Fill),
                functional_block(
                    "Буферный регистр 1",
                    self.display_register_value(RegisterName::B),
                    TOKYO_GREEN,
                    buffer1_target,
                    FunctionalBlockState {
                        selected: active_target == Some(buffer1_target),
                        editing: inline_target == Some(buffer1_target),
                        hovered: hovered_target == Some(buffer1_target),
                    },
                    &self.register_value_input,
                ),
                Space::new().width(Length::Fill),
                functional_block(
                    "Буферный регистр 2",
                    self.display_register_value(RegisterName::C),
                    TOKYO_GREEN,
                    buffer2_target,
                    FunctionalBlockState {
                        selected: active_target == Some(buffer2_target),
                        editing: inline_target == Some(buffer2_target),
                        hovered: hovered_target == Some(buffer2_target),
                    },
                    &self.register_value_input,
                ),
            ]
            .spacing(14),
            row![
                schematic_readout(
                    "Буфер адреса",
                    format!("{:04X}", cpu.last_address_bus),
                    TOKYO_GREEN,
                ),
                Space::new().width(Length::Fill),
                schematic_readout(
                    "Регистр команд",
                    format!("{:02X}", cpu.last_fetched_opcode),
                    TOKYO_GREEN,
                ),
                Space::new().width(Length::Fill),
                schematic_mnemonic_readout(
                    "Д/Ш команд",
                    decode_opcode(cpu.last_fetched_opcode)
                        .map(|info| info.mnemonic)
                        .unwrap_or_else(|_| "-".to_owned()),
                    TOKYO_GREEN,
                ),
            ]
            .spacing(14),
        ]
        .spacing(14);

        let registers_panel =
            legend_panel_left("Регистры и операнды", registers_grid, Length::Shrink);
        let cycles = super::cycles::cycle_panels(cpu);
        let signals_panel = legend_panel_left(
            "Сигналы управления",
            container(control_lamps(cpu))
                .width(Length::Fill)
                .align_x(alignment::Horizontal::Center),
            Length::Shrink,
        );

        let left_board = column![status_row, psw_row, registers_panel, cycles, signals_panel]
            .spacing(12)
            .width(Length::Fixed(520.0));

        let status_register_block = super::status_register::status_register_tooltip(
            cpu,
            schematic_wide_readout(
                "Регистр состояния",
                super::status_register::status_register_bits(cpu),
                TOKYO_GREEN,
            ),
        );
        let flag_bits = format!(
            "{}{}{}{} {}{}{}{}",
            u8::from(cpu.flags.sign),
            u8::from(cpu.flags.zero),
            0,
            u8::from(cpu.flags.auxiliary_carry),
            0,
            u8::from(cpu.flags.parity),
            1,
            u8::from(cpu.flags.carry),
        );
        let status_chip = row![
            ui_text("Статус", 12, TOKYO_MUTED),
            mono_text(&self.status, 13, TOKYO_TEXT),
        ]
        .spacing(12)
        .align_y(alignment::Vertical::Center);
        let central_column = column![
            container(status_chip)
                .width(Length::Fill)
                .align_x(alignment::Horizontal::Right),
            schematic_wide_readout(
                "Буфер данных",
                format!("{:02X}", cpu.last_data_bus_byte),
                TOKYO_GREEN,
            ),
            schematic_wide_readout("Регистр признаков", flag_bits, TOKYO_GREEN),
            mux_panel(
                cpu,
                self.selected_register,
                self.inline_register_target,
                self.active_register_target,
                self.hovered_register_target,
                &self.register_value_input,
                MuxRegisterValues {
                    b: self.display_register_value(RegisterName::B),
                    c: self.display_register_value(RegisterName::C),
                    d: self.display_register_value(RegisterName::D),
                    e: self.display_register_value(RegisterName::E),
                    h: self.display_register_value(RegisterName::H),
                    l: self.display_register_value(RegisterName::L),
                },
            ),
            status_register_block,
        ]
        .spacing(12)
        .width(Length::Fixed(240.0));

        let top = container(
            row![left_board, Space::new().width(Length::Fill), central_column]
                .spacing(20)
                .height(Length::Fill),
        )
        .padding(12)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(schematic_block_style);

        let devices = row![
            device_chip(icons::device_monitor(), TOKYO_GREEN, "Отобразить монитор",),
            device_chip(
                icons::device_floppy(),
                TOKYO_CYAN,
                "Отобразить буфер дисковода",
            ),
            device_chip(
                icons::device_hdd(),
                TOKYO_BLUE,
                "Отобразить буфер жёсткого диска",
            ),
            device_chip(
                icons::device_network(),
                TOKYO_YELLOW,
                "Отобразить буфер сетевого адаптера",
            ),
            device_chip(
                icons::device_printer(),
                TOKYO_MAGENTA,
                "Отобразить буфер принтера",
            ),
        ]
        .spacing(14)
        .align_y(alignment::Vertical::Center);

        let quick_access = container(legend_panel_left(
            "Быстрый доступ",
            container(devices)
                .width(Length::Fill)
                .align_x(alignment::Horizontal::Center),
            Length::Shrink,
        ))
        .width(Length::Fixed(290.0));
        let bottom = row![
            quick_access,
            Space::new().width(Length::Fill),
            speed_panel(self.speed_tier),
        ]
        .spacing(24)
        .align_y(alignment::Vertical::Bottom);

        let content = column![top, bottom]
            .spacing(MAIN_TO_BOTTOM_SPACING)
            .height(Length::Fill);

        container(content)
            .padding(Padding {
                top: 4.0,
                right: 4.0,
                bottom: 0.0,
                left: 0.0,
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .style(schematic_board_style)
            .into()
    }
}

// ---------------------------------------------------------------------------
// schematic-specific helpers
// ---------------------------------------------------------------------------
//
// The pure widget builders that paint individual chips on this plate
// (`schematic_readout`, `schematic_mnemonic_readout`, `flag_strip`,
// `device_chip`, `functional_block`) live in `super::chips`. They were
// extracted to keep this file under the workspace's 400-line ceiling
// (see `AGENTS.md`) — call sites read `chips::schematic_readout(...)`
// the same way they used to read the local function.

/// Builds the "Мультиплексор" panel — implementation lives in
/// `super::mux` to keep this file under the workspace's 400-line
/// ceiling. Re-exported here as a one-liner so the call site reads
/// the same way it did before the split.
fn mux_panel<'a>(
    cpu: &Cpu8080State,
    selected: RegisterName,
    inline_target: Option<RegisterInlineTarget>,
    active_target: Option<RegisterInlineTarget>,
    hovered_target: Option<RegisterInlineTarget>,
    input_value: &'a str,
    values: MuxRegisterValues,
) -> Element<'a, Message> {
    super::mux::mux_panel(
        cpu,
        selected,
        inline_target,
        active_target,
        hovered_target,
        input_value,
        values,
    )
}

/// Four-tier speed switch — implementation lives in `super::speed`
/// to keep this file under the 400-line ceiling. Re-exported here as
/// a one-liner so the call site reads the same way it did before
/// the split.
fn speed_panel(active: SpeedTier) -> Element<'static, Message> {
    super::speed::speed_panel(active)
}
