use k580_ui::install_mode::InstallScope;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use windows_sys::Win32::System::Registry::{HKEY, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};

const CREATE_NO_WINDOW: u32 = 0x08000000;

pub struct IntegrationRequest<'a> {
    pub scope: InstallScope,
    pub install_dir: &'a Path,
    pub k580_path: &'a Path,
    pub uninstaller_path: &'a Path,
    pub create_desktop_shortcut: bool,
}

pub struct IntegrationReport {
    pub desktop_shortcut_created: bool,
}

pub fn install(request: &IntegrationRequest<'_>) -> Result<IntegrationReport, String> {
    let start_menu = start_menu_shortcut_path(request.scope);
    if let Some(parent) = start_menu.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("create start menu shortcut directory: {e}"))?;
    }
    create_shortcut(&start_menu, request.k580_path, "KR580 Emulator")?;

    let desktop_shortcut_created = if request.create_desktop_shortcut {
        let desktop = desktop_shortcut_path(request.scope);
        if let Some(parent) = desktop.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("create desktop shortcut directory: {e}"))?;
        }
        create_shortcut(&desktop, request.k580_path, "KR580 Emulator")?;
        true
    } else {
        false
    };

    write_uninstall_entry(request)?;
    Ok(IntegrationReport {
        desktop_shortcut_created,
    })
}

pub fn remove(_install_dir: &Path, scope: InstallScope) -> Result<(), String> {
    remove_file_if_exists(start_menu_shortcut_path(scope))?;
    remove_file_if_exists(desktop_shortcut_path(scope))?;
    delete_uninstall_entry(scope)?;
    Ok(())
}

pub fn schedule_remove_install_dir(install_dir: &Path) -> Result<(), String> {
    let script = format!(
        "Start-Sleep -Milliseconds 900; Remove-Item -LiteralPath {} -Recurse -Force -ErrorAction SilentlyContinue",
        ps_single_quote(&install_dir.display().to_string())
    );
    powershell_command(&script)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("schedule install directory removal: {e}"))
}

fn create_shortcut(path: &Path, target: &Path, description: &str) -> Result<(), String> {
    let working_dir = target.parent().unwrap_or_else(|| Path::new("."));
    let icon_location = format!("{},0", target.display());
    let script = format!(
        "$s=(New-Object -ComObject WScript.Shell).CreateShortcut({});$s.TargetPath={};$s.WorkingDirectory={};$s.Description={};$s.IconLocation={};$s.Save()",
        ps_single_quote(&path.display().to_string()),
        ps_single_quote(&target.display().to_string()),
        ps_single_quote(&working_dir.display().to_string()),
        ps_single_quote(description),
        ps_single_quote(&icon_location),
    );
    let status = powershell_command(&script)
        .status()
        .map_err(|e| format!("create shortcut {}: {e}", path.display()))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("create shortcut {} failed", path.display()))
    }
}

fn powershell_command(script: &str) -> Command {
    let mut command = Command::new("powershell.exe");
    command
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(CREATE_NO_WINDOW);
    command
}

fn write_uninstall_entry(request: &IntegrationRequest<'_>) -> Result<(), String> {
    let key = uninstall_key(request.scope);
    let uninstall_command = format!(
        "\"{}\" --uninstall \"{}\"",
        request.uninstaller_path.display(),
        request.install_dir.display()
    );
    let icon = format!("{},0", request.uninstaller_path.display());

    write_string(key.root, key.subkey, "DisplayName", "KR580")?;
    write_string(
        key.root,
        key.subkey,
        "DisplayVersion",
        env!("CARGO_PKG_VERSION"),
    )?;
    write_string(key.root, key.subkey, "Publisher", "KR580")?;
    write_string(
        key.root,
        key.subkey,
        "InstallLocation",
        &request.install_dir.display().to_string(),
    )?;
    write_string(key.root, key.subkey, "DisplayIcon", &icon)?;
    write_string(key.root, key.subkey, "UninstallString", &uninstall_command)?;
    write_string(
        key.root,
        key.subkey,
        "QuietUninstallString",
        &uninstall_command,
    )?;
    write_dword(key.root, key.subkey, "NoModify", 1)?;
    write_dword(key.root, key.subkey, "NoRepair", 1)
}

fn delete_uninstall_entry(scope: InstallScope) -> Result<(), String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
    use windows_sys::Win32::System::Registry::RegDeleteTreeW;

    let key = uninstall_key(scope);
    let subkey_w: Vec<u16> = OsStr::new(key.subkey)
        .encode_wide()
        .chain(Some(0))
        .collect();
    // SAFETY: Registry root is a predefined handle and the subkey string is null-terminated UTF-16.
    let status = unsafe { RegDeleteTreeW(key.root, subkey_w.as_ptr()) };
    if status == ERROR_SUCCESS || status == ERROR_FILE_NOT_FOUND {
        Ok(())
    } else {
        Err(format!("delete uninstall entry failed: {status}"))
    }
}

fn write_string(root: HKEY, subkey: &str, name: &str, value: &str) -> Result<(), String> {
    write_registry_value(
        root,
        subkey,
        name,
        value,
        windows_sys::Win32::System::Registry::REG_SZ,
    )
}

