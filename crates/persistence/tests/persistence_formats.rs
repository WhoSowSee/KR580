use k580_core::{Cpu8080State, Flags};
use k580_persistence::{
    ExportModel, Exporters, Importers, Settings, SettingsError, SettingsStore,
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

fn unique_temp_dir() -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("k580-persistence-{nanos}"))
}
