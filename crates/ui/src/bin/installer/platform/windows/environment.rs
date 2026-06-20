use super::super::contains_path_entry;
use k580_ui::install_mode::InstallScope;
use std::path::Path;

pub fn add_to_path(bin_dir: &Path, scope: InstallScope) -> Result<bool, String> {
    let target = bin_dir
        .to_str()
        .ok_or_else(|| "PATH target is not valid UTF-8".to_owned())?;
    let key = env_key(scope);
    let current = read_path_value(key.root, key.subkey)?;
    if contains_path_entry(&current, bin_dir) {
        return Ok(false);
    }
    let trimmed = current.trim_end_matches(';');
    let updated = if trimmed.is_empty() {
        target.to_owned()
    } else {
        format!("{trimmed};{target}")
    };
    write_path_value(key.root, key.subkey, &updated)?;
    broadcast_environment_change();
    Ok(true)
}

pub fn remove_from_path(bin_dir: &Path, scope: InstallScope) -> Result<bool, String> {
    let key = env_key(scope);
    let current = read_path_value(key.root, key.subkey)?;
    if !contains_path_entry(&current, bin_dir) {
        return Ok(false);
    }
    let target = super::super::normalize_path_entry(bin_dir);
    let updated = current
        .split(';')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .filter(|entry| {
            !entry
                .trim_matches('"')
                .trim_end_matches(&['/', '\\'][..])
                .eq_ignore_ascii_case(&target)
        })
        .collect::<Vec<_>>()
        .join(";");
    write_path_value(key.root, key.subkey, &updated)?;
    broadcast_environment_change();
    Ok(true)
}

struct EnvKey {
    root: windows_sys::Win32::System::Registry::HKEY,
    subkey: &'static str,
}

fn env_key(scope: InstallScope) -> EnvKey {
    use windows_sys::Win32::System::Registry::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
    match scope {
        InstallScope::User => EnvKey {
            root: HKEY_CURRENT_USER,
            subkey: "Environment",
        },
        InstallScope::Machine => EnvKey {
            root: HKEY_LOCAL_MACHINE,
            subkey: r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment",
        },
    }
}

fn read_path_value(
    root: windows_sys::Win32::System::Registry::HKEY,
    subkey: &str,
) -> Result<String, String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
    use windows_sys::Win32::System::Registry::{
        HKEY, KEY_QUERY_VALUE, REG_EXPAND_SZ, REG_SZ, RegCloseKey, RegOpenKeyExW, RegQueryValueExW,
    };

    let subkey_w: Vec<u16> = OsStr::new(subkey).encode_wide().chain(Some(0)).collect();
    let name_w: Vec<u16> = OsStr::new("Path").encode_wide().chain(Some(0)).collect();
    let mut key: HKEY = std::ptr::null_mut();
    // SAFETY: Pointers are null-terminated UTF-16 buffers and `key` is closed on every successful open path.
    let status = unsafe { RegOpenKeyExW(root, subkey_w.as_ptr(), 0, KEY_QUERY_VALUE, &mut key) };
    if status == ERROR_FILE_NOT_FOUND {
        return Ok(String::new());
    }
    if status != ERROR_SUCCESS {
        return Err(format!("open environment key failed: {status}"));
    }

    let mut value_type = 0;
    let mut value_bytes = 0;
    // SAFETY: The opened key is valid and the first call only asks Windows for the buffer length.
    let status = unsafe {
        RegQueryValueExW(
            key,
            name_w.as_ptr(),
            std::ptr::null_mut(),
            &mut value_type,
            std::ptr::null_mut(),
            &mut value_bytes,
        )
    };
    if status == ERROR_FILE_NOT_FOUND {
        // SAFETY: `key` was opened by `RegOpenKeyExW` above.
        unsafe { RegCloseKey(key) };
        return Ok(String::new());
    }
    if status != ERROR_SUCCESS || !matches!(value_type, REG_SZ | REG_EXPAND_SZ) {
        // SAFETY: `key` was opened by `RegOpenKeyExW` above.
        unsafe { RegCloseKey(key) };
        return Err(format!("query PATH length failed: {status}"));
    }

    let mut value = vec![0u16; value_bytes as usize / std::mem::size_of::<u16>()];
    // SAFETY: `value` is sized from Windows' reported byte count for this registry value.
    let status = unsafe {
        RegQueryValueExW(
            key,
            name_w.as_ptr(),
            std::ptr::null_mut(),
            &mut value_type,
            value.as_mut_ptr().cast(),
            &mut value_bytes,
        )
    };
    // SAFETY: `key` was opened by `RegOpenKeyExW` above.
    unsafe { RegCloseKey(key) };
    if status != ERROR_SUCCESS {
        return Err(format!("query PATH failed: {status}"));
    }

    let len = value.iter().position(|ch| *ch == 0).unwrap_or(value.len());
    String::from_utf16(&value[..len]).map_err(|e| format!("decode PATH: {e}"))
}

fn write_path_value(
    root: windows_sys::Win32::System::Registry::HKEY,
    subkey: &str,
    value: &str,
) -> Result<(), String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::ERROR_SUCCESS;
    use windows_sys::Win32::System::Registry::{
        HKEY, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_EXPAND_SZ, REG_OPTION_NON_VOLATILE, RegCloseKey,
        RegCreateKeyExW, RegSetValueExW,
    };

    let subkey_w: Vec<u16> = OsStr::new(subkey).encode_wide().chain(Some(0)).collect();
    let name_w: Vec<u16> = OsStr::new("Path").encode_wide().chain(Some(0)).collect();
    let value_w: Vec<u16> = OsStr::new(value).encode_wide().chain(Some(0)).collect();
    let mut key: HKEY = std::ptr::null_mut();
    // SAFETY: All input strings are null-terminated UTF-16 and `key` is closed after the write.
    let status = unsafe {
        RegCreateKeyExW(
            root,
            subkey_w.as_ptr(),
            0,
            std::ptr::null_mut(),
            REG_OPTION_NON_VOLATILE,
            KEY_QUERY_VALUE | KEY_SET_VALUE,
            std::ptr::null(),
            &mut key,
            std::ptr::null_mut(),
        )
    };
    if status != ERROR_SUCCESS {
        return Err(format!("create environment key failed: {status}"));
    }

    let value_bytes = (value_w.len() * std::mem::size_of::<u16>()) as u32;
    // SAFETY: `key` is valid and `value_w` remains alive for the duration of the registry call.
    let status = unsafe {
        RegSetValueExW(
            key,
            name_w.as_ptr(),
            0,
            REG_EXPAND_SZ,
            value_w.as_ptr().cast(),
            value_bytes,
        )
    };
    // SAFETY: `key` was opened by `RegCreateKeyExW` above.
    unsafe { RegCloseKey(key) };

    if status == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(format!("write PATH failed: {status}"))
    }
}

fn broadcast_environment_change() {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        HWND_BROADCAST, SMTO_ABORTIFHUNG, SendMessageTimeoutW, WM_SETTINGCHANGE,
    };

    let env: Vec<u16> = OsStr::new("Environment")
        .encode_wide()
        .chain(Some(0))
        .collect();
    let mut result = 0;
    // SAFETY: The lparam points to a live null-terminated UTF-16 string for the duration of the broadcast.
    unsafe {
        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            0,
            env.as_ptr() as isize,
            SMTO_ABORTIFHUNG,
            3000,
            &mut result,
        );
    }
}
