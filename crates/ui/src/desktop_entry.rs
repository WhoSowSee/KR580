use std::path::Path;

pub fn quote_executable(executable: &Path) -> Result<String, String> {
    let value = executable
        .to_str()
        .ok_or_else(|| "executable path is not valid UTF-8".to_owned())?;
    if value.contains(['\0', '\n', '\r', '=']) {
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
    use super::quote_executable;
    use std::path::Path;

    #[test]
    fn executable_quoting_covers_desktop_entry_metacharacters() {
        let quoted = quote_executable(Path::new(
            "/opt/KR 580/$money/`tick`/back\\slash/\"quote\"/100%/k580",
        ))
        .unwrap();

        assert_eq!(
            quoted,
            r#""/opt/KR 580/\\$money/\\`tick\\`/back\\\\slash/\\\"quote\\\"/100%%/k580""#
        );
        assert!(quote_executable(Path::new("/opt/KR=580/k580")).is_err());
        assert!(quote_executable(Path::new("/opt/KR\n580/k580")).is_err());
    }
}
