use k580_core::{Cpu8080State, Flags};
use k580_persistence::{
    ExportModel, Exporters, Importers, Settings, SettingsError, SettingsStore, Snapshot580Flavour,
    Snapshot580Serializer, SnapshotError, Subprogram, SubprogramSerializer,
};
use std::path::PathBuf;

#[test]
fn snapshot_roundtrips_core_state_without_ui_data() {
    let mut cpu = Cpu8080State::default();
    cpu.registers.a = 0xA5;
    cpu.registers.b = 0x12;
    cpu.pc = 0x1234;
    cpu.sp = 0xABCD;
    cpu.flags = Flags {
        sign: true,
        zero: false,
        auxiliary_carry: true,
        parity: true,
        carry: true,
    };
    cpu.memory.write(0x4000, 0x5A);
    cpu.interrupt_enable = true;
    cpu.interrupt_request_pending = true;
    cpu.interrupt_vector_byte = Some(0xCF);
    cpu.halted = true;
    cpu.cycle_count = 123_456;
    cpu.tact_phase = Some(3);
    cpu.last_completed_tact_phase = Some(7);

    let bytes = Snapshot580Serializer::to_bytes(&cpu);
    assert_eq!(&bytes[0..4], b"K580");
    let restored = Snapshot580Serializer::from_bytes(&bytes).unwrap();
    assert_eq!(restored.registers, cpu.registers);
    assert_eq!(restored.flags.to_psw(), cpu.flags.to_psw());
    assert_eq!(restored.pc, cpu.pc);
    assert_eq!(restored.sp, cpu.sp);
    assert_eq!(restored.memory.read(0x4000), 0x5A);
    assert_eq!(restored.interrupt_vector_byte, Some(0xCF));
    assert_eq!(restored.cycle_count, 123_456);
    assert_eq!(restored.tact_phase, Some(3));
    assert_eq!(restored.last_completed_tact_phase, Some(7));
}

/// Sentinel path in the timing TLV: `tact_phase == None` but
/// `last_completed_tact_phase == Some(_)`. The writer uses sentinel
/// `0xFF` in slot[8] to distinguish "no active phase, last completed
/// is valid" from "neither field set". The loader must round-trip
/// the same pair, otherwise the UI redraws `-` in the tact row
/// instead of the frozen last T-phase.
#[test]
fn snapshot_roundtrips_last_completed_with_no_active_tact_phase() {
    let mut cpu = Cpu8080State::default();
    cpu.tact_phase = None;
    cpu.last_completed_tact_phase = Some(6);
    cpu.cycle_count = 7;

    let bytes = Snapshot580Serializer::to_bytes(&cpu);
    let restored = Snapshot580Serializer::from_bytes(&bytes).unwrap();
    assert_eq!(restored.tact_phase, None);
    assert_eq!(restored.last_completed_tact_phase, Some(6));
    assert_eq!(restored.cycle_count, 7);
}

/// Cold path: both fields `None`. The writer must emit neither
/// slot[8] nor slot[9]; the reader must see an 8-byte payload and
/// keep both fields `None` instead of returning `InvalidLength`.
#[test]
fn snapshot_roundtrips_both_tact_phases_none() {
    let cpu = Cpu8080State::default();
    let bytes = Snapshot580Serializer::to_bytes(&cpu);
    let restored = Snapshot580Serializer::from_bytes(&bytes).unwrap();
    assert_eq!(restored.tact_phase, None);
    assert_eq!(restored.last_completed_tact_phase, None);
}

