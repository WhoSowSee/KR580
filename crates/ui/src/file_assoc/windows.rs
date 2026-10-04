//! Windows file-type association: register `.580` and `.krs` so Explorer launches
//! the emulator on double-click and shows our embedded icon. The
//! second icon resource (id `2`) baked into the `.exe` by `build.rs`
//! is what Explorer renders for files with these extensions.

use crate::install_mode::InstallScope;
use std::path::{Path, PathBuf};
use windows_sys::Win32::System::Registry::{HKEY, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};

const PROG_ID: &str = "K580.Snapshot";
const PROG_ID_KEY: &str = "Software\\Classes\\K580.Snapshot";
const EXTENSION_KEYS: [&str; 2] = ["Software\\Classes\\.580", "Software\\Classes\\.krs"];
const OPEN_COMMAND_KEY: &str = "Software\\Classes\\K580.Snapshot\\shell\\open\\command";

pub fn register() -> Result<(), String> {
    let exe = crate::install_mode::current_integration_executable()?;
    register_for_executable(&exe, InstallScope::User)
}

pub fn register_for_executable(exe: &Path, scope: InstallScope) -> Result<(), String> {
    let exe = crate::install_mode::companion_executable_from_launcher(
        exe.to_path_buf(),
        "kr.exe",
        "kr580.exe",
    );
    let root = class_root(scope);
    write_association(root, &exe)?;
    refresh_shell();
    Ok(())
}

fn write_association(root: HKEY, exe: &Path) -> Result<(), String> {
    for value in registry_values_for_executable(exe)? {
        write_string(root, &value.subkey, &value.name, &value.value)?;
    }
    Ok(())
}

pub struct RegistryAssociationValue {
    pub subkey: String,
    pub name: String,
    pub value: String,
}

/// Returns the exact registry values written by Windows association registration.
pub fn registry_values_for_executable(exe: &Path) -> Result<Vec<RegistryAssociationValue>, String> {
    let icon_resource = icon_resource_for(exe)?;
    let open_command = open_command_for(exe)?;
    let mut values = Vec::new();
    for extension_key in EXTENSION_KEYS {
        values.push(RegistryAssociationValue {
            subkey: extension_key.into(),
            name: String::new(),
            value: PROG_ID.into(),
        });
        values.push(RegistryAssociationValue {
            subkey: format!("{extension_key}\\OpenWithProgids"),
            name: PROG_ID.into(),
            value: String::new(),
        });
    }
    for (subkey, value) in [
        (PROG_ID_KEY, "KR580".to_owned()),
        (
            "Software\\Classes\\K580.Snapshot\\DefaultIcon",
            icon_resource,
        ),
        (OPEN_COMMAND_KEY, open_command),
    ] {
        values.push(RegistryAssociationValue {
            subkey: subkey.into(),
            name: String::new(),
            value,
        });
    }
    Ok(values)
}

pub fn unregister() -> Result<(), String> {
    let exe = crate::install_mode::current_integration_executable()?;
    unregister_for_executable(&exe, InstallScope::User)
}

pub fn unregister_for_executable(exe: &Path, scope: InstallScope) -> Result<(), String> {
    let exe = crate::install_mode::companion_executable_from_launcher(
        exe.to_path_buf(),
        "kr.exe",
        "kr580.exe",
    );
    if remove_owned_association(class_root(scope), &exe)? {
        refresh_shell();
    }
    Ok(())
}

fn remove_owned_association(root: HKEY, exe: &Path) -> Result<bool, String> {
    if !command_matches(root, &open_command_for(exe)?) {
        return Ok(false);
    }
    delete_association(root)?;
    Ok(true)
}

fn delete_association(root: HKEY) -> Result<(), String> {
    for extension_key in EXTENSION_KEYS {
        if read_string(root, extension_key, "").as_deref() == Some(PROG_ID) {
            delete_value(root, extension_key, "")?;
        }
        delete_value(root, &format!("{extension_key}\\OpenWithProgids"), PROG_ID)?;
    }
    for subkey in [
        PROG_ID_KEY,
        "Software\\Classes\\K580.Snapshot\\DefaultIcon",
        OPEN_COMMAND_KEY,
    ] {
        delete_value(root, subkey, "")?;
    }
    Ok(())
}

