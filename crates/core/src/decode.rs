mod table;

use crate::{DecodeError, InstructionTiming};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstructionInfo<M = String> {
    pub opcode: u8,
    pub mnemonic: M,
    pub size: u8,
    pub timing: InstructionTiming,
}

pub type InstructionMetadata = InstructionInfo<&'static str>;

pub const fn is_undocumented_opcode(opcode: u8) -> bool {
    matches!(
        opcode,
        0x08 | 0x10 | 0x18 | 0x20 | 0x28 | 0x30 | 0x38 | 0xCB | 0xD9 | 0xDD | 0xED | 0xFD
    )
}

pub fn decode_metadata(opcode: u8) -> Result<InstructionMetadata, DecodeError> {
    if is_undocumented_opcode(opcode) {
        return Err(DecodeError::UndocumentedOpcode(opcode));
    }
    let (mnemonic, size, t_states_taken, t_states_not_taken, machine_cycles) =
        table::OPCODES[usize::from(opcode)];
    Ok(InstructionInfo {
        opcode,
        mnemonic,
        size,
        timing: InstructionTiming {
            t_states_taken,
            t_states_not_taken,
            machine_cycles,
        },
    })
}

pub fn decode_opcode(opcode: u8) -> Result<InstructionInfo, DecodeError> {
    let metadata = decode_metadata(opcode)?;
    Ok(InstructionInfo {
        opcode,
        mnemonic: metadata.mnemonic.to_owned(),
        size: metadata.size,
        timing: metadata.timing,
    })
}