/// Backward compat: 9-byte timing payload (`cycle_count` + `tact_phase`
/// only, no `last_completed_tact_phase`). Files saved before the field
/// was added must load without migration — `tact_phase` restores and
/// `last_completed_tact_phase` stays `None`. Otherwise older `.580` v1
/// snapshots fail with `SnapshotError`.
#[test]
fn snapshot_loads_legacy_v1_payload_without_last_completed() {
    let mut cpu = Cpu8080State::default();
    cpu.cycle_count = 0x1122_3344_5566_7788;
    cpu.tact_phase = Some(2);
    cpu.last_completed_tact_phase = None;

    // Legacy 9-byte timing payload: 8 LE bytes of cycle_count + 1
    // byte of tact_phase, no slot[9].
    let mut timing = cpu.cycle_count.to_le_bytes().to_vec();
    timing.push(2);
    assert_eq!(timing.len(), 9);

    // Replace tag 0x08 with the 9-byte blob; other tags from the
    // standard writer for `Cpu8080State::default()`.
    let full = Snapshot580Serializer::to_bytes(&cpu);
    let legacy_bytes = rewrite_timing_tlv(full, &timing);

    let restored = Snapshot580Serializer::from_bytes(&legacy_bytes).unwrap();
    assert_eq!(restored.cycle_count, cpu.cycle_count);
    assert_eq!(restored.tact_phase, Some(2));
    assert_eq!(restored.last_completed_tact_phase, None);
}

#[test]
fn legacy_snapshot_roundtrips_ram_and_pc() {
    // Legacy `.580` is a flat 64 KiB RAM dump + 13-byte trailer
    // carrying PC. RAM and PC must round-trip; everything else is
    // not encoded by this format.
    let mut cpu = Cpu8080State::default();
    cpu.memory.write(0x0000, 0x3E);
    cpu.memory.write(0x0001, 0x42);
    cpu.memory.write(0xFFFF, 0xAA);
    cpu.memory.write(0x4000, 0x5A);
    cpu.pc = 0xBEEF;
    // Non-default fields prove the legacy reader doesn't pick them up.
    cpu.registers.a = 0x99;
    cpu.sp = 0xC0DE;
    cpu.halted = true;

    let bytes = Snapshot580Serializer::to_legacy_bytes(&cpu);
    assert_eq!(bytes.len(), Snapshot580Serializer::LEGACY_LENGTH);
    // The reference always ends with FF FF; without it the loader rejects.
    assert_eq!(bytes[bytes.len() - 2], 0xFF);
    assert_eq!(bytes[bytes.len() - 1], 0xFF);

    let restored = Snapshot580Serializer::from_legacy_bytes(&bytes).unwrap();
    assert_eq!(restored.memory.read(0x0000), 0x3E);
    assert_eq!(restored.memory.read(0x0001), 0x42);
    assert_eq!(restored.memory.read(0xFFFF), 0xAA);
    assert_eq!(restored.memory.read(0x4000), 0x5A);
    assert_eq!(restored.pc, 0xBEEF);
    // Unencoded fields fall back to `Cpu8080State::default()`; SP =
    // RESET_SP (0xFFFF), not zero.
    assert_eq!(restored.registers.a, 0);
    assert_eq!(restored.sp, Cpu8080State::RESET_SP);
    assert!(!restored.halted);
}

#[test]
fn legacy_snapshot_rejects_bad_length_and_trailer() {
    let cpu = Cpu8080State::default();
    let mut bytes = Snapshot580Serializer::to_legacy_bytes(&cpu);

    let truncated = &bytes[..bytes.len() - 1];
    assert!(matches!(
        Snapshot580Serializer::from_legacy_bytes(truncated),
        Err(SnapshotError::InvalidLegacyLength(_))
    ));

    // Wipe the FF FF marker — every reference legacy `.580` ends with it.
    let len = bytes.len();
    bytes[len - 2] = 0x00;
    bytes[len - 1] = 0x00;
    assert!(matches!(
        Snapshot580Serializer::from_legacy_bytes(&bytes),
        Err(SnapshotError::InvalidLegacyTrailer)
    ));
}

