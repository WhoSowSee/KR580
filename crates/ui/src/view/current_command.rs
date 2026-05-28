use iced::widget::{Space, column, container, row};
use iced::{Element, Length, alignment};
use k580_core::{Cpu8080State, decode_opcode};

use super::theme::{TOKYO_GREEN, TOKYO_MUTED, mono_text, ui_text};
use crate::app::Message;

const COMMAND_COLUMN_PORTIONS: [u16; 6] = [8, 10, 12, 10, 12, 13];
const COMMAND_GRID_SPACING: f32 = 10.0;
const TYPE_ADDRESS_COMPACT_GAP: f32 = 4.0;
const TYPE_ADDRESS_LONG_GAP: f32 = 6.0;
const TYPE_ADDRESS_LONG_TEXT_THRESHOLD: usize = 19;

#[derive(Debug, PartialEq, Eq)]
struct CurrentCommandFields {
    code: String,
    command: String,
    operand: String,
    length: String,
    kind: &'static str,
    addressing: &'static str,
}

pub(super) fn current_command_panel(cpu: &Cpu8080State) -> Element<'static, Message> {
    let fields = current_command_fields(cpu);

    row![
        command_column("Код", fields.code, command_column_width(0), true),
        command_column("Команда", fields.command, command_column_width(1), true),
        command_column("Операнд", fields.operand, command_column_width(2), true),
        command_column("Длина", fields.length, command_column_width(3), false),
        command_column("Тип", fields.kind, command_column_width(4), false),
        Space::new().width(Length::Fixed(type_address_gap(
            fields.kind,
            fields.addressing,
        ))),
        command_column(
            "Адресация",
            fields.addressing,
            command_column_width(5),
            false,
        ),
    ]
    .spacing(command_grid_spacing())
    .align_y(alignment::Vertical::Center)
    .width(Length::Fill)
    .into()
}

fn command_column(
    label: &'static str,
    value: impl Into<String>,
    width: Length,
    mono: bool,
) -> Element<'static, Message> {
    let align = command_column_alignment();
    let value = value.into();
    let value: Element<'static, Message> = if mono {
        mono_text(value, 14, TOKYO_GREEN).into()
    } else {
        ui_text(value, 14, TOKYO_GREEN).into()
    };

    container(
        column![ui_text(label, 12, TOKYO_MUTED), value]
            .spacing(5)
            .align_x(align),
    )
    .width(width)
    .align_x(align)
    .into()
}

fn command_column_alignment() -> alignment::Horizontal {
    alignment::Horizontal::Center
}

fn command_column_width(index: usize) -> Length {
    Length::FillPortion(command_column_portions()[index])
}

fn command_column_portions() -> [u16; 6] {
    COMMAND_COLUMN_PORTIONS
}

fn command_grid_spacing() -> f32 {
    COMMAND_GRID_SPACING
}

fn type_address_gap(kind: &str, addressing: &str) -> f32 {
    let text_len = kind.chars().count() + addressing.chars().count();
    if text_len >= TYPE_ADDRESS_LONG_TEXT_THRESHOLD {
        TYPE_ADDRESS_LONG_GAP
    } else {
        TYPE_ADDRESS_COMPACT_GAP
    }
}

fn current_command_fields(cpu: &Cpu8080State) -> CurrentCommandFields {
    let opcode = cpu.memory.read(cpu.pc);
    let code = format!("{opcode:02X}");
    let Ok(info) = decode_opcode(opcode) else {
        return CurrentCommandFields {
            code,
            command: "UNDOC".to_owned(),
            operand: "-".to_owned(),
            length: "1 байт".to_owned(),
            kind: "неизвестно",
            addressing: "неявная",
        };
    };

    let (command, operand) = split_instruction(&info.mnemonic);
    CurrentCommandFields {
        code,
        command: command.to_owned(),
        operand: operand.to_owned(),
        length: byte_length(info.size),
        kind: instruction_kind(command),
        addressing: addressing_kind(command, operand),
    }
}

fn split_instruction(mnemonic: &str) -> (&str, &str) {
    mnemonic.split_once(' ').unwrap_or((mnemonic, "-"))
}

