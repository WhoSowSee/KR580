use k580_ui::install_mode::InstallScope;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const BEGIN_MARKER: &str = "# KR580 installer: begin";
const END_MARKER: &str = "# KR580 installer: end";

pub fn default_system_install_dir(_scope: InstallScope) -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        home_dir().join("Applications").join("KR580")
    }
    #[cfg(not(target_os = "macos"))]
    {
        home_dir().join(".local").join("share").join("kr580")
    }
}

pub fn add_to_path(bin_dir: &Path, _scope: InstallScope) -> Result<bool, String> {
    let target = bin_dir
        .to_str()
        .ok_or_else(|| "PATH target is not valid UTF-8".to_owned())?;
    let profile = profile_path();
    let existing = std::fs::read_to_string(&profile).unwrap_or_default();
    if existing.contains(target) {
        return Ok(false);
    }
    let updated = replace_managed_block(&existing, &managed_path_block(target));
    std::fs::write(&profile, updated).map_err(|e| format!("write {}: {e}", profile.display()))?;
    Ok(true)
}

pub fn remove_from_path(_bin_dir: &Path, _scope: InstallScope) -> Result<bool, String> {
    let profile = profile_path();
    let existing = std::fs::read_to_string(&profile).unwrap_or_default();
    if !existing.contains(BEGIN_MARKER) {
        return Ok(false);
    }
    let updated = remove_managed_block(&existing);
    std::fs::write(&profile, updated).map_err(|e| format!("write {}: {e}", profile.display()))?;
    Ok(true)
}

pub fn make_executable(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = std::fs::metadata(path)
        .map_err(|e| format!("metadata {}: {e}", path.display()))?
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions)
        .map_err(|e| format!("chmod {}: {e}", path.display()))
}

pub fn install_system_integration(
    request: &super::SystemIntegrationRequest<'_>,
) -> Result<super::SystemIntegrationReport, String> {
    #[cfg(target_os = "macos")]
    {
        install_macos_integration(request)
    }
    #[cfg(not(target_os = "macos"))]
    {
        install_freedesktop_integration(request)
    }
}

#[cfg(not(target_os = "macos"))]
fn install_freedesktop_integration(
    request: &super::SystemIntegrationRequest<'_>,
) -> Result<super::SystemIntegrationReport, String> {
    let applications = applications_dir();
    std::fs::create_dir_all(&applications).map_err(|e| format!("create applications dir: {e}"))?;
    let desktop_file = applications.join("kr580.desktop");
    std::fs::write(&desktop_file, desktop_entry(request.k580_path))
        .map_err(|e| format!("write desktop entry: {e}"))?;
    make_executable(&desktop_file)?;

    let desktop_shortcut_created = if request.create_desktop_shortcut {
        let desktop = desktop_dir();
        std::fs::create_dir_all(&desktop).map_err(|e| format!("create desktop dir: {e}"))?;
        let shortcut = desktop.join("KR580.desktop");
        std::fs::write(&shortcut, desktop_entry(request.k580_path))
            .map_err(|e| format!("write desktop shortcut: {e}"))?;
        make_executable(&shortcut)?;
        true
    } else {
        false
    };

    update_desktop_database();
    Ok(super::SystemIntegrationReport {
        desktop_shortcut_created,
    })
}

#[cfg(target_os = "macos")]
fn install_macos_integration(
    request: &super::SystemIntegrationRequest<'_>,
) -> Result<super::SystemIntegrationReport, String> {
    let app_root = applications_dir().join("KR580.app");
    let contents = app_root.join("Contents");
    let macos = contents.join("MacOS");
    std::fs::create_dir_all(&macos).map_err(|e| format!("create app bundle: {e}"))?;
    std::fs::write(contents.join("Info.plist"), macos_info_plist())
        .map_err(|e| format!("write Info.plist: {e}"))?;
    let launcher = macos.join("kr580-launcher");
    std::fs::write(&launcher, launcher_script(request.k580_path))
        .map_err(|e| format!("write app launcher: {e}"))?;
    make_executable(&launcher)?;

    let desktop_shortcut_created = if request.create_desktop_shortcut {
        let shortcut = desktop_dir().join("KR580.command");
        std::fs::write(&shortcut, launcher_script(request.k580_path))
            .map_err(|e| format!("write desktop launcher: {e}"))?;
        make_executable(&shortcut)?;
        true
    } else {
        false
    };

    Ok(super::SystemIntegrationReport {
        desktop_shortcut_created,
    })
}