#[test]
fn snapshot_bytes_are_deterministic_and_reject_bad_headers() {
    let mut cpu = Cpu8080State::default();
    cpu.registers.a = 0x5A;
    cpu.memory.write(0x0100, 0xC3);
    let first = Snapshot580Serializer::to_bytes(&cpu);
    let second = Snapshot580Serializer::to_bytes(&cpu);
    assert_eq!(first, second);

    let mut unsupported = first.clone();
    unsupported[4..6].copy_from_slice(&2u16.to_le_bytes());
    assert!(matches!(
        Snapshot580Serializer::from_bytes(&unsupported),
        Err(SnapshotError::UnsupportedVersion(2))
    ));

    let mut bad_len = first;
    bad_len[6..10].copy_from_slice(&1u32.to_le_bytes());
    assert!(matches!(
        Snapshot580Serializer::from_bytes(&bad_len),
        Err(SnapshotError::PayloadLengthMismatch)
    ));
}

#[test]
fn snapshot_unknown_low_tag_fails_and_high_tag_is_skipped() {
    let cpu = Cpu8080State::default();
    let base = Snapshot580Serializer::to_bytes(&cpu);

    let high = append_tlv(base.clone(), 0x80, &[1, 2, 3]);
    assert!(Snapshot580Serializer::from_bytes(&high).is_ok());

    let low = append_tlv(base, 0x09, &[]);
    assert!(matches!(
        Snapshot580Serializer::from_bytes(&low),
        Err(SnapshotError::UnsupportedTag(0x09))
    ));
}

#[test]
fn krs_is_raw_slice_with_explicit_base_address() {
    let subprogram = Subprogram {
        base_address: 0x2000,
        bytes: vec![1, 2, 3, 4],
    };
    assert_eq!(
        SubprogramSerializer::to_bytes(&subprogram),
        vec![1, 2, 3, 4]
    );
    let mut cpu = Cpu8080State::default();
    SubprogramSerializer::load_into_state(&mut cpu, &subprogram).unwrap();
    assert_eq!(cpu.memory.read(0x2000), 1);
    assert_eq!(cpu.memory.read(0x2003), 4);
}

#[test]
fn krs_load_rejects_memory_overflow() {
    let subprogram = Subprogram {
        base_address: 0xFFFE,
        bytes: vec![1, 2, 3],
    };
    let mut cpu = Cpu8080State::default();
    let err = SubprogramSerializer::load_into_state(&mut cpu, &subprogram).unwrap_err();
    assert_eq!(
        err,
        k580_core::ValidationError::MemoryRange {
            start: 0xFFFE,
            end: 0x10001
        }
    );
}

#[test]
fn settings_are_versioned_camel_case_json() {
    let settings = Settings::default();
    let json = SettingsStore::to_json(&settings).unwrap();
    assert!(json.contains("settingsVersion"));
    assert!(json.contains("recentFiles"));
    assert_eq!(SettingsStore::from_json(&json).unwrap(), settings);

    let unsupported = json.replace("\"settingsVersion\": 1", "\"settingsVersion\": 2");
    assert!(matches!(
        SettingsStore::from_json(&unsupported),
        Err(SettingsError::UnsupportedVersion(2))
    ));
}

#[test]
fn exporters_write_stable_direct_files() {
    let mut cpu = Cpu8080State::default();
    cpu.registers.a = 0x42;
    cpu.memory.write(0, 0x76);
    let model = ExportModel::from_cpu(&cpu);
    let dir = unique_temp_dir();
    std::fs::create_dir_all(&dir).unwrap();
    let txt = dir.join("state.txt");
    let xlsx = dir.join("state.xlsx");

    Exporters::write_txt(&txt, &model).unwrap();
    Exporters::write_xlsx(&xlsx, &model).unwrap();

    let text = std::fs::read_to_string(&txt).unwrap();
    assert!(text.contains("[Registers]"));
    assert!(text.contains("A=42"));
    assert!(std::fs::metadata(&xlsx).unwrap().len() > 0);

    std::fs::remove_file(txt).ok();
    std::fs::remove_file(xlsx).ok();
    std::fs::remove_dir(dir).ok();
}

