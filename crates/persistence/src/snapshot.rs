use crate::SnapshotError;
use k580_core::{Cpu8080State, Flags, Memory64K};

pub struct Snapshot580Serializer;

impl Snapshot580Serializer {
    pub const MAGIC: &'static [u8; 4] = b"K580";
    pub const VERSION: u16 = 1;

    pub fn to_bytes(state: &Cpu8080State) -> Vec<u8> {
        let mut payload = Vec::new();
        write_tlv(&mut payload, 0x01, state.memory.as_slice());
        write_tlv(
            &mut payload,
            0x02,
            &[
                state.registers.a,
                state.registers.b,
                state.registers.c,
                state.registers.d,
                state.registers.e,
                state.registers.h,
                state.registers.l,
            ],
        );
        write_tlv(&mut payload, 0x03, &[state.flags.to_psw()]);
        write_tlv(&mut payload, 0x04, &state.pc.to_le_bytes());
        write_tlv(&mut payload, 0x05, &state.sp.to_le_bytes());
        write_tlv(
            &mut payload,
            0x06,
            &[
                u8::from(state.interrupt_enable),
                u8::from(state.interrupt_enable_pending),
                u8::from(state.interrupt_request_pending),
                u8::from(state.interrupt_vector_byte.is_some()),
                state.interrupt_vector_byte.unwrap_or(0),
            ],
        );
        write_tlv(&mut payload, 0x07, &[u8::from(state.halted)]);
        let mut timing = state.cycle_count.to_le_bytes().to_vec();
        if let Some(phase) = state.tact_phase {
            timing.push(phase);
        }
        write_tlv(&mut payload, 0x08, &timing);

        let mut out = Vec::with_capacity(10 + payload.len());
        out.extend_from_slice(Self::MAGIC);
        out.extend_from_slice(&Self::VERSION.to_le_bytes());
        out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        out.extend_from_slice(&payload);
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Cpu8080State, SnapshotError> {
        if bytes.len() < 10 {
            return Err(SnapshotError::Truncated);
        }
        if &bytes[0..4] != Self::MAGIC {
            return Err(SnapshotError::InvalidMagic);
        }
        let version = u16::from_le_bytes([bytes[4], bytes[5]]);
        if version != Self::VERSION {
            return Err(SnapshotError::UnsupportedVersion(version));
        }
        let payload_len = u32::from_le_bytes([bytes[6], bytes[7], bytes[8], bytes[9]]) as usize;
        if bytes.len() - 10 != payload_len {
            return Err(SnapshotError::PayloadLengthMismatch);
        }

        let mut state = Cpu8080State::default();
        let mut seen = [false; 9];
        let mut offset = 10;
        while offset < bytes.len() {
            let (tag, value, next) = read_tlv(bytes, offset)?;
            offset = next;
            match tag {
                0x01 => read_ram(&mut state, value)?,
                0x02 => read_registers(&mut state, value)?,
                0x03 => read_flags(&mut state, value)?,
                0x04 => state.pc = read_u16(tag, value)?,
                0x05 => state.sp = read_u16(tag, value)?,
                0x06 => read_interrupts(&mut state, value)?,
                0x07 => state.halted = read_bool(tag, value)?,
                0x08 => read_timing(&mut state, value)?,
                unknown if unknown & 0x80 != 0 => {}
                unknown => return Err(SnapshotError::UnsupportedTag(unknown)),
            }
            if (1..=8).contains(&tag) {
                seen[tag as usize] = true;
            }
        }

        for tag in 1u8..=8 {
            if tag != 0x08 && !seen[tag as usize] {
                return Err(SnapshotError::MissingTag(tag));
            }
        }
        Ok(state)
    }
}

fn write_tlv(out: &mut Vec<u8>, tag: u8, value: &[u8]) {
    out.push(tag);
    out.extend_from_slice(&(value.len() as u32).to_le_bytes());
    out.extend_from_slice(value);
}

fn read_tlv(bytes: &[u8], offset: usize) -> Result<(u8, &[u8], usize), SnapshotError> {
    if bytes.len() < offset + 5 {
        return Err(SnapshotError::Truncated);
    }
    let tag = bytes[offset];
    let length = u32::from_le_bytes(bytes[offset + 1..offset + 5].try_into().unwrap()) as usize;
    let start = offset + 5;
    let end = start.checked_add(length).ok_or(SnapshotError::Truncated)?;
    if end > bytes.len() {
        return Err(SnapshotError::Truncated);
    }
    Ok((tag, &bytes[start..end], end))
}

fn read_ram(state: &mut Cpu8080State, value: &[u8]) -> Result<(), SnapshotError> {
    if value.len() != Memory64K::SIZE {
        return Err(SnapshotError::InvalidLength {
            tag: 0x01,
            length: value.len(),
        });
    }
    state.memory.as_mut_slice().copy_from_slice(value);
    Ok(())
}

fn read_registers(state: &mut Cpu8080State, value: &[u8]) -> Result<(), SnapshotError> {
    if value.len() != 7 {
        return Err(SnapshotError::InvalidLength {
            tag: 0x02,
            length: value.len(),
        });
    }
    state.registers.a = value[0];
    state.registers.b = value[1];
    state.registers.c = value[2];
    state.registers.d = value[3];
    state.registers.e = value[4];
    state.registers.h = value[5];
    state.registers.l = value[6];
    Ok(())
}

fn read_flags(state: &mut Cpu8080State, value: &[u8]) -> Result<(), SnapshotError> {
    if value.len() != 1 {
        return Err(SnapshotError::InvalidLength {
            tag: 0x03,
            length: value.len(),
        });
    }
    state.flags = Flags::from_psw(value[0]);
    Ok(())
}

fn read_u16(tag: u8, value: &[u8]) -> Result<u16, SnapshotError> {
    if value.len() != 2 {
        return Err(SnapshotError::InvalidLength {
            tag,
            length: value.len(),
        });
    }
    Ok(u16::from_le_bytes([value[0], value[1]]))
}

fn read_bool(tag: u8, value: &[u8]) -> Result<bool, SnapshotError> {
    if value.len() != 1 || value[0] > 1 {
        return Err(SnapshotError::InvalidLength {
            tag,
            length: value.len(),
        });
    }
    Ok(value[0] != 0)
}

fn read_interrupts(state: &mut Cpu8080State, value: &[u8]) -> Result<(), SnapshotError> {
    if value.len() != 5 {
        return Err(SnapshotError::InvalidLength {
            tag: 0x06,
            length: value.len(),
        });
    }
    state.interrupt_enable = value[0] != 0;
    state.interrupt_enable_pending = value[1] != 0;
    state.interrupt_request_pending = value[2] != 0;
    state.interrupt_vector_byte = (value[3] != 0).then_some(value[4]);
    Ok(())
}

fn read_timing(state: &mut Cpu8080State, value: &[u8]) -> Result<(), SnapshotError> {
    if value.len() != 8 && value.len() != 9 {
        return Err(SnapshotError::InvalidLength {
            tag: 0x08,
            length: value.len(),
        });
    }
    state.cycle_count = u64::from_le_bytes(value[0..8].try_into().unwrap());
    state.tact_phase = value.get(8).copied();
    Ok(())
}
