//! Windows file-type association: register `.580` so Explorer launches
//! the emulator on double-click and shows our embedded icon. The
//! second icon resource (id `2`) baked into the `.exe` by `build.rs`
//! is what Explorer renders for any file with this extension.

pub fn register() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    let exe_str = exe
        .to_str()
        .ok_or_else(|| "executable path is not valid UTF-8".to_owned())?;

    let icon_resource = format!("{exe_str},-2");
    let open_command = format!("\"{exe_str}\" \"%1\"");

    write_string("Software\\Classes\\.580", "", "K580.Snapshot")?;
    write_string(
        "Software\\Classes\\K580.Snapshot",
        "",
        "Снимок KR580 (.580)",
    )?;
    write_string(
        "Software\\Classes\\K580.Snapshot\\DefaultIcon",
        "",
        &icon_resource,
    )?;
    write_string(
        "Software\\Classes\\K580.Snapshot\\shell\\open\\command",
        "",
        &open_command,
    )?;

    notify_shell();
    Ok(())
}

pub fn unregister() -> Result<(), String> {
    delete_tree("Software\\Classes\\.580")?;
    delete_tree("Software\\Classes\\K580.Snapshot")?;
    notify_shell();
    Ok(())
}

pub fn is_registered() -> bool {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::ERROR_SUCCESS;
    use windows_sys::Win32::System::Registry::{
        HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE, RegCloseKey, RegOpenKeyExW,
    };

    let subkey = "Software\\Classes\\.580";
    let subkey_w: Vec<u16> = OsStr::new(subkey).encode_wide().chain(Some(0)).collect();
    let mut key: HKEY = std::ptr::null_mut();
    let status = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            subkey_w.as_ptr(),
            0,
            KEY_QUERY_VALUE,
            &mut key,
        )
    };
    if status == ERROR_SUCCESS {
        unsafe { RegCloseKey(key) };
        return true;
    }
    false
}

fn write_string(subkey: &str, name: &str, value: &str) -> Result<(), String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use windows_sys::Win32::Foundation::ERROR_SUCCESS;
    use windows_sys::Win32::System::Registry::{
        HKEY, HKEY_CURRENT_USER, KEY_SET_VALUE, REG_OPTION_NON_VOLATILE, REG_SZ, RegCloseKey,
        RegCreateKeyExW, RegSetValueExW,
    };

    let subkey_w: Vec<u16> = OsStr::new(subkey).encode_wide().chain(Some(0)).collect();
    let name_w: Vec<u16> = OsStr::new(name).encode_wide().chain(Some(0)).collect();
    let value_w: Vec<u16> = OsStr::new(value).encode_wide().chain(Some(0)).collect();

    let mut key: HKEY = ptr::null_mut();
    let status = unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            subkey_w.as_ptr(),
            0,
            ptr::null_mut(),
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE,
            ptr::null_mut(),
            &mut key,
            ptr::null_mut(),
        )
    };
    if status != ERROR_SUCCESS {
        return Err(format!("RegCreateKeyExW({subkey}) failed: {status}"));
    }

    let value_bytes = (value_w.len() * std::mem::size_of::<u16>()) as u32;
    let status = unsafe {
        RegSetValueExW(
            key,
            name_w.as_ptr(),
            0,
            REG_SZ,
            value_w.as_ptr().cast(),
            value_bytes,
        )
    };
    unsafe { RegCloseKey(key) };

    if status != ERROR_SUCCESS {
        return Err(format!("RegSetValueExW({subkey}\\{name}) failed: {status}"));
    }
    Ok(())
}

fn delete_tree(subkey: &str) -> Result<(), String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
    use windows_sys::Win32::System::Registry::{HKEY_CURRENT_USER, RegDeleteTreeW};

    let subkey_w: Vec<u16> = OsStr::new(subkey).encode_wide().chain(Some(0)).collect();
    let status = unsafe { RegDeleteTreeW(HKEY_CURRENT_USER, subkey_w.as_ptr()) };
    if status == ERROR_SUCCESS || status == ERROR_FILE_NOT_FOUND {
        Ok(())
    } else {
        Err(format!("RegDeleteTreeW({subkey}) failed: {status}"))
    }
}

fn notify_shell() {
    use std::ptr;
    use windows_sys::Win32::UI::Shell::{SHCNE_ASSOCCHANGED, SHCNF_IDLIST, SHChangeNotify};

    unsafe {
        SHChangeNotify(
            SHCNE_ASSOCCHANGED as i32,
            SHCNF_IDLIST,
            ptr::null(),
            ptr::null(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_registered_follows_register_and_unregister() {
        let was_registered = is_registered();
        let _ = unregister();
        assert!(!is_registered());
        register().unwrap();
        assert!(is_registered());
        unregister().unwrap();
        assert!(!is_registered());
        if was_registered {
            register().unwrap();
        }
    }
}
