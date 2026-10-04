use super::*;
use std::fs;

#[test]
fn preview_skips_unchanged_bytes_and_caps_large_images() {
    let path = std::env::temp_dir().join(format!(
        "kr580-preview-{}-{:?}.kpd",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::write(&path, b"before").unwrap();
    let initial = read_image(&path, None, true).unwrap();
    assert_eq!(initial.bytes.as_deref(), Some(b"before".as_slice()));
    assert!(
        read_image(&path, Some(&initial.stamp), true)
            .unwrap()
            .bytes
            .is_none()
    );
    fs::write(&path, vec![0x5A; PREVIEW_LIMIT as usize + 1]).unwrap();
    let changed = read_image(&path, Some(&initial.stamp), true).unwrap();
    assert_eq!(changed.bytes.unwrap(), vec![0x5A; PREVIEW_LIMIT as usize]);
    fs::remove_file(path).unwrap();
}

#[test]
fn older_generation_of_same_path_does_not_replace_new_preview() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    let path = PathBuf::from("same.kpd");
    app.snapshot.devices.floppy.path = Some(path.clone());
    app.panels.floppy_image.pending = Some((2, path.clone()));
    let make_image = || ImageRead {
        stamp: FileStamp {
            path: path.clone(),
            modified: None,
            length: 1,
        },
        bytes: Some(vec![0xAA]),
    };
    app.apply_image_contents(StorageKind::Floppy, 1, path.clone(), Ok(make_image()));
    assert!(app.panels.floppy_image.contents.is_empty());
    assert!(app.panels.floppy_image.pending.is_some());
    app.apply_image_contents(StorageKind::Floppy, 2, path.clone(), Ok(make_image()));
    assert_eq!(app.panels.floppy_image.contents, [0xAA]);
}

#[test]
fn save_floppy_buffer_file_defaults_to_kpd() {
    let base = std::env::temp_dir().join(format!("kr580-buffer-{}", std::process::id()));
    let path = super::super::save_floppy_buffer_file(&base, &[b'A', 0x80]).unwrap();
    assert_eq!(path.extension().and_then(|ext| ext.to_str()), Some("kpd"));
    assert_eq!(fs::read(&path).unwrap(), [b'A', 0x80]);
    fs::remove_file(path).unwrap();
}
