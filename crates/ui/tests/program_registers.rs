use k580_core::{Memory64K, Registers};
use k580_ui::persistence::{LEGACY_LENGTH, ProgramSerializer};

#[test]
fn original_register_layout_loads_and_saves_without_data_loss() {
    let dir = std::env::temp_dir().join(format!("kr580-registers-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("registers.580");
    let mut bytes = vec![0; LEGACY_LENGTH];
    bytes[0x0100] = 0x76;
    bytes[Memory64K::SIZE..].copy_from_slice(&[
        0x81, 0x12, 0x23, 0x34, 0x45, 0x56, 0x67, 0xFE, 0xFF, 0x00, 0x01, 0x12, 0xF0,
    ]);
    std::fs::write(&path, &bytes).unwrap();

    let state = ProgramSerializer::load_file(&path).unwrap();
    assert_eq!(
        state.registers,
        Registers {
            a: 0x81,
            b: 0x12,
            c: 0x23,
            d: 0x34,
            e: 0x45,
            h: 0x56,
            l: 0x67,
            w: 0xFE,
            z: 0xFF,
        }
    );
    assert_eq!(state.pc, 0x0100);
    assert_eq!(state.sp, 0xF012);
    assert_eq!(state.memory.as_slice(), &bytes[..Memory64K::SIZE]);
    ProgramSerializer::save_file(&path, &state).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), bytes);

    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(dir).unwrap();
}
