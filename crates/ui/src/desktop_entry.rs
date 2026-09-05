use std::ffi::OsString;
use std::path::{Path, PathBuf};

pub const MIME_XML: &str = include_str!("../assets/linux/application-x-kr580.xml");

pub fn launcher(executable: &Path) -> Result<String, String> {
    render(include_str!("../assets/linux/kr580.desktop"), executable)
}

pub fn file_handler(executable: &Path) -> Result<String, String> {
    render(
        include_str!("../assets/linux/kr580-file-handler.desktop"),
        executable,
    )
}

fn render(template: &str, executable: &Path) -> Result<String, String> {
    Ok(template.replace("@EXEC@", &executable_command(executable)?))
}

pub fn executable_command(executable: &Path) -> Result<String, String> {
    let quoted = quote_executable(executable)?;
    if quoted.contains("%%") {
        // GLib validates argv[0] before expanding %% in Desktop Entry commands.
        Ok(format!("/usr/bin/env -- {quoted}"))
    } else {
        Ok(quoted)
    }
}

fn quote_executable(executable: &Path) -> Result<String, String> {
    let value = executable
        .to_str()
        .ok_or_else(|| "executable path is not valid UTF-8".to_owned())?;
    if value.contains('=') || value.chars().any(char::is_control) {
        return Err("executable path cannot be represented in a desktop entry".to_owned());
    }

    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('"');
    for character in value.chars() {
        match character {
            '\\' => quoted.push_str(r"\\\\"),
            '"' => quoted.push_str(r#"\\\""#),
            '$' => quoted.push_str(r"\\$"),
            '`' => quoted.push_str(r"\\`"),
            '%' => quoted.push_str("%%"),
            character => quoted.push(character),
        }
    }
    quoted.push('"');
    Ok(quoted)
}

pub fn data_home() -> Result<PathBuf, String> {
    base_home_from(
        std::env::var_os("XDG_DATA_HOME"),
        std::env::var_os("HOME"),
        ".local/share",
    )
}

pub fn config_home() -> Result<PathBuf, String> {
    base_home_from(
        std::env::var_os("XDG_CONFIG_HOME"),
        std::env::var_os("HOME"),
        ".config",
    )
}

fn base_home_from(
    xdg_home: Option<OsString>,
    home: Option<OsString>,
    fallback: &str,
) -> Result<PathBuf, String> {
    if let Some(value) = xdg_home.filter(|value| !value.is_empty()) {
        let path = PathBuf::from(value);
        if path.is_absolute() {
            return Ok(path);
        }
    }
    home.filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .map(|home| home.join(fallback))
        .ok_or_else(|| "HOME is missing or is not an absolute path".to_owned())
}

pub fn update_desktop_database(directory: &Path) -> Result<(), String> {
    run_database_command("update-desktop-database", directory)
}

pub fn update_mime_database(directory: &Path) -> Result<(), String> {
    run_database_command("update-mime-database", directory)
}

fn run_database_command(command: &str, directory: &Path) -> Result<(), String> {
    let status = std::process::Command::new(command)
        .arg(directory)
        .status()
        .map_err(|error| format!("{command} {}: {error}", directory.display()))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "{command} {} exited with {status}",
            directory.display()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{base_home_from, launcher, quote_executable};
    use std::ffi::OsString;
    use std::path::Path;

    #[test]
    fn executable_quoting_covers_desktop_entry_metacharacters() {
        let quoted = quote_executable(Path::new(
            "/opt/KR 580/$money/`tick`/back\\slash/\"quote\"/100%/kr580",
        ))
        .unwrap();

        assert_eq!(
            quoted,
            r#""/opt/KR 580/\\$money/\\`tick\\`/back\\\\slash/\\\"quote\\\"/100%%/kr580""#
        );
        assert!(quote_executable(Path::new("/opt/KR=580/kr580")).is_err());
        assert!(quote_executable(Path::new("/opt/KR\n580/kr580")).is_err());
        assert!(quote_executable(Path::new("/opt/KR\t580/kr580")).is_err());
        assert!(quote_executable(Path::new("/opt/KR\u{1}580/kr580")).is_err());
    }

    #[test]
    fn canonical_launcher_renders_its_executable() {
        let executable = Path::new("/opt/kr580/kr580");
        let launcher = launcher(executable).unwrap();

        assert!(launcher.contains("Exec=\"/opt/kr580/kr580\"\n"));
        assert!(!launcher.contains("@EXEC@"));
    }

    #[test]
    fn data_home_uses_absolute_override_or_home_fallback() {
        let absolute = std::env::temp_dir();
        assert_eq!(
            base_home_from(
                Some(absolute.clone().into_os_string()),
                None,
                ".local/share"
            )
            .unwrap(),
            absolute
        );
        assert_eq!(
            base_home_from(
                Some(OsString::from("relative")),
                Some(absolute.clone().into_os_string()),
                ".local/share"
            )
            .unwrap(),
            absolute.join(".local/share")
        );
        assert!(base_home_from(None, None, ".local/share").is_err());
    }
}