pub fn is_registered() -> bool {
    let Ok(exe) = association_executable() else {
        return false;
    };
    let Ok(open_command) = open_command_for(&exe) else {
        return false;
    };
    association_matches(class_root(InstallScope::User), &open_command)
}

fn association_matches(root: HKEY, open_command: &str) -> bool {
    EXTENSION_KEYS.iter().all(|extension_key| {
        read_string(root, extension_key, "").as_deref() == Some(PROG_ID)
            && read_string(root, &format!("{extension_key}\\OpenWithProgids"), PROG_ID).as_deref()
                == Some("")
    }) && command_matches(root, open_command)
}

fn command_matches(root: HKEY, open_command: &str) -> bool {
    read_string(root, OPEN_COMMAND_KEY, "")
        .as_deref()
        .is_some_and(|value| value.eq_ignore_ascii_case(open_command))
}

fn association_executable() -> Result<PathBuf, String> {
    let exe = crate::install_mode::current_integration_executable()?;
    Ok(crate::install_mode::companion_executable_from_launcher(
        exe,
        "kr.exe",
        "kr580.exe",
    ))
}

fn open_command_for(exe: &Path) -> Result<String, String> {
    let exe_str = exe
        .to_str()
        .ok_or_else(|| "executable path is not valid UTF-8".to_owned())?;
    Ok(format!("\"{exe_str}\" \"%1\""))
}

fn icon_resource_for(exe: &Path) -> Result<String, String> {
    let exe_str = exe
        .to_str()
        .ok_or_else(|| "executable path is not valid UTF-8".to_owned())?;
    Ok(format!("{exe_str},-2"))
}

fn class_root(scope: InstallScope) -> HKEY {
    match scope {
        InstallScope::User => HKEY_CURRENT_USER,
        InstallScope::Machine => HKEY_LOCAL_MACHINE,
    }
}

fn read_string(root: HKEY, subkey: &str, name: &str) -> Option<String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::ERROR_SUCCESS;
    use windows_sys::Win32::System::Registry::{
        KEY_QUERY_VALUE, REG_EXPAND_SZ, REG_SZ, RegOpenKeyExW, RegQueryValueExW,
    };

    let subkey_w: Vec<u16> = OsStr::new(subkey).encode_wide().chain(Some(0)).collect();
    let name_w: Vec<u16> = OsStr::new(name).encode_wide().chain(Some(0)).collect();
    let mut key: HKEY = std::ptr::null_mut();
    // SAFETY: The predefined/fixture root and terminated subkey remain live with a writable output handle.
    let status = unsafe { RegOpenKeyExW(root, subkey_w.as_ptr(), 0, KEY_QUERY_VALUE, &mut key) };
    if status != ERROR_SUCCESS {
        return None;
    }

    let key = RegistryKey(key);
    let mut value_type = 0;
    let mut value_bytes = 0;
    // SAFETY: The opened key and terminated value name remain live; null data queries the byte capacity.
    let status = unsafe {
        RegQueryValueExW(
            key.0,
            name_w.as_ptr(),
            std::ptr::null_mut(),
            &mut value_type,
            std::ptr::null_mut(),
            &mut value_bytes,
        )
    };
    if status != ERROR_SUCCESS
        || !matches!(value_type, REG_SZ | REG_EXPAND_SZ)
        || value_bytes == 0
        || !value_bytes.is_multiple_of(2)
    {
        return None;
    }

    let mut value = vec![0u16; value_bytes as usize / std::mem::size_of::<u16>()];
    // SAFETY: The initialized UTF-16 allocation covers the even reported byte capacity; the key/name remain live.
    let status = unsafe {
        RegQueryValueExW(
            key.0,
            name_w.as_ptr(),
            std::ptr::null_mut(),
            &mut value_type,
            value.as_mut_ptr().cast(),
            &mut value_bytes,
        )
    };
    if status != ERROR_SUCCESS
        || !matches!(value_type, REG_SZ | REG_EXPAND_SZ)
        || !value_bytes.is_multiple_of(2)
    {
        return None;
    }

    let len = value.iter().position(|ch| *ch == 0).unwrap_or(value.len());
    String::from_utf16(&value[..len]).ok()
}

