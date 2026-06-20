use k580_ui::install_mode::InstallScope;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct SystemIntegrationRequest<'a> {
    pub scope: InstallScope,
    pub install_dir: &'a Path,
    pub k580_path: &'a Path,
    pub uninstaller_path: &'a Path,
    pub create_desktop_shortcut: bool,
}

pub struct SystemIntegrationReport {
    pub desktop_shortcut_created: bool,
}

#[cfg(windows)]
mod windows;

#[cfg(unix)]
mod unix;

pub fn default_system_install_dir(scope: InstallScope) -> PathBuf {
    platform_default_system_install_dir(scope)
}

pub fn add_to_path(bin_dir: &Path, scope: InstallScope) -> Result<bool, String> {
    platform_add_to_path(bin_dir, scope)
}

pub fn remove_from_path(bin_dir: &Path, scope: InstallScope) -> Result<bool, String> {
    platform_remove_from_path(bin_dir, scope)
}

pub fn make_executable(path: &Path) -> Result<(), String> {
    platform_make_executable(path)
}

pub fn set_rounded_corners(window: &dyn iced::window::Window) {
    platform_set_rounded_corners(window)
}

pub fn open_folder(path: &Path) -> Result<(), String> {
    if !path.is_dir() {
        return Err(format!(
            "installation folder does not exist: {}",
            path.display()
        ));
    }
    spawn_os_opener(path)
}

pub fn launch_app(path: &Path) -> Result<(), String> {
    if !path.is_file() {
        return Err(format!("installed app does not exist: {}", path.display()));
    }
    Command::new(path)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("launch {}: {e}", path.display()))
}

pub fn install_system_integration(
    request: &SystemIntegrationRequest<'_>,
) -> Result<SystemIntegrationReport, String> {
    platform_install_system_integration(request)
}

pub fn remove_system_integration(install_dir: &Path, scope: InstallScope) -> Result<(), String> {
    platform_remove_system_integration(install_dir, scope)
}

pub fn schedule_remove_install_dir(install_dir: &Path) -> Result<(), String> {
    platform_schedule_remove_install_dir(install_dir)
}

#[cfg(windows)]
fn platform_default_system_install_dir(scope: InstallScope) -> PathBuf {
    windows::default_system_install_dir(scope)
}

#[cfg(unix)]
fn platform_default_system_install_dir(scope: InstallScope) -> PathBuf {
    unix::default_system_install_dir(scope)
}

#[cfg(windows)]
fn platform_add_to_path(bin_dir: &Path, scope: InstallScope) -> Result<bool, String> {
    windows::add_to_path(bin_dir, scope)
}

#[cfg(unix)]
fn platform_add_to_path(bin_dir: &Path, scope: InstallScope) -> Result<bool, String> {
    unix::add_to_path(bin_dir, scope)
}

#[cfg(windows)]
fn platform_remove_from_path(bin_dir: &Path, scope: InstallScope) -> Result<bool, String> {
    windows::remove_from_path(bin_dir, scope)
}

#[cfg(unix)]
fn platform_remove_from_path(bin_dir: &Path, scope: InstallScope) -> Result<bool, String> {
    unix::remove_from_path(bin_dir, scope)
}

#[cfg(windows)]
fn platform_make_executable(_path: &Path) -> Result<(), String> {
    Ok(())
}

#[cfg(unix)]
fn platform_make_executable(path: &Path) -> Result<(), String> {
    unix::make_executable(path)
}

#[cfg(windows)]
fn platform_set_rounded_corners(window: &dyn iced::window::Window) {
    windows::set_rounded_corners(window);
}

#[cfg(unix)]
fn platform_set_rounded_corners(_window: &dyn iced::window::Window) {}

#[cfg(windows)]
fn platform_install_system_integration(
    request: &SystemIntegrationRequest<'_>,
) -> Result<SystemIntegrationReport, String> {
    let report = windows::install_system_integration(&windows::IntegrationRequest {
        scope: request.scope,
        install_dir: request.install_dir,
        k580_path: request.k580_path,
        uninstaller_path: request.uninstaller_path,
        create_desktop_shortcut: request.create_desktop_shortcut,
    })?;
    Ok(SystemIntegrationReport {
        desktop_shortcut_created: report.desktop_shortcut_created,
    })
}

#[cfg(unix)]
fn platform_install_system_integration(
    request: &SystemIntegrationRequest<'_>,
) -> Result<SystemIntegrationReport, String> {
    unix::install_system_integration(request)
}

#[cfg(windows)]
fn platform_remove_system_integration(
    install_dir: &Path,
    scope: InstallScope,
) -> Result<(), String> {
    windows::remove_system_integration(install_dir, scope)
}

#[cfg(unix)]
fn platform_remove_system_integration(
    install_dir: &Path,
    scope: InstallScope,
) -> Result<(), String> {
    unix::remove_system_integration(install_dir, scope)
}

#[cfg(windows)]
fn platform_schedule_remove_install_dir(install_dir: &Path) -> Result<(), String> {
    windows::schedule_remove_install_dir(install_dir)
}

#[cfg(unix)]
fn platform_schedule_remove_install_dir(install_dir: &Path) -> Result<(), String> {
    unix::schedule_remove_install_dir(install_dir)
}

#[cfg(windows)]
fn spawn_os_opener(path: &Path) -> Result<(), String> {
    Command::new("explorer.exe")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("open folder {}: {e}", path.display()))
}

#[cfg(target_os = "macos")]
fn spawn_os_opener(path: &Path) -> Result<(), String> {
    Command::new("open")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("open folder {}: {e}", path.display()))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn spawn_os_opener(path: &Path) -> Result<(), String> {
    Command::new("xdg-open")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("open folder {}: {e}", path.display()))
}

pub fn normalize_path_entry(path: &Path) -> String {
    path.to_string_lossy()
        .trim_matches('"')
        .trim_end_matches(&['/', '\\'][..])
        .to_owned()
}

pub fn contains_path_entry(path_value: &str, path: &Path) -> bool {
    let wanted = normalize_path_entry(path);
    path_value
        .split(if cfg!(windows) { ';' } else { ':' })
        .map(|entry| entry.trim())
        .filter(|entry| !entry.is_empty())
        .any(|entry| entries_match(entry, &wanted))
}

fn entries_match(left: &str, right: &str) -> bool {
    let left = left.trim_matches('"').trim_end_matches(&['/', '\\'][..]);
    if cfg!(windows) {
        left.eq_ignore_ascii_case(right)
    } else {
        left == right
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_entry_check_ignores_trailing_separator() {
        let sep = if cfg!(windows) { ';' } else { ':' };
        let value = format!(
            "{}{}{}",
            Path::new("/opt/kr580/bin").display(),
            sep,
            "/usr/bin"
        );

        assert!(contains_path_entry(&value, Path::new("/opt/kr580/bin/")));
    }
}