#[test]
fn importers_round_trip_txt_and_xlsx() {
    let mut cpu = Cpu8080State::default();
    cpu.registers.a = 0xA5;
    cpu.registers.b = 0x12;
    cpu.registers.c = 0x34;
    cpu.pc = 0x1234;
    cpu.sp = 0xABCD;
    cpu.flags = Flags {
        sign: true,
        zero: false,
        auxiliary_carry: true,
        parity: false,
        carry: true,
    };
    cpu.memory.write(0, 0x76);
    cpu.memory.write(1, 0xC3);
    cpu.memory.write(2, 0x00);
    cpu.memory.write(3, 0x10);
    cpu.cycle_count = 4242;

    let model = ExportModel::from_cpu(&cpu);
    let dir = unique_temp_dir();
    std::fs::create_dir_all(&dir).unwrap();
    let txt = dir.join("state.txt");
    let xlsx = dir.join("state.xlsx");

    Exporters::write_txt(&txt, &model).unwrap();
    Exporters::write_xlsx(&xlsx, &model).unwrap();

    let from_txt = Importers::read_txt(&txt).unwrap();
    assert_eq!(from_txt, model);

    let from_xlsx = Importers::read_xlsx(&xlsx).unwrap();
    assert_eq!(from_xlsx, model);

    let mut restored = Cpu8080State::default();
    from_txt.apply_to(&mut restored).unwrap();
    assert_eq!(restored.registers, cpu.registers);
    assert_eq!(restored.flags, cpu.flags);
    assert_eq!(restored.pc, cpu.pc);
    assert_eq!(restored.sp, cpu.sp);
    assert_eq!(restored.cycle_count, cpu.cycle_count);
    assert_eq!(restored.memory.read(0), 0x76);
    assert_eq!(restored.memory.read(3), 0x10);

    std::fs::remove_file(txt).ok();
    std::fs::remove_file(xlsx).ok();
    std::fs::remove_dir(dir).ok();
}

fn append_tlv(mut bytes: Vec<u8>, tag: u8, value: &[u8]) -> Vec<u8> {
    bytes.push(tag);
    bytes.extend_from_slice(&(value.len() as u32).to_le_bytes());
    bytes.extend_from_slice(value);
    let payload_len = (bytes.len() - 10) as u32;
    bytes[6..10].copy_from_slice(&payload_len.to_le_bytes());
    bytes
}

/// Find the timing TLV (tag 0x08) inside a snapshot and replace its
/// value, recomputing the payload length. Used to synthesise a legacy
/// 9-byte timing TLV from the new-format writer output.
fn rewrite_timing_tlv(bytes: Vec<u8>, new_value: &[u8]) -> Vec<u8> {
    let mut out = bytes[..10].to_vec();
    let mut offset = 10;
    while offset < bytes.len() {
        let tag = bytes[offset];
        let length = u32::from_le_bytes(bytes[offset + 1..offset + 5].try_into().unwrap()) as usize;
        let start = offset + 5;
        let end = start + length;
        if tag == 0x08 {
            out.push(tag);
            out.extend_from_slice(&(new_value.len() as u32).to_le_bytes());
            out.extend_from_slice(new_value);
        } else {
            out.extend_from_slice(&bytes[offset..end]);
        }
        offset = end;
    }
    let payload_len = (out.len() - 10) as u32;
    out[6..10].copy_from_slice(&payload_len.to_le_bytes());
    out
}

fn unique_temp_dir() -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("k580-persistence-{nanos}"))
}