fn write_dword(root: HKEY, subkey: &str, name: &str, value: u32) -> Result<(), String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::ERROR_SUCCESS;
    use windows_sys::Win32::System::Registry::{
        KEY_SET_VALUE, REG_DWORD, REG_OPTION_NON_VOLATILE, RegCloseKey, RegCreateKeyExW,
        RegSetValueExW,
    };

    let subkey_w: Vec<u16> = OsStr::new(subkey).encode_wide().chain(Some(0)).collect();
    let name_w: Vec<u16> = OsStr::new(name).encode_wide().chain(Some(0)).collect();
    let mut key: HKEY = std::ptr::null_mut();
    // SAFETY: All input strings are null-terminated UTF-16 and `key` is closed after the write.
    let status = unsafe {
        RegCreateKeyExW(
            root,
            subkey_w.as_ptr(),
            0,
            std::ptr::null_mut(),
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE,
            std::ptr::null(),
            &mut key,
            std::ptr::null_mut(),
        )
    };
    if status != ERROR_SUCCESS {
        return Err(format!("create registry key failed: {status}"));
    }
    // SAFETY: `key` is valid and `value` is a POD u32 kept alive for the call.
    let status = unsafe {
        RegSetValueExW(
            key,
            name_w.as_ptr(),
            0,
            REG_DWORD,
            std::ptr::from_ref(&value).cast(),
            std::mem::size_of::<u32>() as u32,
        )
    };
    // SAFETY: `key` was opened by `RegCreateKeyExW` above.
    unsafe { RegCloseKey(key) };
    if status == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(format!("write registry dword failed: {status}"))
    }
}

fn write_registry_value(
    root: HKEY,
    subkey: &str,
    name: &str,
    value: &str,
    value_type: u32,
) -> Result<(), String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::ERROR_SUCCESS;
    use windows_sys::Win32::System::Registry::{
        KEY_SET_VALUE, REG_OPTION_NON_VOLATILE, RegCloseKey, RegCreateKeyExW, RegSetValueExW,
    };

    let subkey_w: Vec<u16> = OsStr::new(subkey).encode_wide().chain(Some(0)).collect();
    let name_w: Vec<u16> = OsStr::new(name).encode_wide().chain(Some(0)).collect();
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
            KEY_SET_VALUE,
            std::ptr::null(),
            &mut key,
            std::ptr::null_mut(),
        )
    };
    if status != ERROR_SUCCESS {
        return Err(format!("create registry key failed: {status}"));
    }

    let value_bytes = (value_w.len() * std::mem::size_of::<u16>()) as u32;
    // SAFETY: `key` is valid and `value_w` remains alive for the duration of the registry call.
    let status = unsafe {
        RegSetValueExW(
            key,
            name_w.as_ptr(),
            0,
            value_type,
            value_w.as_ptr().cast(),
            value_bytes,
        )
    };
    // SAFETY: `key` was opened by `RegCreateKeyExW` above.
    unsafe { RegCloseKey(key) };

    if status == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(format!("write registry string failed: {status}"))
    }
}

struct UninstallKey {
    root: HKEY,
    subkey: &'static str,
}

fn uninstall_key(scope: InstallScope) -> UninstallKey {
    match scope {
        InstallScope::User => UninstallKey {
            root: HKEY_CURRENT_USER,
            subkey: r"Software\Microsoft\Windows\CurrentVersion\Uninstall\KR580",
        },
        InstallScope::Machine => UninstallKey {
            root: HKEY_LOCAL_MACHINE,
            subkey: r"Software\Microsoft\Windows\CurrentVersion\Uninstall\KR580",
        },
    }
}

fn start_menu_shortcut_path(scope: InstallScope) -> PathBuf {
    start_menu_programs_dir(scope).join("KR580.lnk")
}

fn desktop_shortcut_path(scope: InstallScope) -> PathBuf {
    desktop_dir(scope).join("KR580.lnk")
}

fn start_menu_programs_dir(scope: InstallScope) -> PathBuf {
    match scope {
        InstallScope::User => {
            env_path("APPDATA", ".").join(r"Microsoft\Windows\Start Menu\Programs")
        }
        InstallScope::Machine => env_path("ProgramData", r"C:\ProgramData")
            .join(r"Microsoft\Windows\Start Menu\Programs"),
    }
}

fn desktop_dir(scope: InstallScope) -> PathBuf {
    match scope {
        InstallScope::User => env_path("USERPROFILE", ".").join("Desktop"),
        InstallScope::Machine => env_path("PUBLIC", r"C:\Users\Public").join("Desktop"),
    }
}

fn env_path(name: &str, fallback: &str) -> PathBuf {
    std::env::var_os(name)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(fallback))
}

fn remove_file_if_exists(path: PathBuf) -> Result<(), String> {
    match std::fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("remove {}: {error}", path.display())),
    }
}

fn ps_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn powershell_quote_doubles_single_quotes() {
        assert_eq!(ps_single_quote(r"C:\Jack's\KR580"), r"'C:\Jack''s\KR580'");
    }
}
