use k580_core::{Cpu8080State, Memory64K};
use k580_persistence::{
    LEGACY_LENGTH, ProgramError, ProgramSerializer, Settings, SettingsError, SettingsStore,
};

#[test]
fn program_saves_and_loads_in_legacy_format() {
    let mut cpu = Cpu8080State::default();
    cpu.memory.write(0x0000, 0x3E);
    cpu.memory.write(0x0001, 0x42);
    cpu.memory.write(0x0100, 0xC3);
    cpu.memory.write(0x0101, 0x00);
    cpu.memory.write(0x0102, 0x01);
    cpu.pc = 0x0100;

    let dir = std::env::temp_dir().join(format!("k580-prg-save-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("test.580");

    ProgramSerializer::save_file(&path, &cpu).unwrap();

    let raw = std::fs::read(&path).unwrap();
    assert_eq!(raw.len(), LEGACY_LENGTH);
    assert_eq!(raw[0x0100], 0xC3);
    assert_eq!(raw[0x0102], 0x01);
    // trailer: 9 zeros, PC_LO, PC_HI, FF, FF
    assert_eq!(raw[Memory64K::SIZE + 9], 0x00);
    assert_eq!(raw[Memory64K::SIZE + 10], 0x01);
    assert_eq!(raw[LEGACY_LENGTH - 2], 0xFF);
    assert_eq!(raw[LEGACY_LENGTH - 1], 0xFF);

    let restored = ProgramSerializer::load_file(&path).unwrap();
    assert_eq!(restored.memory.read(0x0000), 0x3E);
    assert_eq!(restored.memory.read(0x0001), 0x42);
    assert_eq!(restored.memory.read(0x0100), 0xC3);
    assert_eq!(restored.memory.read(0x0102), 0x01);
    assert_eq!(restored.pc, 0x0100);

    std::fs::remove_file(&path).ok();
    std::fs::remove_dir(dir).ok();
}

#[test]
fn program_save_empty_state_writes_full_legacy_format() {
    let cpu = Cpu8080State::default();
    let dir = std::env::temp_dir().join(format!("k580-prg-empty-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("empty.580");

    ProgramSerializer::save_file(&path, &cpu).unwrap();
    let raw = std::fs::read(&path).unwrap();
    assert_eq!(raw.len(), LEGACY_LENGTH);
    assert_eq!(raw[LEGACY_LENGTH - 2], 0xFF);
    assert_eq!(raw[LEGACY_LENGTH - 1], 0xFF);

    std::fs::remove_file(&path).ok();
    std::fs::remove_dir(dir).ok();
}

#[test]
fn program_load_rejects_wrong_extension() {
    let dir = std::env::temp_dir().join(format!("k580-prg-ext-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("test.bin");
    std::fs::write(&path, [0x3E, 0x42]).unwrap();

    let err = ProgramSerializer::load_file(&path).unwrap_err();
    assert!(matches!(err, ProgramError::NotA580File));

    std::fs::remove_file(&path).ok();
    std::fs::remove_dir(dir).ok();
}

#[test]
fn program_load_rejects_empty_file() {
    let dir = std::env::temp_dir().join(format!("k580-prg-emptyload-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("empty.580");
    std::fs::write(&path, []).unwrap();

    let err = ProgramSerializer::load_file(&path).unwrap_err();
    assert!(matches!(err, ProgramError::EmptyFile));

    std::fs::remove_file(&path).ok();
    std::fs::remove_dir(dir).ok();
}

#[test]
fn program_load_rejects_oversized_file() {
    let dir = std::env::temp_dir().join(format!("k580-prg-big-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("big.580");
    let big = vec![0u8; LEGACY_LENGTH + 1];
    std::fs::write(&path, big).unwrap();

    let err = ProgramSerializer::load_file(&path).unwrap_err();
    assert!(matches!(err, ProgramError::WrongSize { .. }));

    std::fs::remove_file(&path).ok();
    std::fs::remove_dir(dir).ok();
}

#[test]
fn program_loads_legacy_format_with_pc_from_trailer() {
    let dir = std::env::temp_dir().join(format!("k580-prg-legacy-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("legacy.580");
    let mut bytes = vec![0u8; LEGACY_LENGTH];
    bytes[0x0100] = 0xC3;
    bytes[0x0101] = 0x00;
    bytes[0x0102] = 0x01;
    // PC = 0x1234 at trailer[9..11]
    bytes[Memory64K::SIZE + 9] = 0x34;
    bytes[Memory64K::SIZE + 10] = 0x12;
    bytes[LEGACY_LENGTH - 2] = 0xFF;
    bytes[LEGACY_LENGTH - 1] = 0xFF;
    std::fs::write(&path, &bytes).unwrap();

    let restored = ProgramSerializer::load_file(&path).unwrap();
    assert_eq!(restored.memory.read(0x0100), 0xC3);
    assert_eq!(restored.pc, 0x1234);

    std::fs::remove_file(&path).ok();
    std::fs::remove_dir(dir).ok();
}

#[test]
fn program_load_rejects_legacy_format_with_bad_trailer() {
    let dir = std::env::temp_dir().join(format!("k580-prg-badtrail-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("badtrail.580");
    let mut bytes = vec![0u8; LEGACY_LENGTH];
    bytes[LEGACY_LENGTH - 2] = 0x00;
    bytes[LEGACY_LENGTH - 1] = 0x00;
    std::fs::write(&path, &bytes).unwrap();

    let err = ProgramSerializer::load_file(&path).unwrap_err();
    assert!(matches!(err, ProgramError::InvalidLegacyTrailer));

    std::fs::remove_file(&path).ok();
    std::fs::remove_dir(dir).ok();
}

#[test]
fn program_load_rejects_raw_code_without_trailer() {
    let dir = std::env::temp_dir().join(format!("k580-prg-raw-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("raw.580");
    let raw = vec![0x76u8; Memory64K::SIZE];
    std::fs::write(&path, &raw).unwrap();

    let err = ProgramSerializer::load_file(&path).unwrap_err();
    assert!(matches!(err, ProgramError::WrongSize { .. }));

    std::fs::remove_file(&path).ok();
    std::fs::remove_dir(dir).ok();
}

#[test]
fn program_bytes_are_deterministic() {
    let mut cpu = Cpu8080State::default();
    cpu.memory.write(0x0100, 0xC3);
    cpu.memory.write(0x0101, 0x00);
    cpu.memory.write(0x0102, 0x10);
    cpu.pc = 0x0100;

    let dir = std::env::temp_dir().join(format!("k580-prg-det-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("det.580");

    ProgramSerializer::save_file(&path, &cpu).unwrap();
    let first = std::fs::read(&path).unwrap();
    ProgramSerializer::save_file(&path, &cpu).unwrap();
    let second = std::fs::read(&path).unwrap();
    assert_eq!(first.len(), LEGACY_LENGTH);
    assert_eq!(first, second);

    std::fs::remove_file(&path).ok();
    std::fs::remove_dir(dir).ok();
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
