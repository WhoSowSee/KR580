//! `.580` versioned binary snapshot.
//!
//! Format per `prompt/04_file_formats.md`:
//!
//! ```text
//! magic    : "K580" (4 bytes)
//! version  : u16 little-endian
//! length   : u32 little-endian — total payload size
//! payload  : a sequence of TLV blocks
//!
//! TLV layout:
//! tag      : u8
//! length   : u32 little-endian
//! value    : `length` bytes
//! ```
//!
//! Required tags are listed in the prompt and re-stated below. Unknown tags
//! with the high bit clear must fail; unknown tags with the high bit set may
//! be skipped by readers that don't understand them.

use crate::error::SnapshotError;
use kr580_core::{Cpu8080State, Flags, Memory64K};

/// Snapshot magic header.
pub const MAGIC: &[u8; 4] = b"K580";
/// Initial snapshot version.
pub const SNAPSHOT_VERSION: u16 = 1;

const TAG_RAM: u8 = 0x01;
const TAG_REGS: u8 = 0x02;
const TAG_FLAGS: u8 = 0x03;
const TAG_PC: u8 = 0x04;
const TAG_SP: u8 = 0x05;
const TAG_INT: u8 = 0x06;
const TAG_HALT: u8 = 0x07;
const TAG_TIMING: u8 = 0x08;

/// `.580` snapshot serializer.
pub struct Snapshot580Serializer;

