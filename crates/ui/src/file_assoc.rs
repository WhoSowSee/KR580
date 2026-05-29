//! Windows file-type association: register `.580` so Explorer launches
//! the emulator on double-click and shows our embedded icon. The
//! second icon resource (id `2`) baked into the `.exe` by `build.rs`
//! is what Explorer renders for any file with this extension.

#[cfg(windows)]
pub(crate) fn register() -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;

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

    let _ = exe.as_os_str().encode_wide().count();
    notify_shell();
    Ok(())
}

#[cfg(windows)]
pub(crate) fn unregister() -> Result<(), String> {
    delete_tree("Software\\Classes\\.580")?;
    delete_tree("Software\\Classes\\K580.Snapshot")?;
    notify_shell();
    Ok(())
}

#[cfg(windows)]
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

#[cfg(windows)]
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

#[cfg(windows)]
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

#[cfg(not(windows))]
pub(crate) fn register() -> Result<(), String> {
    Err("file-type association is Windows-only".to_owned())
}

#[cfg(not(windows))]
pub(crate) fn unregister() -> Result<(), String> {
    Err("file-type association is Windows-only".to_owned())
}
