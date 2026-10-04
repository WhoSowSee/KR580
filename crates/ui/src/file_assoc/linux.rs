use super::linux_files::{
    Backup, acquire_lock, atomic_write, read_optional, remove_file_if_exists, with_rollback,
};
use std::path::{Path, PathBuf};

pub(super) const HANDLER_DESKTOP_FILE: &str = "kr580-file-handler.desktop";

pub fn register() -> Result<(), String> {
    let exe = crate::install_mode::current_integration_executable()?;
    register_for_executable(&exe, crate::install_mode::InstallScope::User)
}

pub fn register_for_executable(
    exe: &Path,
    _scope: crate::install_mode::InstallScope,
) -> Result<(), String> {
    if !super::is_user_configurable() {
        return Err("file associations are managed by snapd".to_owned());
    }
    let exe =
        crate::install_mode::companion_executable_from_launcher(exe.to_path_buf(), "kr", "kr580");
    let paths = IntegrationPaths::current()?;
    let _lock = acquire_lock(&paths.data_home)?;
    let backup = Backup::capture(registration_files(&paths))?;
    let default = super::linux_default::DefaultRegistration::prepare(&paths.data_home, &exe)?;
    let result = (|| {
        write_registration(&paths, &exe)?;
        update_databases(&paths)?;
        default.apply()
    })();
    match result {
        Ok(()) => Ok(()),
        Err(error) => Err(with_rollback(error, rollback_registration(&paths, &backup))),
    }
}

fn write_registration(paths: &IntegrationPaths, exe: &Path) -> Result<(), String> {
    let entry = crate::desktop_entry::file_handler(exe)?;

    let mime_file = paths.mime_packages.join("application-x-kr580.xml");
    let desktop_file = paths.applications.join(HANDLER_DESKTOP_FILE);
    let dest_icon = paths.application_icons.join("kr580.png");

    atomic_write(&mime_file, crate::desktop_entry::MIME_XML.as_bytes())
        .map_err(|e| format!("write mime file: {e}"))?;
    atomic_write(&desktop_file, entry.as_bytes())
        .map_err(|e| format!("write desktop file: {e}"))?;
    atomic_write(&dest_icon, crate::integration_assets::APPLICATION_ICON_PNG)
        .map_err(|e| format!("write app icon: {e}"))?;
    atomic_write(
        &paths.file_type_icons.join("application-x-kr580.png"),
        crate::integration_assets::FILE_TYPE_ICON_PNG,
    )
    .map_err(|e| format!("write file icon: {e}"))?;

    Ok(())
}

pub fn unregister() -> Result<(), String> {
    let exe = crate::install_mode::current_integration_executable()?;
    unregister_for_executable(&exe, crate::install_mode::InstallScope::User)
}

fn remove_registration_files(paths: &IntegrationPaths) -> Result<(), String> {
    remove_file_if_exists(&paths.mime_packages.join("application-x-kr580.xml"))?;
    remove_file_if_exists(&paths.applications.join(HANDLER_DESKTOP_FILE))?;
    if !paths.applications.join("kr580.desktop").is_file() {
        remove_file_if_exists(&paths.application_icons.join("kr580.png"))?;
    }
    remove_file_if_exists(&paths.file_type_icons.join("application-x-kr580.png"))
}

pub fn unregister_for_executable(
    exe: &Path,
    _scope: crate::install_mode::InstallScope,
) -> Result<(), String> {
    if !super::is_user_configurable() {
        return Err("file associations are managed by snapd".to_owned());
    }
    let exe =
        crate::install_mode::companion_executable_from_launcher(exe.to_path_buf(), "kr", "kr580");
    let paths = IntegrationPaths::current()?;
    let desktop_file = paths.applications.join(HANDLER_DESKTOP_FILE);
    let Some(initial) = read_optional(&desktop_file)? else {
        return Ok(());
    };
    if !desktop_entry_owned_by(&String::from_utf8_lossy(&initial), &exe) {
        return Ok(());
    }
    let _lock = acquire_lock(&paths.data_home)?;
    let Some(current) = read_optional(&desktop_file)? else {
        return Ok(());
    };
    let current =
        String::from_utf8(current).map_err(|error| format!("read desktop entry: {error}"))?;
    if !desktop_entry_owned_by(&current, &exe) {
        return Ok(());
    }
    let backup = Backup::capture(registration_files(&paths))?;
    let default = super::linux_default::DefaultRestoration::prepare(&paths.data_home, &exe)?;
    let result = (|| {
        remove_registration_files(&paths)?;
        update_databases(&paths)?;
        default.apply()
    })();
    if let Err(error) = result {
        return Err(with_rollback(error, rollback_registration(&paths, &backup)));
    }
    Ok(())
}

