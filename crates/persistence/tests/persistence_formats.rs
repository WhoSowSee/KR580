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

/// Sentinel-путь в timing-TLV: `tact_phase == None`, но
/// `last_completed_tact_phase == Some(_)`. Запись использует sentinel
/// 0xFF в slot[8] чтобы отличить «нет активной фазы, есть последняя
/// выполненная» от «нет ни одной». Loader должен прочитать обратно
/// ровно ту же пару — иначе после сохранения/загрузки UI будет
/// рисовать `-` в строке «Такт» вместо застывшей последней T-фазы.
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

/// Холодный путь: оба поля `None`. Writer не должен писать ни slot[8],
/// ни slot[9]; reader должен видеть 8-байтовый payload и оставить
/// оба поля `None` (Default), не падая в `InvalidLength`.
#[test]
fn snapshot_roundtrips_both_tact_phases_none() {
    let cpu = Cpu8080State::default();
    let bytes = Snapshot580Serializer::to_bytes(&cpu);
    let restored = Snapshot580Serializer::from_bytes(&bytes).unwrap();
    assert_eq!(restored.tact_phase, None);
    assert_eq!(restored.last_completed_tact_phase, None);
}

/// Backward compat: старый 9-байтовый timing-payload (только
/// `cycle_count` + `tact_phase`, без `last_completed_tact_phase`).
/// Файлы, сохранённые до добавления поля, должны грузиться без
/// миграции — `tact_phase` восстанавливается, `last_completed_tact_phase`
/// остаётся `None`. Иначе пользователь получит `SnapshotError`
/// при открытии прежних `.580` v1.
#[test]
fn snapshot_loads_legacy_v1_payload_without_last_completed() {
    let mut cpu = Cpu8080State::default();
    cpu.cycle_count = 0x1122_3344_5566_7788;
    cpu.tact_phase = Some(2);
    cpu.last_completed_tact_phase = None;

    // Соберём timing-payload руками в старом формате (9 байт):
    // 8 байт cycle_count LE + 1 байт tact_phase. Без slot[9].
    let mut timing = cpu.cycle_count.to_le_bytes().to_vec();
    timing.push(2);
    assert_eq!(timing.len(), 9);

    // Соберём весь снимок руками: магик + версия + payload, где
    // tag 0x08 несёт нашу 9-байтовую timing-нагрузку, а остальные
    // теги берём из стандартного writer'а Cpu8080State::default().
    let full = Snapshot580Serializer::to_bytes(&cpu);
    let legacy_bytes = rewrite_timing_tlv(full, &timing);

    let restored = Snapshot580Serializer::from_bytes(&legacy_bytes).unwrap();
    assert_eq!(restored.cycle_count, cpu.cycle_count);
    assert_eq!(restored.tact_phase, Some(2));
    assert_eq!(restored.last_completed_tact_phase, None);
}