pub fn remove_system_integration(_install_dir: &Path, _scope: InstallScope) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let _ = std::fs::remove_dir_all(applications_dir().join("KR580.app"));
        let _ = std::fs::remove_file(desktop_dir().join("KR580.command"));
        return Ok(());
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = std::fs::remove_file(applications_dir().join("kr580.desktop"));
        let _ = std::fs::remove_file(desktop_dir().join("KR580.desktop"));
        update_desktop_database();
        Ok(())
    }
}

pub fn schedule_remove_install_dir(install_dir: &Path) -> Result<(), String> {
    let script = format!(
        "sleep 1; rm -rf -- {}",
        shell_single_quote(&install_dir.display().to_string())
    );
    Command::new("sh")
        .args(["-c", &script])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("schedule install directory removal: {e}"))
}

fn profile_path() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        home_dir().join(".zprofile")
    }
    #[cfg(not(target_os = "macos"))]
    {
        home_dir().join(".profile")
    }
}

fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn applications_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        home_dir().join("Applications")
    }
    #[cfg(not(target_os = "macos"))]
    {
        home_dir().join(".local/share/applications")
    }
}

fn desktop_dir() -> PathBuf {
    home_dir().join("Desktop")
}

fn managed_path_block(target: &str) -> String {
    let target = shell_single_quote(target);
    format!(
        "{BEGIN_MARKER}\nKR580_BIN={target}\ncase \":$PATH:\" in\n  *\":$KR580_BIN:\"*) ;;\n  *) export PATH=\"$PATH:$KR580_BIN\" ;;\nesac\n{END_MARKER}\n"
    )
}

fn replace_managed_block(existing: &str, block: &str) -> String {
    let Some(begin) = existing.find(BEGIN_MARKER) else {
        let separator = if existing.is_empty() || existing.ends_with('\n') {
            ""
        } else {
            "\n"
        };
        return format!("{existing}{separator}{block}");
    };
    let Some(relative_end) = existing[begin..].find(END_MARKER) else {
        return format!("{}{}", existing.trim_end(), block);
    };
    let end = begin + relative_end + END_MARKER.len();
    format!("{}{}{}", &existing[..begin], block, &existing[end..])
}

fn remove_managed_block(existing: &str) -> String {
    let Some(begin) = existing.find(BEGIN_MARKER) else {
        return existing.to_owned();
    };
    let Some(relative_end) = existing[begin..].find(END_MARKER) else {
        return existing[..begin].trim_end().to_owned();
    };
    let end = begin + relative_end + END_MARKER.len();
    format!("{}{}", &existing[..begin], &existing[end..])
}

fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn desktop_entry(k580_path: &Path) -> String {
    format!(
        "[Desktop Entry]\n\
         Name=KR580\n\
         Comment=KR580 emulator\n\
         Exec={}\n\
         Type=Application\n\
         Terminal=false\n\
         Categories=Development;\n",
        desktop_exec_quote(&k580_path.display().to_string())
    )
}

fn desktop_exec_quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(target_os = "macos")]
fn launcher_script(k580_path: &Path) -> String {
    format!(
        "#!/bin/sh\nexec {} \"$@\"\n",
        shell_single_quote(&k580_path.display().to_string())
    )
}

#[cfg(target_os = "macos")]
fn macos_info_plist() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleExecutable</key>
  <string>kr580-launcher</string>
  <key>CFBundleIdentifier</key>
  <string>dev.kr580.emulator</string>
  <key>CFBundleName</key>
  <string>KR580</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
</dict>
</plist>
"#
}

fn update_desktop_database() {
    let _ = Command::new("update-desktop-database")
        .arg(applications_dir())
        .status();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn managed_block_replaces_old_target() {
        let old = managed_path_block("/old/bin");
        let updated = replace_managed_block(&old, &managed_path_block("/new/bin"));

        assert!(updated.contains("/new/bin"));
        assert!(!updated.contains("/old/bin"));
    }

    #[test]
    fn single_quote_escapes_shell_quote() {
        assert_eq!(shell_single_quote("/tmp/o'clock"), "'/tmp/o'\\''clock'");
    }

    #[test]
    fn managed_block_can_be_removed() {
        let block = managed_path_block("/new/bin");
        let updated = remove_managed_block(&format!("before\n{block}after\n"));

        assert!(updated.contains("before"));
        assert!(updated.contains("after"));
        assert!(!updated.contains("KR580_BIN"));
    }
}
