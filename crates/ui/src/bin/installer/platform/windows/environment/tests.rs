use super::read_path_value;
use windows_sys::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_ALL_ACCESS, REG_OPTION_NON_VOLATILE, REG_SZ, RegCloseKey,
    RegCreateKeyExW, RegDeleteTreeW, RegSetValueExW,
};

struct RegistryFixture {
    path: Vec<u16>,
    handle: HKEY,
}

impl Drop for RegistryFixture {
    fn drop(&mut self) {
        // SAFETY: This fixture owns the handle and private test subtree; all reads have finished.
        unsafe {
            RegCloseKey(self.handle);
            RegDeleteTreeW(HKEY_CURRENT_USER, self.path.as_ptr());
        }
    }
}

#[test]
fn odd_byte_path_value_is_rejected_in_an_isolated_registry_root() {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = format!(
        "Software\\KR580\\Tests\\Path-{}-{stamp}",
        std::process::id()
    );
    let wide: Vec<_> = path.encode_utf16().chain([0]).collect();
    let mut handle = std::ptr::null_mut();
    // SAFETY: The terminated private fixture path and writable handle output remain live during creation.
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
    let fixture = RegistryFixture { path: wide, handle };
    let name: Vec<_> = "Path".encode_utf16().chain([0]).collect();
    // SAFETY: The fixture owns the key and the terminated name plus one byte cover the supplied count.
    let result =
        unsafe { RegSetValueExW(fixture.handle, name.as_ptr(), 0, REG_SZ, [65u8].as_ptr(), 1) };
    assert_eq!(result, 0);
    assert!(read_path_value(HKEY_CURRENT_USER, &path).is_err());
}