fn write_string(root: HKEY, subkey: &str, name: &str, value: &str) -> Result<(), String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use windows_sys::Win32::Foundation::ERROR_SUCCESS;
    use windows_sys::Win32::System::Registry::{
        KEY_SET_VALUE, REG_OPTION_NON_VOLATILE, REG_SZ, RegCreateKeyExW, RegSetValueExW,
    };

    let subkey_w: Vec<u16> = OsStr::new(subkey).encode_wide().chain(Some(0)).collect();
    let name_w: Vec<u16> = OsStr::new(name).encode_wide().chain(Some(0)).collect();
    let value_w: Vec<u16> = OsStr::new(value).encode_wide().chain(Some(0)).collect();

    let mut key: HKEY = ptr::null_mut();
    // SAFETY: The root and terminated subkey remain live and the output receives an owned registry handle.
    let status = unsafe {
        RegCreateKeyExW(
            root,
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

    let key = RegistryKey(key);
    let value_bytes = u32::try_from(value_w.len() * std::mem::size_of::<u16>())
        .map_err(|_| "registry string is too large".to_owned())?;
    // SAFETY: The owned key/name and terminated UTF-16 data remain live for the checked byte count.
    let status = unsafe {
        RegSetValueExW(
            key.0,
            name_w.as_ptr(),
            0,
            REG_SZ,
            value_w.as_ptr().cast(),
            value_bytes,
        )
    };

    if status != ERROR_SUCCESS {
        return Err(format!("RegSetValueExW({subkey}\\{name}) failed: {status}"));
    }
    Ok(())
}

#[cfg(test)]
fn delete_tree(root: HKEY, subkey: &str) -> Result<(), String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::{
        ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND, ERROR_SUCCESS,
    };
    use windows_sys::Win32::System::Registry::RegDeleteTreeW;

    let subkey_w: Vec<u16> = OsStr::new(subkey).encode_wide().chain(Some(0)).collect();
    // SAFETY: The predefined/fixture root and terminated subkey remain live for this synchronous deletion.
    let status = unsafe { RegDeleteTreeW(root, subkey_w.as_ptr()) };
    if matches!(
        status,
        ERROR_SUCCESS | ERROR_FILE_NOT_FOUND | ERROR_PATH_NOT_FOUND
    ) {
        Ok(())
    } else {
        Err(format!("RegDeleteTreeW({subkey}) failed: {status}"))
    }
}

fn delete_value(root: HKEY, subkey: &str, name: &str) -> Result<(), String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::{
        ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND, ERROR_SUCCESS,
    };
    use windows_sys::Win32::System::Registry::RegDeleteKeyValueW;

    let subkey_w: Vec<u16> = OsStr::new(subkey).encode_wide().chain(Some(0)).collect();
    let name_w: Vec<u16> = OsStr::new(name).encode_wide().chain(Some(0)).collect();
    // SAFETY: Both UTF-16 buffers are NUL-terminated and live for the duration of the call.
    let status = unsafe { RegDeleteKeyValueW(root, subkey_w.as_ptr(), name_w.as_ptr()) };
    if matches!(
        status,
        ERROR_SUCCESS | ERROR_FILE_NOT_FOUND | ERROR_PATH_NOT_FOUND
    ) {
        Ok(())
    } else {
        Err(format!(
            "RegDeleteKeyValueW({subkey}\\{name}) failed: {status}"
        ))
    }
}

/// Refreshes Explorer's association and icon caches after restoring registration values.
pub fn refresh_shell() {
    use std::ptr;
    use windows_sys::Win32::UI::Shell::{SHCNE_ASSOCCHANGED, SHCNF_IDLIST, SHChangeNotify};

    // SAFETY: ASSOCCHANGED with IDLIST permits null item pointers and retains no Rust data.
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
mod tests;

struct RegistryKey(HKEY);

impl Drop for RegistryKey {
    fn drop(&mut self) {
        // SAFETY: This private guard owns a successful open/create handle, never a predefined root.
        unsafe { windows_sys::Win32::System::Registry::RegCloseKey(self.0) };
    }
}