#[test]
fn legacy_snapshot_roundtrips_ram_and_pc() {
    // The reference emulator's `.580` file is a flat 64 KiB RAM
    // dump followed by a 13-byte trailer carrying the PC. Saving
    // and reloading must preserve every byte of RAM plus the PC,
    // and must zero everything else (registers, flags, SP, halt
    // bit, …) — the legacy format simply does not encode them.
    let mut cpu = Cpu8080State::default();
    cpu.memory.write(0x0000, 0x3E);
    cpu.memory.write(0x0001, 0x42);
    cpu.memory.write(0xFFFF, 0xAA);
    cpu.memory.write(0x4000, 0x5A);
    cpu.pc = 0xBEEF;
    // Set every other field to a non-default value so we can prove
    // the legacy reader does not pick them up by accident.
    cpu.registers.a = 0x99;
    cpu.sp = 0xC0DE;
    cpu.halted = true;

    let bytes = Snapshot580Serializer::to_legacy_bytes(&cpu);
    assert_eq!(bytes.len(), Snapshot580Serializer::LEGACY_LENGTH);
    // Trailer must end with the FF FF marker every reference file
    // we inspected carries; without it the loader rejects the file.
    assert_eq!(bytes[bytes.len() - 2], 0xFF);
    assert_eq!(bytes[bytes.len() - 1], 0xFF);

    let restored = Snapshot580Serializer::from_legacy_bytes(&bytes).unwrap();
    assert_eq!(restored.memory.read(0x0000), 0x3E);
    assert_eq!(restored.memory.read(0x0001), 0x42);
    assert_eq!(restored.memory.read(0xFFFF), 0xAA);
    assert_eq!(restored.memory.read(0x4000), 0x5A);
    assert_eq!(restored.pc, 0xBEEF);
    // Fields not encoded by the legacy format come back as defaults.
    // SP по новому дефолту равен `Cpu8080State::RESET_SP` (0xFFFF):
    // legacy не несёт регистров, поэтому загрузка должна давать
    // ровно те же дефолты, что `Cpu8080State::default()` — а тот
    // теперь ставит SP=FFFF (как школьный референс), не 0.
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

    // Wipe the FF FF marker — every reference file ends with it,
    // so a file that doesn't is not a legacy `.580`.
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

/// Найти TLV с тегом 0x08 (timing) в готовом снимке и заменить его
/// `value` на переданный `new_value`, пересчитав длину payload в
/// заголовке. Используется чтобы синтезировать «старый» 9-байтовый
/// timing-TLV из снимка, который writer выдал в новом формате.
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
/// field. This is the path a double-clicked modern `.580` file goes
/// through, so the recovered state has to match `from_bytes` exactly.
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
/// `FF FF`) must resolve to `Snapshot580Flavour::Legacy`, recover RAM
/// and PC, and leave everything else at default — exactly what
/// `from_legacy_bytes` does. This is the bug the user filed: opening
/// such a file via double-click failed with `InvalidMagic` because
/// the UI dispatched the modern decoder unconditionally.
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
    // Legacy carries no register state — verify defaults survive.
    // SP по-новому дефолту равен `Cpu8080State::RESET_SP` (0xFFFF):
    // школьный референс ставит вершину 64K, чтобы первый PUSH без
    // явного `LXI SP` не топтал низкие адреса. Раньше тест ожидал
    // 0x0000 — это был автодеривированный `Default`, и он расходился
    // со школьным эталоном уже на холодном старте.
    assert_eq!(restored.registers.a, 0);
    assert_eq!(restored.sp, Cpu8080State::RESET_SP);
    assert!(!restored.halted);
}

/// Garbage that matches neither flavour (no magic, wrong length)
/// must be rejected — otherwise the auto-detect path would happily
/// hand the modern decoder a stray binary and surface a misleading
/// `Truncated` error instead of the cleaner `InvalidMagic` the
/// existing UI diagnostics already key off.
#[test]
fn from_any_bytes_rejects_unrecognised_blob() {
    let garbage = vec![0u8; 100];
    let err = Snapshot580Serializer::from_any_bytes(&garbage).unwrap_err();
    assert!(matches!(err, SnapshotError::InvalidMagic));
}

/// A modern blob whose magic matches but whose body is corrupt must
/// not be silently misclassified as legacy — the magic check pins
/// the flavour, and the modern decoder's own error (here:
/// `PayloadLengthMismatch`) bubbles up.
#[test]
fn from_any_bytes_propagates_modern_decode_error() {
    let mut bytes = Snapshot580Serializer::to_bytes(&Cpu8080State::default());
    // Corrupt the payload length so `from_bytes` errors out, while
    // keeping the K580 magic intact.
    bytes[6] = 0xFF;
    bytes[7] = 0xFF;
    let err = Snapshot580Serializer::from_any_bytes(&bytes).unwrap_err();
    assert!(matches!(err, SnapshotError::PayloadLengthMismatch));
}

/// A 65 549-byte blob with the wrong end-of-record marker must be
/// rejected by the legacy decoder rather than silently round-trip
/// — `from_any_bytes` keeps the existing trailer-check semantics.
#[test]
fn from_any_bytes_propagates_legacy_decode_error() {
    let mut bytes = Snapshot580Serializer::to_legacy_bytes(&Cpu8080State::default());
    let last = bytes.len() - 1;
    bytes[last] = 0x00; // break the FF FF tail
    let err = Snapshot580Serializer::from_any_bytes(&bytes).unwrap_err();
    assert!(matches!(err, SnapshotError::InvalidLegacyTrailer));
}