fn desktop_entry_owned_by(entry: &str, exe: &Path) -> bool {
    crate::desktop_entry::executable_command(exe)
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
        Ok(Self::from_data_home(crate::desktop_entry::data_home()?))
    }

    fn from_data_home(data_home: PathBuf) -> Self {
        Self {
            mime_packages: data_home.join("mime/packages"),
            applications: data_home.join("applications"),
            application_icons: data_home.join("icons/hicolor/256x256/apps"),
            file_type_icons: data_home.join("icons/hicolor/256x256/mimetypes"),
            data_home,
        }
    }
}

fn registration_files(paths: &IntegrationPaths) -> [PathBuf; 4] {
    [
        paths.mime_packages.join("application-x-kr580.xml"),
        paths.applications.join(HANDLER_DESKTOP_FILE),
        paths.application_icons.join("kr580.png"),
        paths.file_type_icons.join("application-x-kr580.png"),
    ]
}

/// Returns writable association metadata and default-handler files for installer rollback.
pub fn registration_paths() -> Result<Vec<PathBuf>, String> {
    if !super::is_user_configurable() {
        return Err("file associations are managed by snapd".into());
    }
    let paths = IntegrationPaths::current()?;
    let mut files = registration_files(&paths).to_vec();
    files.extend(super::linux_default::registration_files(&paths.data_home)?);
    Ok(files)
}

fn rollback_registration(paths: &IntegrationPaths, backup: &Backup) -> Result<(), String> {
    backup.restore()?;
    update_databases(paths)
}

fn update_databases(paths: &IntegrationPaths) -> Result<(), String> {
    crate::desktop_entry::update_mime_database(&paths.data_home.join("mime"))?;
    crate::desktop_entry::update_desktop_database(&paths.applications)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    #[test]
    #[ignore = "requires the gio Desktop Entry launcher"]
    fn desktop_entry_consumer_preserves_executable_and_file_arguments() {
        use std::os::unix::fs::PermissionsExt;

        let root = unique_temp_dir().join("space $cash `tick` 100%");
        std::fs::create_dir_all(&root).unwrap();
        let executable = root.join("handler");
        let output = root.join("received.txt");
        let program = root.join("program with spaces.580");
        let script = format!(
            "#!/bin/sh\nprintf '%s' \"$1\" > {}\n",
            crate::shell_quote::single(&output.display().to_string())
        );
        std::fs::write(&executable, script).unwrap();
        let mut permissions = std::fs::metadata(&executable).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&executable, permissions).unwrap();
        std::fs::write(&program, []).unwrap();
        let applications = root.join("data/applications");
        std::fs::create_dir_all(&applications).unwrap();
        let desktop = applications.join("kr580-test.desktop");
        std::fs::write(
            &desktop,
            crate::desktop_entry::file_handler(&executable).unwrap(),
        )
        .unwrap();

        let status = std::process::Command::new("gio")
            .args(["launch"])
            .arg(&desktop)
            .arg(&program)
            .env("XDG_DATA_HOME", root.join("data"))
            .status()
            .unwrap();
        assert!(status.success());
        for _ in 0..100 {
            if output.is_file() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert_eq!(
            std::fs::read_to_string(output).unwrap(),
            program.to_string_lossy()
        );

        std::fs::remove_dir_all(root.parent().unwrap()).unwrap();
    }

    fn unique_temp_dir() -> PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "kr580-linux-association-{}-{nonce}",
            std::process::id()
        ))
    }
}
