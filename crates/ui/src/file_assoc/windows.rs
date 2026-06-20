//! Windows file-type association: register `.580` so Explorer launches
//! the emulator on double-click and shows our embedded icon. The
//! second icon resource (id `2`) baked into the `.exe` by `build.rs`
//! is what Explorer renders for any file with this extension.

use crate::install_mode::InstallScope;
use std::path::{Path, PathBuf};
use windows_sys::Win32::System::Registry::{HKEY, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};

pub fn register() -> Result<(), String> {
    let exe = association_executable()?;
    register_for_executable(&exe, InstallScope::User)
}

pub fn register_for_executable(exe: &Path, scope: InstallScope) -> Result<(), String> {
    let exe = association_executable_from(exe.to_path_buf());
    let icon_resource = icon_resource_for(&exe)?;
    let open_command = open_command_for(&exe)?;
    let root = class_root(scope);

    write_string(root, "Software\\Classes\\.580", "", "K580.Snapshot")?;
    write_string(
        root,
        "Software\\Classes\\K580.Snapshot",
        "",
        "Снимок KR580 (.580)",
    )?;
    write_string(
        root,
        "Software\\Classes\\K580.Snapshot\\DefaultIcon",
        "",
        &icon_resource,
    )?;
    write_string(
        root,
        "Software\\Classes\\K580.Snapshot\\shell\\open\\command",
        "",
        &open_command,
    )?;

    notify_shell();
    Ok(())
}

pub fn unregister() -> Result<(), String> {
    delete_association(class_root(InstallScope::User))
}

pub fn unregister_for_executable(exe: &Path, scope: InstallScope) -> Result<(), String> {
    let exe = association_executable_from(exe.to_path_buf());
    if is_registered_for_executable(&exe, scope) {
        delete_association(class_root(scope))?;
    }
    Ok(())
}

fn delete_association(root: HKEY) -> Result<(), String> {
    delete_tree(root, "Software\\Classes\\.580")?;
    delete_tree(root, "Software\\Classes\\K580.Snapshot")?;
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

fn is_registered_for_executable(exe: &Path, scope: InstallScope) -> bool {
    let Ok(open_command) = open_command_for(exe) else {
        return false;
    };
    association_matches(class_root(scope), &open_command)
}

fn association_matches(root: HKEY, open_command: &str) -> bool {
    let extension = read_string(root, "Software\\Classes\\.580", "");
    let command = read_string(
        root,
        "Software\\Classes\\K580.Snapshot\\shell\\open\\command",
        "",
    );

    extension.as_deref() == Some("K580.Snapshot")
        && command
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
    use windows_sys::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
    use windows_sys::Win32::System::Registry::RegDeleteTreeW;

    let subkey_w: Vec<u16> = OsStr::new(subkey).encode_wide().chain(Some(0)).collect();
    let status = unsafe { RegDeleteTreeW(root, subkey_w.as_ptr()) };
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
    use std::path::PathBuf;

    #[test]
    fn file_association_registered_from_launcher_points_to_gui_binary() {
        assert_eq!(
            association_executable_from(PathBuf::from(r"D:\kr-580\target\release\kr.exe")),
            PathBuf::from(r"D:\kr-580\target\release\k580.exe")
        );
    }

    #[test]
    fn file_association_registered_from_gui_keeps_gui_binary() {
        assert_eq!(
            association_executable_from(PathBuf::from(r"D:\kr-580\target\release\k580.exe")),
            PathBuf::from(r"D:\kr-580\target\release\k580.exe")
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

    #[test]
    #[ignore = "mutates HKCU file association"]
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