impl Snapshot580Serializer {
    /// Serialize core state into a versioned `.580` byte stream.
    pub fn save(state: &Cpu8080State) -> Vec<u8> {
        let mut payload: Vec<u8> = Vec::with_capacity(0x11000);

        // RAM
        write_tlv(&mut payload, TAG_RAM, state.ram.as_slice());
        // Registers in fixed order: A B C D E H L
        let regs = [
            state.a, state.b, state.c, state.d, state.e, state.h, state.l,
        ];
        write_tlv(&mut payload, TAG_REGS, &regs);
        // Flags
        write_tlv(&mut payload, TAG_FLAGS, &[state.flags.to_psw_byte()]);
        // PC, SP
        write_tlv(&mut payload, TAG_PC, &state.pc.to_le_bytes());
        write_tlv(&mut payload, TAG_SP, &state.sp.to_le_bytes());
        // Interrupt state: ie, ie_pending, irq_pending, has_vector, vector_byte
        let int_block = [
            state.interrupt_enable as u8,
            state.interrupt_enable_pending as u8,
            state.interrupt_request_pending as u8,
            state.interrupt_vector_byte.is_some() as u8,
            state.interrupt_vector_byte.unwrap_or(0),
        ];
        write_tlv(&mut payload, TAG_INT, &int_block);
        // Halt
        write_tlv(&mut payload, TAG_HALT, &[state.halted as u8]);
        // Timing: cycle_count + optional tact_phase
        let mut timing = Vec::with_capacity(9);
        timing.extend_from_slice(&state.cycle_count.to_le_bytes());
        if let Some(phase) = state.tact_phase {
            timing.push(phase);
        }
        write_tlv(&mut payload, TAG_TIMING, &timing);

        // Header
        let mut out = Vec::with_capacity(payload.len() + 10);
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&SNAPSHOT_VERSION.to_le_bytes());
        out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        out.extend_from_slice(&payload);
        out
    }

    /// Deserialize a `.580` byte stream into a fresh `Cpu8080State`.
    pub fn load(bytes: &[u8]) -> Result<Cpu8080State, SnapshotError> {
        if bytes.len() < 10 {
            return Err(SnapshotError::Truncated {
                expected: 10,
                actual: bytes.len(),
            });
        }
        if &bytes[0..4] != MAGIC {
            return Err(SnapshotError::BadMagic);
        }
        let version = u16::from_le_bytes([bytes[4], bytes[5]]);
        if version != SNAPSHOT_VERSION {
            return Err(SnapshotError::UnsupportedVersion(version));
        }
        let payload_len = u32::from_le_bytes([bytes[6], bytes[7], bytes[8], bytes[9]]) as usize;
        let payload = &bytes[10..];
        if payload.len() < payload_len {
            return Err(SnapshotError::Truncated {
                expected: payload_len,
                actual: payload.len(),
            });
        }
        let payload = &payload[..payload_len];

        let mut state = Cpu8080State::new();
        let mut got_ram = false;
        let mut got_regs = false;
        let mut got_flags = false;
        let mut got_pc = false;
        let mut got_sp = false;
        let mut got_int = false;
        let mut got_halt = false;

        let mut idx = 0usize;
        while idx < payload.len() {
            if idx + 5 > payload.len() {
                return Err(SnapshotError::Truncated {
                    expected: idx + 5,
                    actual: payload.len(),
                });
            }
            let tag = payload[idx];
            let len = u32::from_le_bytes([
                payload[idx + 1],
                payload[idx + 2],
                payload[idx + 3],
                payload[idx + 4],
            ]) as usize;
            idx += 5;
            if idx + len > payload.len() {
                return Err(SnapshotError::Truncated {
                    expected: idx + len,
                    actual: payload.len(),
                });
            }
            let value = &payload[idx..idx + len];
            idx += len;

            match tag {
                TAG_RAM => {
                    let mut ram = Memory64K::new();
                    ram.replace_from(value)
                        .map_err(|_| SnapshotError::MemorySize)?;
                    state.ram = ram;
                    got_ram = true;
                }
                TAG_REGS => {
                    if value.len() != 7 {
                        return Err(SnapshotError::InvalidLength {
                            tag,
                            len: len as u32,
                        });
                    }
                    state.a = value[0];
                    state.b = value[1];
                    state.c = value[2];
                    state.d = value[3];
                    state.e = value[4];
                    state.h = value[5];
                    state.l = value[6];
                    got_regs = true;
                }
                TAG_FLAGS => {
                    if value.len() != 1 {
                        return Err(SnapshotError::InvalidLength {
                            tag,
                            len: len as u32,
                        });
                    }
                    state.flags = Flags::from_psw_byte(value[0]);
                    got_flags = true;
                }
                TAG_PC => {
                    if value.len() != 2 {
                        return Err(SnapshotError::InvalidLength {
                            tag,
                            len: len as u32,
                        });
                    }
                    state.pc = u16::from_le_bytes([value[0], value[1]]);
                    got_pc = true;
                }
                TAG_SP => {
                    if value.len() != 2 {
                        return Err(SnapshotError::InvalidLength {
                            tag,
                            len: len as u32,
                        });
                    }
                    state.sp = u16::from_le_bytes([value[0], value[1]]);
                    got_sp = true;
                }
                TAG_INT => {
                    if value.len() != 5 {
                        return Err(SnapshotError::InvalidLength {
                            tag,
                            len: len as u32,
                        });
                    }
                    state.interrupt_enable = value[0] != 0;
                    state.interrupt_enable_pending = value[1] != 0;
                    state.interrupt_request_pending = value[2] != 0;
                    state.interrupt_vector_byte = if value[3] != 0 { Some(value[4]) } else { None };
                    got_int = true;
                }
                TAG_HALT => {
                    if value.len() != 1 {
                        return Err(SnapshotError::InvalidLength {
                            tag,
                            len: len as u32,
                        });
                    }
                    state.halted = value[0] != 0;
                    got_halt = true;
                }
                TAG_TIMING => {
                    if value.len() < 8 {
                        return Err(SnapshotError::InvalidLength {
                            tag,
                            len: len as u32,
                        });
                    }
                    state.cycle_count = u64::from_le_bytes([
                        value[0], value[1], value[2], value[3], value[4], value[5], value[6],
                        value[7],
                    ]);
                    state.tact_phase = if value.len() >= 9 {
                        Some(value[8])
                    } else {
                        None
                    };
                }
                other => {
                    if other & 0x80 == 0 {
                        return Err(SnapshotError::UnsupportedTag(other));
                    }
                    // High-bit-set unknown tags may be skipped silently.
                }
            }
        }

        for (got, tag) in [
            (got_ram, TAG_RAM),
            (got_regs, TAG_REGS),
            (got_flags, TAG_FLAGS),
            (got_pc, TAG_PC),
            (got_sp, TAG_SP),
            (got_int, TAG_INT),
            (got_halt, TAG_HALT),
        ] {
            if !got {
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

#[cfg(test)]
mod tests {
    use super::*;
    use kr580_core::{Cpu8080State, Flags};

    fn rich_state() -> Cpu8080State {
        let mut s = Cpu8080State::new();
        s.a = 0xAB;
        s.b = 0x01;
        s.c = 0x02;
        s.d = 0x03;
        s.e = 0x04;
        s.h = 0x05;
        s.l = 0x06;
        s.pc = 0x1234;
        s.sp = 0x4321;
        s.flags = Flags {
            s: true,
            z: false,
            ac: true,
            p: true,
            cy: false,
        };
        s.ram.write(0x0100, 0x42);
        s.interrupt_enable = true;
        s.interrupt_enable_pending = false;
        s.interrupt_request_pending = true;
        s.interrupt_vector_byte = Some(0xFF);
        s.halted = false;
        s.cycle_count = 9_999;
        s.tact_phase = Some(2);
        s
    }

    #[test]
    fn header_layout_is_stable() {
        let state = rich_state();
        let bytes = Snapshot580Serializer::save(&state);
        assert_eq!(&bytes[0..4], MAGIC);
        assert_eq!(u16::from_le_bytes([bytes[4], bytes[5]]), SNAPSHOT_VERSION);
    }

    #[test]
    fn roundtrip_preserves_state() {
        let state = rich_state();
        let bytes = Snapshot580Serializer::save(&state);
        let back = Snapshot580Serializer::load(&bytes).unwrap();
        assert_eq!(back.a, state.a);
        assert_eq!(back.pc, state.pc);
        assert_eq!(back.sp, state.sp);
        assert_eq!(back.flags, state.flags);
        assert_eq!(back.ram.read(0x0100), 0x42);
        assert!(back.interrupt_enable);
        assert!(back.interrupt_request_pending);
        assert_eq!(back.interrupt_vector_byte, Some(0xFF));
        assert_eq!(back.cycle_count, 9_999);
        assert_eq!(back.tact_phase, Some(2));
    }

    #[test]
    fn bad_magic_rejected() {
        let mut bytes = Snapshot580Serializer::save(&Cpu8080State::new());
        bytes[0] = b'X';
        assert_eq!(
            Snapshot580Serializer::load(&bytes).unwrap_err(),
            SnapshotError::BadMagic
        );
    }

    #[test]
    fn unsupported_low_bit_tag_rejected() {
        // Build a stream with a single unknown low-bit tag.
        let mut payload = Vec::new();
        write_tlv(&mut payload, 0x09, &[]);
        let mut bytes = Vec::new();
        bytes.extend_from_slice(MAGIC);
        bytes.extend_from_slice(&SNAPSHOT_VERSION.to_le_bytes());
        bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&payload);
        let err = Snapshot580Serializer::load(&bytes).unwrap_err();
        assert_eq!(err, SnapshotError::UnsupportedTag(0x09));
    }

    #[test]
    fn high_bit_unknown_tag_is_skipped() {
        // Save real state, then inject a high-bit tag at the end.
        let state = rich_state();
        let real = Snapshot580Serializer::save(&state);
        // Re-use the payload by extending it with a high-bit tag.
        let header = &real[..10];
        let payload = &real[10..];
        let mut new_payload = payload.to_vec();
        write_tlv(&mut new_payload, 0x80, &[1, 2, 3]);
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&header[0..6]); // magic + version
        bytes.extend_from_slice(&(new_payload.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&new_payload);
        let back = Snapshot580Serializer::load(&bytes).unwrap();
        assert_eq!(back.pc, state.pc);
    }

    #[test]
    fn unsupported_version_rejected() {
        let mut bytes = Snapshot580Serializer::save(&Cpu8080State::new());
        bytes[4] = 0x99;
        bytes[5] = 0x00;
        assert!(matches!(
            Snapshot580Serializer::load(&bytes).unwrap_err(),
            SnapshotError::UnsupportedVersion(0x99)
        ));
    }
}