/// Auto-detect path: a K580 v1 blob (with the `K580` magic) must
/// resolve to `Snapshot580Flavour::Modern` and round-trip every CPU
/// field — same path a double-clicked modern `.580` goes through.
#[test]
fn from_any_bytes_recognises_modern_snapshot() {
    let mut cpu = Cpu8080State::default();
    cpu.registers.a = 0x77;
    cpu.pc = 0x4321;
    cpu.sp = 0x8000;
    cpu.flags.carry = true;
    cpu.memory.write(0x1000, 0xAB);

    let bytes = Snapshot580Serializer::to_bytes(&cpu);
    let (restored, flavour) = Snapshot580Serializer::from_any_bytes(&bytes).unwrap();
    assert_eq!(flavour, Snapshot580Flavour::Modern);
    assert_eq!(restored.registers.a, 0x77);
    assert_eq!(restored.pc, 0x4321);
    assert_eq!(restored.sp, 0x8000);
    assert!(restored.flags.carry);
    assert_eq!(restored.memory.read(0x1000), 0xAB);
}

/// Auto-detect path: a 65 549-byte legacy dump (no magic, ends with
/// `FF FF`) must resolve to `Snapshot580Flavour::Legacy`. Regression
/// for the bug where double-clicking such a file failed with
/// `InvalidMagic` because the UI dispatched the modern decoder
/// unconditionally.
#[test]
fn from_any_bytes_recognises_legacy_snapshot() {
    let mut cpu = Cpu8080State::default();
    cpu.pc = 0x1234;
    cpu.memory.write(0x0100, 0x42);
    cpu.memory.write(0xFFFE, 0x99);

    let bytes = Snapshot580Serializer::to_legacy_bytes(&cpu);
    assert_eq!(bytes.len(), Snapshot580Serializer::LEGACY_LENGTH);

    let (restored, flavour) = Snapshot580Serializer::from_any_bytes(&bytes).unwrap();
    assert_eq!(flavour, Snapshot580Flavour::Legacy);
    assert_eq!(restored.pc, 0x1234);
    assert_eq!(restored.memory.read(0x0100), 0x42);
    assert_eq!(restored.memory.read(0xFFFE), 0x99);
    // Defaults survive; SP falls back to RESET_SP (0xFFFF), not zero.
    assert_eq!(restored.registers.a, 0);
    assert_eq!(restored.sp, Cpu8080State::RESET_SP);
    assert!(!restored.halted);
}

/// Garbage with no magic and the wrong length must surface
/// `InvalidMagic`, not the modern decoder's `Truncated`.
#[test]
fn from_any_bytes_rejects_unrecognised_blob() {
    let garbage = vec![0u8; 100];
    let err = Snapshot580Serializer::from_any_bytes(&garbage).unwrap_err();
    assert!(matches!(err, SnapshotError::InvalidMagic));
}

/// A modern blob whose magic matches but whose body is corrupt must
/// not be silently misclassified as legacy — the magic pins the
/// flavour and the modern decoder's error bubbles up.
#[test]
fn from_any_bytes_propagates_modern_decode_error() {
    let mut bytes = Snapshot580Serializer::to_bytes(&Cpu8080State::default());
    // Corrupt the payload length while keeping the K580 magic intact.
    bytes[6] = 0xFF;
    bytes[7] = 0xFF;
    let err = Snapshot580Serializer::from_any_bytes(&bytes).unwrap_err();
    assert!(matches!(err, SnapshotError::PayloadLengthMismatch));
}

/// A 65 549-byte blob with the wrong end-of-record marker must be
/// rejected by the legacy decoder rather than silently round-trip.
#[test]
fn from_any_bytes_propagates_legacy_decode_error() {
    let mut bytes = Snapshot580Serializer::to_legacy_bytes(&Cpu8080State::default());
    let last = bytes.len() - 1;
    bytes[last] = 0x00; // break the FF FF tail
    let err = Snapshot580Serializer::from_any_bytes(&bytes).unwrap_err();
    assert!(matches!(err, SnapshotError::InvalidLegacyTrailer));
}
