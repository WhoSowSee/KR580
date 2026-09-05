use std::ffi::OsString;
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
    let paths = IntegrationPaths::current()?;

    std::fs::create_dir_all(&paths.mime_packages).map_err(|e| format!("create mime dir: {e}"))?;
    std::fs::create_dir_all(&paths.applications).map_err(|e| format!("create apps dir: {e}"))?;
    std::fs::create_dir_all(&paths.application_icons)
        .map_err(|e| format!("create icons dir: {e}"))?;

    let mime_file = paths.mime_packages.join("application-x-kr580.xml");
    let desktop_file = paths.applications.join(HANDLER_DESKTOP_FILE);
    let dest_icon = paths.application_icons.join("kr580.png");

    std::fs::write(&mime_file, mime_xml()).map_err(|e| format!("write mime file: {e}"))?;
    std::fs::write(&desktop_file, desktop_entry(&exe)?)
        .map_err(|e| format!("write desktop file: {e}"))?;
    if let Some(icon) = super::find_icon() {
        std::fs::copy(&icon, &dest_icon).map_err(|e| format!("copy icon: {e}"))?;
    }

    update_databases(&paths);
    Ok(())
}

pub fn unregister() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    unregister_for_executable(&exe, crate::install_mode::InstallScope::User)
}

fn remove_registration(paths: &IntegrationPaths) -> Result<(), String> {
    let _ = std::fs::remove_file(paths.mime_packages.join("application-x-kr580.xml"));
    let _ = std::fs::remove_file(paths.applications.join(HANDLER_DESKTOP_FILE));
    let _ = std::fs::remove_file(paths.application_icons.join("kr580.png"));
    update_databases(&paths);
    Ok(())
}

pub fn unregister_for_executable(
    exe: &Path,
    _scope: crate::install_mode::InstallScope,
) -> Result<(), String> {
    let exe = association_executable_from(exe.to_path_buf());
    let paths = IntegrationPaths::current()?;
    let desktop_file = paths.applications.join(HANDLER_DESKTOP_FILE);
    let current = std::fs::read_to_string(&desktop_file).unwrap_or_default();
    if desktop_entry_owned_by(&current, &exe) {
        remove_registration(&paths)?;
    }
    Ok(())
}

pub fn is_registered() -> bool {
    let Ok(exe) = std::env::current_exe() else {
        return false;
    };
    let exe = association_executable_from(exe);
    let Ok(paths) = IntegrationPaths::current() else {
        return false;
    };
    let Ok(mime) = std::fs::read_to_string(paths.mime_packages.join("application-x-kr580.xml"))
    else {
        return false;
    };
    let Ok(desktop) = std::fs::read_to_string(paths.applications.join(HANDLER_DESKTOP_FILE)) else {
        return false;
    };
    mime_registration_is_current(&mime) && desktop_entry_owned_by(&desktop, &exe)
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

fn desktop_entry_owned_by(entry: &str, exe: &Path) -> bool {
    crate::desktop_entry::quote_executable(exe)
        .map(|exe| entry.lines().any(|line| line == format!("Exec={exe} %f")))
        .unwrap_or(false)
}

struct IntegrationPaths {
    data_home: PathBuf,
    mime_packages: PathBuf,
    applications: PathBuf,
    application_icons: PathBuf,
}

impl IntegrationPaths {
    fn current() -> Result<Self, String> {
        let data_home =
            data_home_from(std::env::var_os("XDG_DATA_HOME"), std::env::var_os("HOME"))?;
        Ok(Self {
            mime_packages: data_home.join("mime/packages"),
            applications: data_home.join("applications"),
            application_icons: data_home.join("icons/hicolor/64x64/apps"),
            data_home,
        })
    }
}

fn data_home_from(
    xdg_data_home: Option<OsString>,
    home: Option<OsString>,
) -> Result<PathBuf, String> {
    if let Some(value) = xdg_data_home.filter(|value| !value.is_empty()) {
        let path = PathBuf::from(value);
        if path.is_absolute() {
            return Ok(path);
        }
    }

    let home = home
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .ok_or_else(|| "HOME is missing or is not an absolute path".to_owned())?;
    Ok(home.join(".local/share"))
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

fn desktop_entry(executable: &Path) -> Result<String, String> {
    Ok(format!(
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
        crate::desktop_entry::quote_executable(executable)?
    ))
}

fn update_databases(paths: &IntegrationPaths) {
    let _ = std::process::Command::new("update-mime-database")
        .arg(paths.data_home.join("mime"))
        .status();
    let _ = std::process::Command::new("update-desktop-database")
        .arg(&paths.applications)
        .status();
}

#[cfg(test)]
mod tests {
    use super::{
        association_executable_from, data_home_from, desktop_entry, desktop_entry_owned_by,
        mime_registration_is_current, mime_xml,
    };
    use std::ffi::OsString;
    use std::path::PathBuf;

    #[test]
    fn mime_registration_covers_snapshots_and_subprograms() {
        assert!(mime_registration_is_current(mime_xml()));
        assert!(!mime_registration_is_current(r#"<glob pattern="*.580"/>"#));
    }

    #[test]
    fn file_handler_is_hidden_from_application_menus() {
        let executable = PathBuf::from("/opt/kr580/k580");
        let entry = desktop_entry(&executable).unwrap();

        assert!(entry.contains("NoDisplay=true\n"));
        assert!(entry.contains("MimeType=application/x-kr580;\n"));
    }

    #[test]
    fn ownership_requires_the_exact_executable() {
        let executable = PathBuf::from("/opt/kr580/app/k580");
        let entry = desktop_entry(&executable).unwrap();

        assert!(desktop_entry_owned_by(&entry, &executable));
        assert!(!desktop_entry_owned_by(
            &entry,
            PathBuf::from("/opt/kr580/app/k58").as_path()
        ));
    }

    #[test]
    fn adjacent_launcher_resolves_to_gui() {
        assert_eq!(
            association_executable_from(PathBuf::from("/opt/kr580/kr")),
            PathBuf::from("/opt/kr580/k580")
        );
    }

    #[test]
    fn xdg_data_home_uses_absolute_override_or_home_fallback() {
        assert_eq!(
            data_home_from(Some(OsString::from("/var/lib/kr580")), None).unwrap(),
            PathBuf::from("/var/lib/kr580")
        );
        assert_eq!(
            data_home_from(
                Some(OsString::from("relative")),
                Some(OsString::from("/home/user"))
            )
            .unwrap(),
            PathBuf::from("/home/user/.local/share")
        );
        assert!(data_home_from(None, None).is_err());
    }
}
