//! Left-hand "board" that imitates a CPU schematic with live readouts.
//!
//! Everything in this module is a pure function of the latest
//! `AppSnapshot`: status strip, registers, multiplexer, control lamps,
//! and the I/O device row. The multiplexer panel lives in `mux.rs` and
//! the lamp strip in `lamps.rs` so this file stays focused on the
//! framing logic that ties the panels together.

use iced::widget::{Space, column, container, mouse_area, row};
use iced::{Element, Length, Padding, alignment};
use k580_core::{RegisterName, decode_opcode};

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
use crate::app::{DesktopApp, Message, RegisterInlineTarget};
use crate::i18n::Key;

const MAIN_TO_BOTTOM_SPACING: f32 = 8.0;

impl DesktopApp {
    pub(super) fn schematic_panel(&self) -> Element<'_, Message> {
        let cpu = &self.snapshot.cpu;
        let lang = self.lang;

        let halt_indicator = mouse_area(mono_text(
            if cpu.halted {
                lang.t(Key::HltOn)
            } else {
                lang.t(Key::HltOff)
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
                    lang.t(Key::Accumulator),
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
                    lang.t(Key::BufferRegister1),
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
                    lang.t(Key::BufferRegister2),
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
                    lang.t(Key::AddressBuffer),
                    format!("{:04X}", cpu.last_address_bus),
                    TOKYO_GREEN,
                ),
                Space::new().width(Length::Fill),
                schematic_readout(
                    lang.t(Key::InstructionRegister),
                    format!("{:02X}", cpu.last_fetched_opcode),
                    TOKYO_GREEN,
                ),
                Space::new().width(Length::Fill),
                schematic_mnemonic_readout(
                    lang.t(Key::InstructionDecoder),
                    decode_opcode(cpu.last_fetched_opcode)
                        .map(|info| info.mnemonic)
                        .unwrap_or_else(|_| "-".to_owned()),
                    TOKYO_GREEN,
                ),
            ]
            .spacing(14),
        ]
        .spacing(14);

        let registers_panel = legend_panel_left(
            lang.t(Key::RegistersAndOperands),
            registers_grid,
            Length::Shrink,
        );
        let cycles = super::cycles::cycle_panels(cpu, lang);
        let signals_panel = legend_panel_left(
            lang.t(Key::ControlSignals),
            container(control_lamps(cpu))
                .width(Length::Fill)
                .align_x(alignment::Horizontal::Center),
            Length::Shrink,
        );
        let current_command_panel = legend_panel_left(
            lang.t(Key::CurrentCommand),
            super::current_command::current_command_panel(cpu, lang),
            Length::Shrink,
        );

        let left_board = column![
            psw_row,
            registers_panel,
            cycles,
            signals_panel,
            current_command_panel
        ]
        .spacing(12)
        .width(Length::Fixed(520.0));

