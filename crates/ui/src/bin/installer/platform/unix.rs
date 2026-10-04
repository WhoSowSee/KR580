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

pub fn remove_from_path(bin_dir: &Path, _scope: InstallScope) -> Result<bool, String> {
    let profile = profile_path();
    let existing = std::fs::read_to_string(&profile).unwrap_or_default();
    let target = bin_dir
        .to_str()
        .ok_or_else(|| "PATH target is not valid UTF-8".to_owned())?;
    if !existing.contains(&managed_path_block(target)) {
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
    let applications = applications_dir()?;
    std::fs::create_dir_all(&applications).map_err(|e| format!("create applications dir: {e}"))?;
    let desktop_file = applications.join("kr580.desktop");
    let desktop_entry = k580_ui::desktop_entry::launcher(request.kr580_path)?;
    std::fs::write(&desktop_file, &desktop_entry)
        .map_err(|e| format!("write desktop entry: {e}"))?;
    make_executable(&desktop_file)?;

    let desktop_shortcut_created = if request.create_desktop_shortcut {
        let desktop = desktop_dir();
        std::fs::create_dir_all(&desktop).map_err(|e| format!("create desktop dir: {e}"))?;
        let shortcut = desktop.join("KR580.desktop");
        std::fs::write(&shortcut, desktop_entry)
            .map_err(|e| format!("write desktop shortcut: {e}"))?;
        make_executable(&shortcut)?;
        true
    } else {
        false
    };

    k580_ui::desktop_entry::update_desktop_database(&applications)?;
    Ok(super::SystemIntegrationReport {
        desktop_shortcut_created,
    })
}

#[cfg(target_os = "macos")]
fn install_macos_integration(
    request: &super::SystemIntegrationRequest<'_>,
) -> Result<super::SystemIntegrationReport, String> {
    let app_root =
        k580_ui::macos_bundle::applications_dir()?.join(k580_ui::macos_bundle::APP_BUNDLE_NAME);
    k580_ui::macos_bundle::write_launcher_bundle(&app_root, request.kr580_path)?;

    let desktop_shortcut_created = if request.create_desktop_shortcut {
        let shortcut = desktop_dir().join("KR580.command");
        std::fs::write(&shortcut, launcher_script(request.kr580_path))
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
        let app =
            k580_ui::macos_bundle::applications_dir()?.join(k580_ui::macos_bundle::APP_BUNDLE_NAME);
        let executable = _install_dir.join("app").join("kr580");
        k580_ui::macos_bundle::remove_launcher_bundle(&app, &executable)?;
        let shortcut = desktop_dir().join("KR580.command");
        if std::fs::read_to_string(&shortcut).is_ok_and(|content| {
            content == launcher_script(&executable)
                || content == launcher_script(&executable.with_file_name("k580"))
        }) {
            remove_file_if_exists(&shortcut)?;
        }
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        let applications = applications_dir()?;
        let executable = _install_dir.join("app/kr580");
        let mut removed = false;
        for path in [
            applications.join("kr580.desktop"),
            desktop_dir().join("KR580.desktop"),
        ] {
            if std::fs::read_to_string(&path).is_ok_and(|entry| {
                [executable.clone(), executable.with_file_name("k580")]
                    .iter()
                    .any(|exe| {
                        k580_ui::desktop_entry::executable_command(exe).is_ok_and(|quoted| {
                            entry.lines().any(|line| line == format!("Exec={quoted}"))
                        })
                    })
            }) {
                remove_file_if_exists(&path)?;
                removed = true;
            }
        }
        if removed {
            k580_ui::desktop_entry::update_desktop_database(&applications)?;
        }
        Ok(())
    }
}

pub fn schedule_remove_install_dir(install_dir: &Path) -> Result<(), String> {
    let script = format!(
        "while kill -0 {} 2>/dev/null; do sleep 0.1; done; rm -rf -- {}",
        std::process::id(),
        k580_ui::shell_quote::single(&install_dir.display().to_string())
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

pub(in crate::installer) fn profile_path() -> PathBuf {
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

#[cfg(not(target_os = "macos"))]
fn applications_dir() -> Result<PathBuf, String> {
    Ok(k580_ui::desktop_entry::data_home()?.join("applications"))
}

pub(in crate::installer) fn desktop_dir() -> PathBuf {
    home_dir().join("Desktop")
}

fn managed_path_block(target: &str) -> String {
    let target = k580_ui::shell_quote::single(target);
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

#[cfg(target_os = "macos")]
use k580_ui::macos_bundle::launcher_script;

fn remove_file_if_exists(path: &Path) -> Result<(), String> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("remove {}: {error}", path.display())),
    }
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
    fn managed_block_can_be_removed() {
        let block = managed_path_block("/new/bin");
        let updated = remove_managed_block(&format!("before\n{block}after\n"));

        assert!(updated.contains("before"));
        assert!(updated.contains("after"));
        assert!(!updated.contains("KR580_BIN"));
    }
}
