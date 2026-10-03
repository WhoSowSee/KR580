use super::{AlignedDevMode, DEVMODEW, read_devmode};

#[test]
fn devmode_length_and_alignment_are_checked_before_native_access() {
    let header = std::mem::size_of::<DEVMODEW>();
    let mut bytes = vec![0; header + 8];
    for (size, extra, valid) in [
        (header - 1, 0, false),
        (header, 9, false),
        (header, 8, true),
    ] {
        for (offset, value) in [
            (std::mem::offset_of!(DEVMODEW, dmSize), size),
            (std::mem::offset_of!(DEVMODEW, dmDriverExtra), extra),
        ] {
            bytes[offset..offset + 2].copy_from_slice(&(value as u16).to_le_bytes());
        }
        assert_eq!(read_devmode(&bytes).is_some(), valid);
    }
    assert!(read_devmode(&bytes[..header - 1]).is_none());
    let aligned = AlignedDevMode::from_bytes(&bytes);
    assert_eq!(
        aligned
            .as_ptr()
            .align_offset(std::mem::align_of::<DEVMODEW>()),
        0
    );
    assert_eq!(aligned.into_bytes(), bytes);
}
