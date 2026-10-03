use super::*;
use windows_sys::Win32::System::Registry::{
    KEY_ALL_ACCESS, REG_OPTION_NON_VOLATILE, RegCloseKey, RegCreateKeyExW,
};

struct RegistryRoot {
    path: String,
    handle: HKEY,
}

impl RegistryRoot {
    fn new() -> Self {
        static NEXT_ROOT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let nonce = NEXT_ROOT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let path = format!("Software\\KR580\\Tests\\{}-{nonce}", std::process::id());
        let wide: Vec<_> = path.encode_utf16().chain([0]).collect();
        let mut handle = std::ptr::null_mut();
        // SAFETY: Valid root, terminated key name and a live output handle pointer.
        let result = unsafe {
            RegCreateKeyExW(
                HKEY_CURRENT_USER,
                wide.as_ptr(),
                0,
                std::ptr::null_mut(),
                REG_OPTION_NON_VOLATILE,
                KEY_ALL_ACCESS,
                std::ptr::null(),
                &mut handle,
                std::ptr::null_mut(),
            )
        };
        assert_eq!(result, 0);
        Self { path, handle }
    }
}

impl Drop for RegistryRoot {
    fn drop(&mut self) {
        // SAFETY: The handle is owned by this fixture and closed exactly once.
        unsafe { RegCloseKey(self.handle) };
        let _ = delete_tree(HKEY_CURRENT_USER, &self.path);
    }
}

#[test]
fn association_roundtrip_preserves_foreign_defaults_and_open_with_entries() {
    let root = RegistryRoot::new();
    let executable = Path::new(r"C:\Program Files\KR580\app\kr580.exe");
    write_association(root.handle, executable).unwrap();
    assert_eq!(
        read_string(root.handle, OPEN_COMMAND_KEY, "").as_deref(),
        Some(r#""C:\Program Files\KR580\app\kr580.exe" "%1""#)
    );
    assert!(association_matches(
        root.handle,
        &open_command_for(executable).unwrap()
    ));
    for extension in EXTENSION_KEYS {
        write_string(root.handle, extension, "", "Foreign.App").unwrap();
        write_string(
            root.handle,
            &format!("{extension}\\OpenWithProgids"),
            "Foreign.App",
            "",
        )
        .unwrap();
    }
    assert!(remove_owned_association(root.handle, executable).unwrap());
    for extension in EXTENSION_KEYS {
        assert_eq!(
            read_string(root.handle, extension, "").as_deref(),
            Some("Foreign.App")
        );
        assert_eq!(
            read_string(
                root.handle,
                &format!("{extension}\\OpenWithProgids"),
                "Foreign.App"
            )
            .as_deref(),
            Some("")
        );
        assert!(
            read_string(
                root.handle,
                &format!("{extension}\\OpenWithProgids"),
                PROG_ID
            )
            .is_none()
        );
    }
    assert!(read_string(root.handle, PROG_ID_KEY, "").is_none());
}

#[test]
fn one_installation_cannot_unregister_another() {
    let root = RegistryRoot::new();
    let owner = Path::new(r"C:\Programs\new\kr580.exe");
    write_association(root.handle, owner).unwrap();
    assert!(
        !remove_owned_association(root.handle, Path::new(r"C:\Programs\old\kr580.exe")).unwrap()
    );
    assert!(association_matches(
        root.handle,
        &open_command_for(owner).unwrap()
    ));
    assert!(remove_owned_association(root.handle, owner).unwrap());
    for extension in EXTENSION_KEYS {
        assert!(read_string(root.handle, extension, "").is_none());
    }
}
