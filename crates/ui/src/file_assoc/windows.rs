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
    let exe = association_executable()?;
    register_for_executable(&exe, InstallScope::User)
}

pub fn register_for_executable(exe: &Path, scope: InstallScope) -> Result<(), String> {
    let exe = association_executable_from(exe.to_path_buf());
    let icon_resource = icon_resource_for(&exe)?;
    let open_command = open_command_for(&exe)?;
    let root = class_root(scope);

    for extension_key in EXTENSION_KEYS {
        write_string(root, extension_key, "", PROG_ID)?;
        write_string(
            root,
            &format!("{extension_key}\\OpenWithProgids"),
            PROG_ID,
            "",
        )?;
    }
    write_string(root, PROG_ID_KEY, "", "Файл KR580 (.580, .krs)")?;
    write_string(
        root,
        "Software\\Classes\\K580.Snapshot\\DefaultIcon",
        "",
        &icon_resource,
    )?;
    write_string(root, OPEN_COMMAND_KEY, "", &open_command)?;

    notify_shell();
    Ok(())
}

pub fn unregister() -> Result<(), String> {
    let exe = association_executable()?;
    unregister_for_executable(&exe, InstallScope::User)
}

pub fn unregister_for_executable(exe: &Path, scope: InstallScope) -> Result<(), String> {
    let exe = association_executable_from(exe.to_path_buf());
    let Ok(open_command) = open_command_for(&exe) else {
        return Ok(());
    };
    if command_matches(class_root(scope), &open_command) {
        delete_association(class_root(scope))?;
    }
    Ok(())
}

fn delete_association(root: HKEY) -> Result<(), String> {
    for extension_key in EXTENSION_KEYS {
        if read_string(root, extension_key, "").as_deref() == Some(PROG_ID) {
            delete_value(root, extension_key, "")?;
        }
        delete_value(root, &format!("{extension_key}\\OpenWithProgids"), PROG_ID)?;
    }
    delete_tree(root, PROG_ID_KEY)?;
    notify_shell();
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
    let exe = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    Ok(association_executable_from(exe))
}

fn association_executable_from(exe: PathBuf) -> PathBuf {
    if exe
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("kr.exe"))
    {
        if let Some(directory) = exe.parent()
            && directory
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.eq_ignore_ascii_case("bin"))
            && let Some(root) = directory.parent()
        {
            return root.join("app").join("k580.exe");
        }
        return exe.with_file_name("k580.exe");
    }
    exe
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
        KEY_QUERY_VALUE, REG_EXPAND_SZ, REG_SZ, RegCloseKey, RegOpenKeyExW, RegQueryValueExW,
    };

    let subkey_w: Vec<u16> = OsStr::new(subkey).encode_wide().chain(Some(0)).collect();
    let name_w: Vec<u16> = OsStr::new(name).encode_wide().chain(Some(0)).collect();
    let mut key: HKEY = std::ptr::null_mut();
    let status = unsafe { RegOpenKeyExW(root, subkey_w.as_ptr(), 0, KEY_QUERY_VALUE, &mut key) };
    if status != ERROR_SUCCESS {
        return None;
    }

    let mut value_type = 0;
    let mut value_bytes = 0;
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
    if status != ERROR_SUCCESS || !matches!(value_type, REG_SZ | REG_EXPAND_SZ) || value_bytes == 0
    {
        unsafe { RegCloseKey(key) };
        return None;
    }

    let mut value = vec![0u16; value_bytes as usize / std::mem::size_of::<u16>()];
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
    unsafe { RegCloseKey(key) };
    if status != ERROR_SUCCESS {
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
        KEY_SET_VALUE, REG_OPTION_NON_VOLATILE, REG_SZ, RegCloseKey, RegCreateKeyExW,
        RegSetValueExW,
    };

    let subkey_w: Vec<u16> = OsStr::new(subkey).encode_wide().chain(Some(0)).collect();
    let name_w: Vec<u16> = OsStr::new(name).encode_wide().chain(Some(0)).collect();
    let value_w: Vec<u16> = OsStr::new(value).encode_wide().chain(Some(0)).collect();

    let mut key: HKEY = ptr::null_mut();
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

fn delete_tree(root: HKEY, subkey: &str) -> Result<(), String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::{
        ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND, ERROR_SUCCESS,
    };
    use windows_sys::Win32::System::Registry::RegDeleteTreeW;

    let subkey_w: Vec<u16> = OsStr::new(subkey).encode_wide().chain(Some(0)).collect();
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
    use std::path::PathBuf;

    #[test]
    fn file_association_registered_from_launcher_points_to_gui_binary() {
        assert_eq!(
            association_executable_from(PathBuf::from(r"D:\kr-580\target\release\kr.exe")),
            PathBuf::from(r"D:\kr-580\target\release\k580.exe")
        );
    }

    #[test]
    fn installed_launcher_points_to_app_gui_binary() {
        assert_eq!(
            association_executable_from(PathBuf::from(r"C:\Programs\KR580\bin\kr.exe")),
            PathBuf::from(r"C:\Programs\KR580\app\k580.exe")
        );
    }

    #[test]
    fn open_command_registered_from_launcher_uses_gui_binary() {
        let exe = association_executable_from(PathBuf::from(r"D:\kr-580\target\release\kr.exe"));
        assert_eq!(
            open_command_for(&exe).unwrap(),
            r#""D:\kr-580\target\release\k580.exe" "%1""#
        );
    }
}
