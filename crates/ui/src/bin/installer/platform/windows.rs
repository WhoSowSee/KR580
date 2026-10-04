mod environment;
mod system;

pub use system::{IntegrationReport, IntegrationRequest};

use k580_ui::install_mode::InstallScope;
use std::path::{Path, PathBuf};

pub fn default_system_install_dir(scope: InstallScope) -> PathBuf {
    match scope {
        InstallScope::User => std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Programs")
            .join("KR580"),
        InstallScope::Machine => std::env::var_os("ProgramFiles")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(r"C:\Program Files"))
            .join("KR580"),
    }
}

pub fn set_rounded_corners(window: &dyn iced::window::Window) {
    use iced::window::raw_window_handle::RawWindowHandle;
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::Graphics::Dwm::{
        DWM_WINDOW_CORNER_PREFERENCE, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND,
        DwmSetWindowAttribute,
    };

    let Ok(handle) = window.window_handle() else {
        return;
    };

    let RawWindowHandle::Win32(win32) = handle.as_raw() else {
        return;
    };

    let hwnd = win32.hwnd.get() as HWND;
    let value: DWM_WINDOW_CORNER_PREFERENCE = DWMWCP_ROUND;

    // SAFETY: HWND comes from winit's live window; the DWM attribute is a POD integer.
    unsafe {
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE as u32,
            std::ptr::from_ref(&value).cast(),
            std::mem::size_of::<DWM_WINDOW_CORNER_PREFERENCE>() as u32,
        );
    }
}

pub fn add_to_path(bin_dir: &Path, scope: InstallScope) -> Result<bool, String> {
    environment::add_to_path(bin_dir, scope)
}

pub(in crate::installer) fn rollback_files(scope: InstallScope, desktop: bool) -> Vec<PathBuf> {
    let mut files = vec![system::start_menu_shortcut_path(scope)];
    if desktop {
        files.push(system::desktop_shortcut_path(scope));
    }
    files
}

pub(in crate::installer) fn planned_registry_values(
    request: &crate::installer::operations::InstallRequest,
) -> Result<Vec<(String, String, winreg::RegValue)>, String> {
    use winreg::types::ToRegValue;
    let mut values = Vec::new();
    if request.add_to_path {
        let location = environment::env_key(request.scope);
        let value =
            match environment::planned_path_value(&request.install_dir.join("bin"), request.scope)?
            {
                Some(updated) => {
                    let mut value = updated.to_reg_value();
                    value.vtype = winreg::enums::REG_EXPAND_SZ;
                    value
                }
                None => winreg::RegKey::predef(location.root)
                    .open_subkey_with_flags(location.subkey, winreg::enums::KEY_QUERY_VALUE)
                    .and_then(|key| key.get_raw_value("Path"))
                    .map_err(|error| format!("capture existing PATH: {error}"))?,
            };
        values.push((location.subkey.to_owned(), "Path".to_owned(), value));
    }
    if request.mode == k580_ui::install_mode::InstallMode::System {
        let gui = request.install_dir.join("app/kr580.exe");
        let uninstaller = request.install_dir.join("app/uninstaller.exe");
        let plan = IntegrationRequest {
            scope: request.scope,
            install_dir: &request.install_dir,
            kr580_path: &gui,
            uninstaller_path: &uninstaller,
            create_desktop_shortcut: request.create_desktop_shortcut,
        };
        values.extend(system::uninstall_values(&plan).into_iter().map(|value| {
            (
                system::uninstall_key(request.scope).subkey.to_owned(),
                value.name.to_owned(),
                value.value,
            )
        }));
    }
    Ok(values)
}

pub(in crate::installer) fn notify_restored_environment() {
    environment::broadcast_environment_change();
}

pub fn remove_from_path(bin_dir: &Path, scope: InstallScope) -> Result<bool, String> {
    environment::remove_from_path(bin_dir, scope)
}

pub fn install_system_integration(
    request: &IntegrationRequest<'_>,
) -> Result<IntegrationReport, String> {
    system::install(request)
}

pub fn remove_system_integration(install_dir: &Path, scope: InstallScope) -> Result<(), String> {
    system::remove(install_dir, scope)
}

pub fn schedule_remove_install_dir(install_dir: &Path) -> Result<(), String> {
    system::schedule_remove_install_dir(install_dir)
}
