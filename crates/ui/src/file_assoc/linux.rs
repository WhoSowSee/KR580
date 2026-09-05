use std::path::{Path, PathBuf};

const HANDLER_DESKTOP_FILE: &str = "kr580-file-handler.desktop";

pub fn register() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    register_for_executable(&exe, crate::install_mode::InstallScope::User)
}

pub fn register_for_executable(
    exe: &Path,
    _scope: crate::install_mode::InstallScope,
) -> Result<(), String> {
    let exe = association_executable_from(exe.to_path_buf());
    let exe_str = exe
        .to_str()
        .ok_or_else(|| "executable path is not valid UTF-8".to_owned())?;
    let mime_dir = mime_dir();
    let apps_dir = apps_dir();
    let hicolor_dir = hicolor_icon_dir();

    std::fs::create_dir_all(&mime_dir).map_err(|e| format!("create mime dir: {e}"))?;
    std::fs::create_dir_all(&apps_dir).map_err(|e| format!("create apps dir: {e}"))?;
    std::fs::create_dir_all(&hicolor_dir).map_err(|e| format!("create icons dir: {e}"))?;

    let mime_file = mime_dir.join("application-x-kr580.xml");
    let desktop_file = apps_dir.join(HANDLER_DESKTOP_FILE);
    let dest_icon = hicolor_dir.join("kr580.png");

    std::fs::write(&mime_file, mime_xml()).map_err(|e| format!("write mime file: {e}"))?;
    std::fs::write(&desktop_file, desktop_entry(exe_str))
        .map_err(|e| format!("write desktop file: {e}"))?;
    if let Some(icon) = super::find_icon() {
        std::fs::copy(&icon, &dest_icon).map_err(|e| format!("copy icon: {e}"))?;
    }

    update_databases();
    Ok(())
}

pub fn unregister() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    unregister_for_executable(&exe, crate::install_mode::InstallScope::User)
}

fn remove_registration() -> Result<(), String> {
    let _ = std::fs::remove_file(mime_dir().join("application-x-kr580.xml"));
    let _ = std::fs::remove_file(apps_dir().join(HANDLER_DESKTOP_FILE));
    let _ = std::fs::remove_file(hicolor_icon_dir().join("kr580.png"));
    update_databases();
    Ok(())
}

pub fn unregister_for_executable(
    exe: &Path,
    _scope: crate::install_mode::InstallScope,
) -> Result<(), String> {
    let exe = association_executable_from(exe.to_path_buf());
    let exe_str = exe
        .to_str()
        .ok_or_else(|| "executable path is not valid UTF-8".to_owned())?;
    let desktop_file = apps_dir().join(HANDLER_DESKTOP_FILE);
    let current = std::fs::read_to_string(&desktop_file).unwrap_or_default();
    if desktop_entry_owned_by(&current, exe_str) {
        remove_registration()?;
    }
    Ok(())
}

pub fn is_registered() -> bool {
    let Ok(exe) = std::env::current_exe() else {
        return false;
    };
    let exe = association_executable_from(exe);
    let Some(exe_str) = exe.to_str() else {
        return false;
    };
    let Ok(mime) = std::fs::read_to_string(mime_dir().join("application-x-kr580.xml")) else {
        return false;
    };
    let Ok(desktop) = std::fs::read_to_string(apps_dir().join(HANDLER_DESKTOP_FILE)) else {
        return false;
    };
    mime_registration_is_current(&mime) && desktop_entry_owned_by(&desktop, exe_str)
}

fn association_executable_from(exe: PathBuf) -> PathBuf {
    if exe.file_name().is_some_and(|name| name == "kr") {
        if let Some(directory) = exe.parent()
            && directory.file_name().is_some_and(|name| name == "bin")
            && let Some(root) = directory.parent()
            && root.join(crate::install_mode::MANIFEST_FILENAME).is_file()
        {
            return root.join("app").join("k580");
        }
        return exe.with_file_name("k580");
    }
    exe
}

fn desktop_entry_owned_by(entry: &str, exe: &str) -> bool {
    let expected = format!("Exec={exe} %f");
    entry.lines().any(|line| line == expected)
}

fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn mime_dir() -> PathBuf {
    home_dir().join(".local/share/mime/packages")
}

fn apps_dir() -> PathBuf {
    home_dir().join(".local/share/applications")
}

fn hicolor_icon_dir() -> PathBuf {
    home_dir().join(".local/share/icons/hicolor/64x64/apps")
}

fn mime_xml() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<mime-info xmlns="http://www.freedesktop.org/standards/shared-mime-info">
  <mime-type type="application/x-kr580">
    <comment>KR580 program file</comment>
    <glob pattern="*.580"/>
    <glob pattern="*.krs"/>
  </mime-type>
</mime-info>
"#
}

fn mime_registration_is_current(mime: &str) -> bool {
    mime.contains(r#"<glob pattern="*.580"/>"#) && mime.contains(r#"<glob pattern="*.krs"/>"#)
}

fn desktop_entry(exec: &str) -> String {
    format!(
        "[Desktop Entry]\n\
         Name=KR580 Emulator\n\
         Comment=KR580 emulator\n\
         Exec={} %f\n\
         Icon=kr580\n\
         Type=Application\n\
         NoDisplay=true\n\
         Terminal=false\n\
         MimeType=application/x-kr580;\n\
         Categories=Development;\n",
        exec
    )
}

fn update_databases() {
    let _ = std::process::Command::new("update-mime-database")
        .arg(home_dir().join(".local/share/mime"))
        .status();
    let _ = std::process::Command::new("update-desktop-database")
        .arg(apps_dir())
        .status();
}

#[cfg(test)]
mod tests {
    use super::{
        association_executable_from, desktop_entry, desktop_entry_owned_by,
        mime_registration_is_current, mime_xml,
    };
    use std::path::PathBuf;

    #[test]
    fn mime_registration_covers_snapshots_and_subprograms() {
        assert!(mime_registration_is_current(mime_xml()));
        assert!(!mime_registration_is_current(r#"<glob pattern="*.580"/>"#));
    }

    #[test]
    fn file_handler_is_hidden_from_application_menus() {
        let entry = desktop_entry("/opt/kr580/k580");

        assert!(entry.contains("NoDisplay=true\n"));
        assert!(entry.contains("MimeType=application/x-kr580;\n"));
    }

    #[test]
    fn ownership_requires_the_exact_executable() {
        let entry = desktop_entry("/opt/kr580/app/k580");

        assert!(desktop_entry_owned_by(&entry, "/opt/kr580/app/k580"));
        assert!(!desktop_entry_owned_by(&entry, "/opt/kr580/app/k58"));
    }

    #[test]
    fn adjacent_launcher_resolves_to_gui() {
        assert_eq!(
            association_executable_from(PathBuf::from("/opt/kr580/kr")),
            PathBuf::from("/opt/kr580/k580")
        );
    }
}
