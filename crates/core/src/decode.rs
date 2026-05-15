use crate::{DecodeError, InstructionTiming};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstructionInfo {
    pub opcode: u8,
    pub mnemonic: String,
    pub size: u8,
    pub timing: InstructionTiming,
}

const REGS: [&str; 8] = ["B", "C", "D", "E", "H", "L", "M", "A"];
const RPS: [&str; 4] = ["B", "D", "H", "SP"];
const CONDS: [&str; 8] = ["NZ", "Z", "NC", "C", "PO", "PE", "P", "M"];

pub const fn is_undocumented_opcode(opcode: u8) -> bool {
    matches!(
        opcode,
        0x08 | 0x10 | 0x18 | 0x20 | 0x28 | 0x30 | 0x38 | 0xCB | 0xD9 | 0xDD | 0xED | 0xFD
    )
}

pub fn decode_opcode(opcode: u8) -> Result<InstructionInfo, DecodeError> {
    if is_undocumented_opcode(opcode) {
        return Err(DecodeError::UndocumentedOpcode(opcode));
    }

    if (0x40..=0x7F).contains(&opcode) {
        return Ok(if opcode == 0x76 {
            info(opcode, "HLT", 1, InstructionTiming::fixed(7))
        } else {
            let dst = ((opcode >> 3) & 7) as usize;
            let src = (opcode & 7) as usize;
            let cycles = if dst == 6 || src == 6 { 7 } else { 5 };
            info(
                opcode,
                format!("MOV {},{}", REGS[dst], REGS[src]),
                1,
                InstructionTiming::fixed(cycles),
            )
        });
    }

    if (0x80..=0xBF).contains(&opcode) {
        let src = (opcode & 7) as usize;
        let cycles = if src == 6 { 7 } else { 4 };
        let family = match (opcode >> 3) & 7 {
            0 => "ADD",
            1 => "ADC",
            2 => "SUB",
            3 => "SBB",
            4 => "ANA",
            5 => "XRA",
            6 => "ORA",
            _ => "CMP",
        };
        return Ok(info(
            opcode,
            format!("{} {}", family, REGS[src]),
            1,
            InstructionTiming::fixed(cycles),
        ));
    }

    if opcode & 0xC7 == 0x04 {
        let reg = ((opcode >> 3) & 7) as usize;
        return Ok(info(
            opcode,
            format!("INR {}", REGS[reg]),
            1,
            InstructionTiming::fixed(if reg == 6 { 10 } else { 5 }),
        ));
    }
    if opcode & 0xC7 == 0x05 {
        let reg = ((opcode >> 3) & 7) as usize;
        return Ok(info(
            opcode,
            format!("DCR {}", REGS[reg]),
            1,
            InstructionTiming::fixed(if reg == 6 { 10 } else { 5 }),
        ));
    }
    if opcode & 0xC7 == 0x06 {
        let reg = ((opcode >> 3) & 7) as usize;
        return Ok(info(
            opcode,
            format!("MVI {},d8", REGS[reg]),
            2,
            InstructionTiming::fixed(if reg == 6 { 10 } else { 7 }),
        ));
    }
    if opcode & 0xCF == 0x01 {
        let rp = ((opcode >> 4) & 3) as usize;
        return Ok(info(
            opcode,
            format!("LXI {},d16", RPS[rp]),
            3,
            InstructionTiming::fixed(10),
        ));
    }
    if opcode & 0xCF == 0x03 {
        let rp = ((opcode >> 4) & 3) as usize;
        return Ok(info(
            opcode,
            format!("INX {}", RPS[rp]),
            1,
            InstructionTiming::fixed(5),
        ));
    }
    if opcode & 0xCF == 0x09 {
        let rp = ((opcode >> 4) & 3) as usize;
        return Ok(info(
            opcode,
            format!("DAD {}", RPS[rp]),
            1,
            InstructionTiming::fixed(10),
        ));
    }
    if opcode & 0xCF == 0x0B {
        let rp = ((opcode >> 4) & 3) as usize;
        return Ok(info(
            opcode,
            format!("DCX {}", RPS[rp]),
            1,
            InstructionTiming::fixed(5),
        ));
    }
    if opcode & 0xC7 == 0xC0 {
        let cond = ((opcode >> 3) & 7) as usize;
        return Ok(info(
            opcode,
            format!("R{}", CONDS[cond]),
            1,
            InstructionTiming::conditional(11, 5),
        ));
    }
    if opcode & 0xC7 == 0xC2 {
        let cond = ((opcode >> 3) & 7) as usize;
        return Ok(info(
            opcode,
            format!("J{} a16", CONDS[cond]),
            3,
            InstructionTiming::conditional(10, 10),
        ));
    }
    if opcode & 0xC7 == 0xC4 {
        let cond = ((opcode >> 3) & 7) as usize;
        return Ok(info(
            opcode,
            format!("C{} a16", CONDS[cond]),
            3,
            InstructionTiming::conditional(17, 11),
        ));
    }
    if opcode & 0xC7 == 0xC7 {
        let n = (opcode >> 3) & 7;
        return Ok(info(
            opcode,
            format!("RST {}", n),
            1,
            InstructionTiming::fixed(11),
        ));
    }
    if opcode & 0xCF == 0xC1 {
        return Ok(info(
            opcode,
            format!("POP {}", stack_pair(opcode)),
            1,
            InstructionTiming::fixed(10),
        ));
    }
    if opcode & 0xCF == 0xC5 {
        return Ok(info(
            opcode,
            format!("PUSH {}", stack_pair(opcode)),
            1,
            InstructionTiming::fixed(11),
        ));
    }

    Ok(match opcode {
        0x00 => info(opcode, "NOP", 1, InstructionTiming::fixed(4)),
        0x02 => info(opcode, "STAX B", 1, InstructionTiming::fixed(7)),
        0x07 => info(opcode, "RLC", 1, InstructionTiming::fixed(4)),
        0x0A => info(opcode, "LDAX B", 1, InstructionTiming::fixed(7)),
        0x0F => info(opcode, "RRC", 1, InstructionTiming::fixed(4)),
        0x12 => info(opcode, "STAX D", 1, InstructionTiming::fixed(7)),
        0x17 => info(opcode, "RAL", 1, InstructionTiming::fixed(4)),
        0x1A => info(opcode, "LDAX D", 1, InstructionTiming::fixed(7)),
        0x1F => info(opcode, "RAR", 1, InstructionTiming::fixed(4)),
        0x22 => info(opcode, "SHLD a16", 3, InstructionTiming::fixed(16)),
        0x27 => info(opcode, "DAA", 1, InstructionTiming::fixed(4)),
        0x2A => info(opcode, "LHLD a16", 3, InstructionTiming::fixed(16)),
        0x2F => info(opcode, "CMA", 1, InstructionTiming::fixed(4)),
        0x32 => info(opcode, "STA a16", 3, InstructionTiming::fixed(13)),
        0x37 => info(opcode, "STC", 1, InstructionTiming::fixed(4)),
        0x3A => info(opcode, "LDA a16", 3, InstructionTiming::fixed(13)),
        0x3F => info(opcode, "CMC", 1, InstructionTiming::fixed(4)),
        0xC3 => info(opcode, "JMP a16", 3, InstructionTiming::fixed(10)),
        0xC6 => info(opcode, "ADI d8", 2, InstructionTiming::fixed(7)),
        0xC9 => info(opcode, "RET", 1, InstructionTiming::fixed(10)),
        0xCD => info(opcode, "CALL a16", 3, InstructionTiming::fixed(17)),
        0xCE => info(opcode, "ACI d8", 2, InstructionTiming::fixed(7)),
        0xD3 => info(opcode, "OUT d8", 2, InstructionTiming::fixed(10)),
        0xD6 => info(opcode, "SUI d8", 2, InstructionTiming::fixed(7)),
        0xDB => info(opcode, "IN d8", 2, InstructionTiming::fixed(10)),
        0xDE => info(opcode, "SBI d8", 2, InstructionTiming::fixed(7)),
        0xE3 => info(opcode, "XTHL", 1, InstructionTiming::fixed(18)),
        0xE6 => info(opcode, "ANI d8", 2, InstructionTiming::fixed(7)),
        0xE9 => info(opcode, "PCHL", 1, InstructionTiming::fixed(5)),
        0xEB => info(opcode, "XCHG", 1, InstructionTiming::fixed(5)),
        0xEE => info(opcode, "XRI d8", 2, InstructionTiming::fixed(7)),
        0xF3 => info(opcode, "DI", 1, InstructionTiming::fixed(4)),
        0xF6 => info(opcode, "ORI d8", 2, InstructionTiming::fixed(7)),
        0xF9 => info(opcode, "SPHL", 1, InstructionTiming::fixed(5)),
        0xFB => info(opcode, "EI", 1, InstructionTiming::fixed(4)),
        0xFE => info(opcode, "CPI d8", 2, InstructionTiming::fixed(7)),
        _ => unreachable!("documented opcode must be covered: {opcode:#04X}"),
    })
}

fn info(
    opcode: u8,
    mnemonic: impl Into<String>,
    size: u8,
    timing: InstructionTiming,
) -> InstructionInfo {
    InstructionInfo {
        opcode,
        mnemonic: mnemonic.into(),
        size,
        timing,
    }
}

fn stack_pair(opcode: u8) -> &'static str {
    match (opcode >> 4) & 3 {
        0 => "B",
        1 => "D",
        2 => "H",
        _ => "PSW",
    }
}
