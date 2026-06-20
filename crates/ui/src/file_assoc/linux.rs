use std::path::{Path, PathBuf};

pub fn register() -> Result<(), String> {
    let kr = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    register_for_executable(&kr, crate::install_mode::InstallScope::User)
}

pub fn register_for_executable(
    exe: &Path,
    _scope: crate::install_mode::InstallScope,
) -> Result<(), String> {
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
    let desktop_file = apps_dir.join("kr580.desktop");
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
    let _ = std::fs::remove_file(mime_dir().join("application-x-kr580.xml"));
    let _ = std::fs::remove_file(apps_dir().join("kr580.desktop"));
    let _ = std::fs::remove_file(hicolor_icon_dir().join("kr580.png"));
    update_databases();
    Ok(())
}

pub fn unregister_for_executable(
    exe: &Path,
    _scope: crate::install_mode::InstallScope,
) -> Result<(), String> {
    let exe_str = exe
        .to_str()
        .ok_or_else(|| "executable path is not valid UTF-8".to_owned())?;
    let desktop_file = apps_dir().join("kr580.desktop");
    let current = std::fs::read_to_string(&desktop_file).unwrap_or_default();
    if current.contains(&format!("Exec={exe_str} %f")) {
        unregister()?;
    }
    Ok(())
}

pub fn is_registered() -> bool {
    mime_dir().join("application-x-kr580.xml").is_file()
        && apps_dir().join("kr580.desktop").is_file()
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
    <comment>KR580 snapshot</comment>
    <glob pattern="*.580"/>
  </mime-type>
</mime-info>
"#
}

fn desktop_entry(exec: &str) -> String {
    format!(
        "[Desktop Entry]\n\
         Name=KR580 Emulator\n\
         Comment=KR580 emulator\n\
         Exec={} %f\n\
         Icon=kr580\n\
         Type=Application\n\
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
