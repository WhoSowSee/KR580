use std::ffi::OsString;
use std::path::{Path, PathBuf};

const HANDLER_DESKTOP_FILE: &str = "kr580-file-handler.desktop";
const MIME_TYPE: &str = "application/x-kr580";

pub fn register() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    register_for_executable(&exe, crate::install_mode::InstallScope::User)
}

pub fn register_for_executable(
    exe: &Path,
    _scope: crate::install_mode::InstallScope,
) -> Result<(), String> {
    if !super::is_user_configurable() {
        return Err("file associations are managed by snapd".to_owned());
    }
    let exe = association_executable_from(exe.to_path_buf());
    let paths = IntegrationPaths::current()?;

    std::fs::create_dir_all(&paths.mime_packages).map_err(|e| format!("create mime dir: {e}"))?;
    std::fs::create_dir_all(&paths.applications).map_err(|e| format!("create apps dir: {e}"))?;
    std::fs::create_dir_all(&paths.application_icons)
        .map_err(|e| format!("create icons dir: {e}"))?;
    std::fs::create_dir_all(&paths.file_type_icons)
        .map_err(|e| format!("create file icons dir: {e}"))?;

    let mime_file = paths.mime_packages.join("application-x-kr580.xml");
    let desktop_file = paths.applications.join(HANDLER_DESKTOP_FILE);
    let dest_icon = paths.application_icons.join("kr580.png");

    std::fs::write(&mime_file, crate::desktop_entry::MIME_XML)
        .map_err(|e| format!("write mime file: {e}"))?;
    std::fs::write(&desktop_file, crate::desktop_entry::file_handler(&exe)?)
        .map_err(|e| format!("write desktop file: {e}"))?;
    std::fs::write(dest_icon, crate::integration_assets::APPLICATION_ICON_PNG)
        .map_err(|e| format!("write app icon: {e}"))?;
    std::fs::write(
        paths.file_type_icons.join("application-x-kr580.png"),
        crate::integration_assets::FILE_TYPE_ICON_PNG,
    )
    .map_err(|e| format!("write file icon: {e}"))?;

    update_databases(&paths)?;
    set_default_handler()
}

pub fn unregister() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    unregister_for_executable(&exe, crate::install_mode::InstallScope::User)
}

fn remove_registration(paths: &IntegrationPaths) -> Result<(), String> {
    let _ = std::fs::remove_file(paths.mime_packages.join("application-x-kr580.xml"));
    let _ = std::fs::remove_file(paths.applications.join(HANDLER_DESKTOP_FILE));
    if !paths.applications.join("kr580.desktop").is_file() {
        let _ = std::fs::remove_file(paths.application_icons.join("kr580.png"));
    }
    let _ = std::fs::remove_file(paths.file_type_icons.join("application-x-kr580.png"));
    update_databases(&paths)?;
    Ok(())
}

pub fn unregister_for_executable(
    exe: &Path,
    _scope: crate::install_mode::InstallScope,
) -> Result<(), String> {
    if !super::is_user_configurable() {
        return Err("file associations are managed by snapd".to_owned());
    }
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
    if !super::is_user_configurable() {
        return false;
    }
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
    file_type_icons: PathBuf,
}

impl IntegrationPaths {
    fn current() -> Result<Self, String> {
        let data_home =
            data_home_from(std::env::var_os("XDG_DATA_HOME"), std::env::var_os("HOME"))?;
        Ok(Self {
            mime_packages: data_home.join("mime/packages"),
            applications: data_home.join("applications"),
            application_icons: data_home.join("icons/hicolor/256x256/apps"),
            file_type_icons: data_home.join("icons/hicolor/256x256/mimetypes"),
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

fn mime_registration_is_current(mime: &str) -> bool {
    mime.contains(r#"<glob pattern="*.580"/>"#) && mime.contains(r#"<glob pattern="*.krs"/>"#)
}

fn update_databases(paths: &IntegrationPaths) -> Result<(), String> {
    crate::desktop_entry::update_mime_database(&paths.data_home.join("mime"))?;
    crate::desktop_entry::update_desktop_database(&paths.applications)
}

fn set_default_handler() -> Result<(), String> {
    let status = std::process::Command::new("xdg-mime")
        .args(["default", HANDLER_DESKTOP_FILE, MIME_TYPE])
        .status()
        .map_err(|error| format!("xdg-mime default: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("xdg-mime default exited with {status}"))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        association_executable_from, data_home_from, desktop_entry_owned_by,
        mime_registration_is_current,
    };
    use std::ffi::OsString;
    use std::path::PathBuf;

    #[test]
    fn mime_registration_covers_snapshots_and_subprograms() {
        assert!(mime_registration_is_current(crate::desktop_entry::MIME_XML));
        assert!(!mime_registration_is_current(r#"<glob pattern="*.580"/>"#));
    }

    #[test]
    fn file_handler_is_hidden_from_application_menus() {
        let executable = PathBuf::from("/opt/kr580/k580");
        let entry = crate::desktop_entry::file_handler(&executable).unwrap();

        assert!(entry.contains("NoDisplay=true\n"));
        assert!(entry.contains("MimeType=application/x-kr580;\n"));
    }

    #[test]
    fn ownership_requires_the_exact_executable() {
        let executable = PathBuf::from("/opt/kr580/app/k580");
        let entry = crate::desktop_entry::file_handler(&executable).unwrap();

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
