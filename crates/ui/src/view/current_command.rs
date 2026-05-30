use iced::widget::{Space, column, container, row};
use iced::{Element, Length, alignment};
use k580_core::{Cpu8080State, decode_opcode};

use super::theme::{TOKYO_GREEN, TOKYO_MUTED, mono_text, ui_text};
use crate::app::Message;
use crate::i18n::{Key, Lang};

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
    kind: Key,
    addressing: Key,
}

pub(super) fn current_command_panel(cpu: &Cpu8080State, lang: Lang) -> Element<'_, Message> {
    let fields = current_command_fields(cpu, lang);

    row![
        command_column(
            lang.t(Key::ColCmdCode),
            fields.code,
            command_column_width(0),
            true
        ),
        command_column(
            lang.t(Key::ColCmdMnemonic),
            fields.command,
            command_column_width(1),
            true
        ),
        command_column(
            lang.t(Key::ColCmdOperand),
            fields.operand,
            command_column_width(2),
            true
        ),
        command_column(
            lang.t(Key::ColCmdLength),
            fields.length,
            command_column_width(3),
            false
        ),
        command_column(
            lang.t(Key::ColCmdKind),
            lang.t(fields.kind),
            command_column_width(4),
            false
        ),
        Space::new().width(Length::Fixed(type_address_gap(
            lang.t(fields.kind),
            lang.t(fields.addressing),
        ))),
        command_column(
            lang.t(Key::ColCmdAddressing),
            lang.t(fields.addressing),
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
    label: &str,
    value: impl Into<String>,
    width: Length,
    mono: bool,
) -> Element<'_, Message> {
    let align = command_column_alignment();
    let value = value.into();
    let value: Element<'_, Message> = if mono {
        mono_text(value, 14, TOKYO_GREEN).into()
    } else {
        ui_text(value, 14, TOKYO_GREEN).into()
    };

    container(
        column![ui_text(label.to_owned(), 12, TOKYO_MUTED), value]
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

fn current_command_fields(cpu: &Cpu8080State, lang: Lang) -> CurrentCommandFields {
    let opcode = cpu.memory.read(cpu.pc);
    let code = format!("{opcode:02X}");
    let Ok(info) = decode_opcode(opcode) else {
        return CurrentCommandFields {
            code,
            command: "UNDOC".to_owned(),
            operand: "-".to_owned(),
            length: byte_length(1, lang),
            kind: Key::CmdKindUnknown,
            addressing: Key::CmdAddrImplicit,
        };
    };

    let (command, operand) = split_instruction(&info.mnemonic);
    CurrentCommandFields {
        code,
        command: command.to_owned(),
        operand: operand.to_owned(),
        length: byte_length(info.size, lang),
        kind: instruction_kind_key(command),
        addressing: addressing_kind_key(command, operand),
    }
}

fn split_instruction(mnemonic: &str) -> (&str, &str) {
    mnemonic.split_once(' ').unwrap_or((mnemonic, "-"))
}

fn byte_length(size: u8, lang: Lang) -> String {
    match (size, lang) {
        (1, _) => lang.t(Key::CmdLengthByte).to_owned(),
        (2, _) => lang.t(Key::CmdLengthBytes2).to_owned(),
        (3, _) => lang.t(Key::CmdLengthBytes3).to_owned(),
        (n, Lang::Ru) => format!("{n} байт"),
        (n, Lang::En) => format!("{n} bytes"),
    }
}

fn instruction_kind_key(command: &str) -> Key {
    match command {
        "NOP" | "HLT" | "EI" | "DI" => Key::CmdKindControl,
        "JMP" | "JNZ" | "JZ" | "JNC" | "JC" | "JPO" | "JPE" | "JP" | "JM" | "CALL" | "CNZ"
        | "CZ" | "CNC" | "CC" | "CPO" | "CPE" | "CP" | "CM" | "RET" | "RNZ" | "RZ" | "RNC"
        | "RC" | "RPO" | "RPE" | "RP" | "RM" | "RST" | "PCHL" => Key::CmdKindBranch,
        "PUSH" | "POP" | "XTHL" | "SPHL" => Key::CmdKindStack,
        "IN" | "OUT" => Key::CmdKindIo,
        "MOV" | "MVI" | "LXI" | "LDA" | "STA" | "LHLD" | "SHLD" | "LDAX" | "STAX" | "XCHG" => {
            Key::CmdKindMove
        }
        "ANA" | "XRA" | "ORA" | "CMP" | "ANI" | "XRI" | "ORI" | "CPI" | "RLC" | "RRC" | "RAL"
        | "RAR" | "CMA" | "STC" | "CMC" => Key::CmdKindLogic,
        _ => Key::CmdKindArithmetic,
    }
}

fn addressing_kind_key(command: &str, operand: &str) -> Key {
    if operand == "-" {
        return Key::CmdAddrImplicit;
    }
    if operand.contains("d8") || operand.contains("d16") {
        return Key::CmdAddrImmediate;
    }
    if operand.contains("a16") {
        return Key::CmdAddrDirect;
    }
    if operand.contains('M') || matches!(command, "LDAX" | "STAX" | "PCHL" | "SPHL" | "XTHL") {
        return Key::CmdAddrIndirect;
    }
    Key::CmdAddrRegister
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::{Key, Lang};
    use k580_core::Cpu8080State;

    #[test]
    fn nop_current_command_matches_reference_panel() {
        let cpu = Cpu8080State::default();

        let fields = current_command_fields(&cpu, Lang::Ru);

        assert_eq!(fields.code, "00");
        assert_eq!(fields.command, "NOP");
        assert_eq!(fields.operand, "-");
        assert_eq!(fields.length, "1 байт");
        assert_eq!(fields.kind, Key::CmdKindControl);
        assert_eq!(fields.addressing, Key::CmdAddrImplicit);
    }

    #[test]
    fn immediate_command_extracts_operand_and_length() {
        let mut cpu = Cpu8080State::default();
        cpu.set_memory(cpu.pc, 0x06);

        let fields = current_command_fields(&cpu, Lang::Ru);

        assert_eq!(fields.code, "06");
        assert_eq!(fields.command, "MVI");
        assert_eq!(fields.operand, "B,d8");
        assert_eq!(fields.length, "2 байта");
        assert_eq!(fields.kind, Key::CmdKindMove);
        assert_eq!(fields.addressing, Key::CmdAddrImmediate);
    }

    #[test]
    fn current_command_uses_opcode_at_pc_not_stale_instruction_register() {
        let mut cpu = Cpu8080State::default();
        cpu.last_fetched_opcode = 0x00;
        cpu.pc = 0x0000;
        cpu.set_memory(0x0000, 0x21);

        let fields = current_command_fields(&cpu, Lang::Ru);

        assert_eq!(fields.code, "21");
        assert_eq!(fields.command, "LXI");
        assert_eq!(fields.operand, "H,d16");
        assert_eq!(fields.length, "3 байта");
        assert_eq!(fields.kind, Key::CmdKindMove);
        assert_eq!(fields.addressing, Key::CmdAddrImmediate);
    }

    #[test]
    fn english_command_renders_english_byte_length_and_kind() {
        let cpu = Cpu8080State::default();

        let fields = current_command_fields(&cpu, Lang::En);

        assert_eq!(fields.length, "1 byte");
        assert_eq!(Lang::En.t(fields.kind), "control");
        assert_eq!(Lang::En.t(fields.addressing), "implicit");
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
