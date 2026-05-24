use crate::SnapshotError;
use k580_core::{Cpu8080State, Flags, Memory64K};

/// Which on-disk `.580` flavour a freshly opened file turned out to be.
///
/// The two formats share the `.580` extension (the user's reference
/// emulator and ours both write to it) but the wire shape is
/// completely different: K580 v1 starts with a `K580` magic, then a
/// version word, then a TLV payload, while the legacy reference dump
/// is a flat 64 KiB of RAM followed by a 13-byte trailer ending in
/// `FF FF`. The double-click / `argv[1]` path therefore needs to
/// *probe* the file, not just trust the extension — and the caller
/// needs to know which branch matched so it can route a subsequent
/// "Сохранить" / "Сохранить (старый формат)" gesture to the right
/// serializer.
///
/// Returned by `Snapshot580Serializer::from_any_bytes`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Snapshot580Flavour {
    /// Modern, versioned K580 v1 format with `K580` magic and a TLV
    /// payload. Round-trips every CPU field.
    Modern,
    /// Reference 65 549-byte legacy dump. Carries RAM + PC only;
    /// every other CPU field comes back as default.
    Legacy,
}

pub struct Snapshot580Serializer;

impl Snapshot580Serializer {
    pub const MAGIC: &'static [u8; 4] = b"K580";
    pub const VERSION: u16 = 1;

    /// Length of the reference ("legacy") `.580` file produced by
    /// the original emulator the project was based on: 65 536 bytes
    /// of raw RAM followed by a 13-byte trailer. No magic, no
    /// version, no TLV — just a flat dump.
    pub const LEGACY_LENGTH: usize = Memory64K::SIZE + Self::LEGACY_TRAILER_LENGTH;
    /// Length of the legacy trailer that sits *after* the 64 KiB of
    /// RAM. Layout (offsets relative to the trailer's first byte):
    ///
    /// - `[0..9]`  — nine bytes of zeros in every reference file we
    ///   inspected. These almost certainly correspond to register /
    ///   flag / SP fields the reference emulator zeroes when the
    ///   snapshot is saved at idle, but without the original
    ///   emulator's source we cannot map them precisely. We write
    ///   zeros here on save and ignore the value on load — that is
    ///   what every reference file we have on disk does, so it
    ///   round-trips cleanly.
    /// - `[9..11]` — `PC_LO PC_HI` (little-endian u16). The slot
    ///   carries different values across the seven reference files
    ///   (`0x0011`, `0x0012`, `0x0000`, …); the only consistent
    ///   reading that fits all of them is "the program counter at
    ///   save time". We write our live `state.pc` here and read it
    ///   back into `state.pc` on load.
    /// - `[11..13]` — `FF FF` end-of-record marker. Constant across
    ///   every reference file; the loader rejects files where these
    ///   two bytes are anything else.
    pub const LEGACY_TRAILER_LENGTH: usize = 13;

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

    /// Serialize the CPU state into the reference ("legacy") 65 549-byte
    /// `.580` layout used by the emulator the project was originally
    /// based on. The output is `RAM (65 536 B) + 13-byte trailer`,
    /// where the trailer is `0x00 × 9 + PC_LO + PC_HI + 0xFF + 0xFF`.
    /// See `LEGACY_TRAILER_LENGTH` for why we map only PC across the
    /// 13 bytes — the reference format does not preserve registers,
    /// flags, SP, halt, or cycle counters, so this dumps RAM and PC
    /// only and leaves everything else for the K580 v1 path. Round-trips
    /// the seven reference files we have on disk byte-for-byte when
    /// loaded with `from_legacy_bytes` (the trailer's first 9 bytes are
    /// already zero in every reference file, and `FF FF` is constant).
    pub fn to_legacy_bytes(state: &Cpu8080State) -> Vec<u8> {
        let mut out = Vec::with_capacity(Self::LEGACY_LENGTH);
        out.extend_from_slice(state.memory.as_slice());
        // Nine zero bytes covering whatever the reference format
        // stores here (likely registers / flags / SP at idle). Every
        // reference file we have on disk leaves this zone clear, so
        // emitting zeros keeps the round-trip clean.
        out.resize(out.len() + 9, 0);
        out.extend_from_slice(&state.pc.to_le_bytes());
        // End-of-record marker. Constant across every reference file
        // we have; the loader rejects files where it's anything else.
        out.push(0xFF);
        out.push(0xFF);
        debug_assert_eq!(out.len(), Self::LEGACY_LENGTH);
        out
    }

    /// Probes `bytes` and dispatches to the matching deserializer.
    ///
    /// Both the modern K580 v1 format and the legacy reference dump share
    /// the `.580` extension, so a double-click / `argv[1]` open cannot
    /// trust the file name — it has to *probe* the contents. The cheap
    /// discriminator is the four-byte `K580` magic at offset 0: present
    /// → modern TLV path; absent → legacy 65 549-byte flat-RAM path.
    ///
    /// Returns the recovered CPU state plus the flavour that matched, so
    /// the UI can remember which serializer to use when the user later
    /// hits "Сохранить" (modern) versus "Сохранить (старый формат)"
    /// (legacy). Without that hint a legacy file opened by double-click
    /// would silently round-trip into the modern format on the next save.
    pub fn from_any_bytes(
        bytes: &[u8],
    ) -> Result<(Cpu8080State, Snapshot580Flavour), SnapshotError> {
        if bytes.len() >= 4 && &bytes[0..4] == Self::MAGIC {
            Self::from_bytes(bytes).map(|state| (state, Snapshot580Flavour::Modern))
        } else if bytes.len() == Self::LEGACY_LENGTH {
            Self::from_legacy_bytes(bytes).map(|state| (state, Snapshot580Flavour::Legacy))
        } else {
            // Neither magic nor legacy length matched. Surface the
            // modern-format error so the caller's existing diagnostics
            // ("not a valid .580 snapshot") still apply — the file
            // genuinely isn't either flavour.
            Err(SnapshotError::InvalidMagic)
        }
    }

    /// Parse a legacy 65 549-byte `.580` file produced by the original
    /// emulator. The format carries only RAM + PC (see
    /// `to_legacy_bytes` for the layout rationale), so registers,
    /// flags, SP, halt, interrupts, cycle count, and tact phase all
    /// come back as `Cpu8080State::default()`. That is what the
    /// reference files actually carry — every trailer we've inspected
    /// has nine zero bytes ahead of the PC slot — and it makes the
    /// "open legacy file" gesture predictable: the user gets RAM +
    /// resume-from-PC, with everything else in a clean power-on state.
    pub fn from_legacy_bytes(bytes: &[u8]) -> Result<Cpu8080State, SnapshotError> {
        if bytes.len() != Self::LEGACY_LENGTH {
            return Err(SnapshotError::InvalidLegacyLength(bytes.len()));
        }
        let trailer_start = Memory64K::SIZE;
        let trailer = &bytes[trailer_start..];
        if trailer[Self::LEGACY_TRAILER_LENGTH - 2] != 0xFF
            || trailer[Self::LEGACY_TRAILER_LENGTH - 1] != 0xFF
        {
            return Err(SnapshotError::InvalidLegacyTrailer);
        }
        let mut state = Cpu8080State::default();
        state
            .memory
            .as_mut_slice()
            .copy_from_slice(&bytes[..trailer_start]);
        state.pc = u16::from_le_bytes([trailer[9], trailer[10]]);
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