fn byte_length(size: u8) -> String {
    match size {
        1 => "1 байт".to_owned(),
        2..=4 => format!("{size} байта"),
        _ => format!("{size} байт"),
    }
}

fn instruction_kind(command: &str) -> &'static str {
    match command {
        "NOP" | "HLT" | "EI" | "DI" => "управление",
        "JMP" | "JNZ" | "JZ" | "JNC" | "JC" | "JPO" | "JPE" | "JP" | "JM" | "CALL" | "CNZ"
        | "CZ" | "CNC" | "CC" | "CPO" | "CPE" | "CP" | "CM" | "RET" | "RNZ" | "RZ" | "RNC"
        | "RC" | "RPO" | "RPE" | "RP" | "RM" | "RST" | "PCHL" => "переход",
        "PUSH" | "POP" | "XTHL" | "SPHL" => "стек",
        "IN" | "OUT" => "ввод/вывод",
        "MOV" | "MVI" | "LXI" | "LDA" | "STA" | "LHLD" | "SHLD" | "LDAX" | "STAX" | "XCHG" => {
            "пересылка"
        }
        "ANA" | "XRA" | "ORA" | "CMP" | "ANI" | "XRI" | "ORI" | "CPI" | "RLC" | "RRC" | "RAL"
        | "RAR" | "CMA" | "STC" | "CMC" => "логика",
        _ => "арифметика",
    }
}

fn addressing_kind(command: &str, operand: &str) -> &'static str {
    if operand == "-" {
        return "неявная";
    }
    if operand.contains("d8") || operand.contains("d16") {
        return "непосредств";
    }
    if operand.contains("a16") {
        return "прямая";
    }
    if operand.contains('M') || matches!(command, "LDAX" | "STAX" | "PCHL" | "SPHL" | "XTHL") {
        return "косвенная";
    }
    "регистровая"
}

#[cfg(test)]
mod tests {
    use super::*;
    use k580_core::Cpu8080State;

    #[test]
    fn nop_current_command_matches_reference_panel() {
        let cpu = Cpu8080State::default();

        let fields = current_command_fields(&cpu);

        assert_eq!(fields.code, "00");
        assert_eq!(fields.command, "NOP");
        assert_eq!(fields.operand, "-");
        assert_eq!(fields.length, "1 байт");
        assert_eq!(fields.kind, "управление");
        assert_eq!(fields.addressing, "неявная");
    }

    #[test]
    fn immediate_command_extracts_operand_and_length() {
        let mut cpu = Cpu8080State::default();
        cpu.set_memory(cpu.pc, 0x06);

        let fields = current_command_fields(&cpu);

        assert_eq!(fields.code, "06");
        assert_eq!(fields.command, "MVI");
        assert_eq!(fields.operand, "B,d8");
        assert_eq!(fields.length, "2 байта");
        assert_eq!(fields.kind, "пересылка");
        assert_eq!(fields.addressing, "непосредств");
    }

    #[test]
    fn current_command_uses_opcode_at_pc_not_stale_instruction_register() {
        let mut cpu = Cpu8080State::default();
        cpu.last_fetched_opcode = 0x00;
        cpu.pc = 0x0000;
        cpu.set_memory(0x0000, 0x21);

        let fields = current_command_fields(&cpu);

        assert_eq!(fields.code, "21");
        assert_eq!(fields.command, "LXI");
        assert_eq!(fields.operand, "H,d16");
        assert_eq!(fields.length, "3 байта");
        assert_eq!(fields.kind, "пересылка");
        assert_eq!(fields.addressing, "непосредств");
    }

    #[test]
    fn command_columns_use_weighted_grid_for_long_text() {
        assert_eq!(command_column_alignment(), alignment::Horizontal::Center);
        assert_eq!(command_grid_spacing(), 10.0);
        assert_eq!(command_column_portions(), [8, 10, 12, 10, 12, 13]);
        assert_eq!(type_address_gap("пересылка", "непосредств"), 6.0);
        assert_eq!(type_address_gap("управление", "неявная"), 4.0);
    }
}