        let status_register_block = super::status_register::status_register_tooltip(
            cpu,
            schematic_wide_readout(
                lang.t(Key::StatusRegister),
                super::status_register::status_register_bits(cpu),
                TOKYO_GREEN,
            ),
            lang,
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
        let (status_text, status_note) = split_legacy_status_note(&self.status, self.lang);
        let note_reservation_px = match status_note {
            Some(note) => 12.0 + (note.chars().count() + 2) as f32 * 7.5,
            None => 0.0,
        };
        let status_budget_px = (self.window_width - note_reservation_px).max(0.0);
        let shortened_status = crate::app::shorten_status_for_width(status_text, status_budget_px);
        let status_value: Element<'_, Message> = match status_note {
            Some(note) => row![
                mono_text(shortened_status, 13, TOKYO_TEXT)
                    .wrapping(iced::widget::text::Wrapping::None),
                ui_text(note, 12, TOKYO_MUTED).wrapping(iced::widget::text::Wrapping::None),
            ]
            .spacing(10)
            .align_y(alignment::Vertical::Center)
            .into(),
            None => mono_text(shortened_status, 13, TOKYO_TEXT)
                .wrapping(iced::widget::text::Wrapping::None)
                .into(),
        };
        let status_chip = row![
            ui_text(lang.t(Key::HeaderStatus), 12, TOKYO_MUTED),
            status_value,
        ]
        .spacing(12)
        .align_y(alignment::Vertical::Center);
        let header_row = row![
            status_row,
            container(status_chip)
                .width(Length::Fill)
                .align_x(alignment::Horizontal::Right)
                .clip(true),
        ]
        .spacing(20)
        .align_y(alignment::Vertical::Center)
        .width(Length::Fill);
        let central_column = column![
            schematic_wide_readout(
                lang.t(Key::DataBuffer),
                format!("{:02X}", cpu.last_data_bus_byte),
                TOKYO_GREEN,
            ),
            schematic_wide_readout(lang.t(Key::FlagsRegister), flag_bits, TOKYO_GREEN),
            super::mux::mux_panel(
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
                lang,
            ),
            status_register_block,
        ]
        .spacing(12)
        .width(Length::Fixed(240.0));

        let top = container(
            column![
                header_row,
                row![left_board, Space::new().width(Length::Fill), central_column]
                    .spacing(20)
                    .height(Length::Fill),
            ]
            .spacing(12)
            .height(Length::Fill),
        )
        .padding(12)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(schematic_block_style);

        let devices = row![
            device_chip(
                icons::device_monitor(),
                TOKYO_GREEN,
                lang.t(Key::DeviceMonitor),
                Some(Message::OpenMonitor),
            ),
            device_chip(
                icons::device_floppy(),
                TOKYO_CYAN,
                lang.t(Key::DeviceFloppy),
                Some(Message::OpenFloppy),
            ),
            device_chip(
                icons::device_hdd(),
                TOKYO_BLUE,
                lang.t(Key::DeviceHdd),
                None,
            ),
            device_chip(
                icons::device_network(),
                TOKYO_YELLOW,
                lang.t(Key::DeviceNetwork),
                None,
            ),
            device_chip(
                icons::device_printer(),
                TOKYO_MAGENTA,
                lang.t(Key::DevicePrinter),
                None,
            ),
        ]
        .spacing(14)
        .align_y(alignment::Vertical::Center);

        let quick_access = container(legend_panel_left(
            lang.t(Key::QuickAccess),
            container(devices)
                .width(Length::Fill)
                .align_x(alignment::Horizontal::Center),
            Length::Shrink,
        ))
        .width(Length::Fixed(290.0));
        let bottom = row![
            quick_access,
            Space::new().width(Length::Fill),
            super::speed::speed_panel(self.speed_tier, self.lang),
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

fn split_legacy_status_note(status: &str, lang: crate::i18n::Lang) -> (&str, Option<&'static str>) {
    let note = lang.t(crate::i18n::Key::LegacyFormatNote);
    let suffix = format!(" ({note})");
    match status.strip_suffix(&suffix) {
        Some(base) => (base, Some(note)),
        None => (status, None),
    }
}

#[cfg(test)]
mod tests {
    use super::split_legacy_status_note;
    use crate::i18n::Lang;

    #[test]
    fn legacy_status_suffix_renders_as_note_without_parentheses() {
        assert_eq!(
            split_legacy_status_note("Открыто C:\\test.580 (старый формат)", Lang::Ru),
            ("Открыто C:\\test.580", Some("старый формат"))
        );
        assert_eq!(
            split_legacy_status_note("Opened C:\\test.580 (legacy format)", Lang::En),
            ("Opened C:\\test.580", Some("legacy format"))
        );
    }

    #[test]
    fn regular_status_has_no_format_note() {
        assert_eq!(split_legacy_status_note("Готов", Lang::Ru), ("Готов", None));
        assert_eq!(split_legacy_status_note("Ready", Lang::En), ("Ready", None));
    }
}
