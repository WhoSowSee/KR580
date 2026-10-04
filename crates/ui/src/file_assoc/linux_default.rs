use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

use super::linux::HANDLER_DESKTOP_FILE as HANDLER;
use super::linux_files::{
    Backup, atomic_write, read_optional, remove_file_if_exists, with_rollback,
};

const MIME_TYPE: &str = "application/x-kr580";

#[derive(Deserialize, Serialize)]
struct HandlerState {
    owner: PathBuf,
    previous: Option<String>,
}

pub(super) struct DefaultRegistration {
    state_path: PathBuf,
    backup: Backup,
    state: HandlerState,
}

impl DefaultRegistration {
    pub(super) fn prepare(data_home: &Path, owner: &Path) -> Result<Self, String> {
        let state_path = state_path(data_home);
        let saved = read_state(&state_path)?;
        let current = query_default()?;
        let previous = if current.as_deref() == Some(HANDLER) {
            saved.and_then(|state| state.previous)
        } else {
            current
        };
        let mut files = mimeapps_paths(data_home)?;
        files.push(state_path.clone());
        Ok(Self {
            state_path,
            backup: Backup::capture(files)?,
            state: HandlerState {
                owner: owner.to_path_buf(),
                previous,
            },
        })
    }

    pub(super) fn apply(self) -> Result<(), String> {
        let result = (|| {
            let bytes = serde_json::to_vec_pretty(&self.state)
                .map_err(|error| format!("serialize file association state: {error}"))?;
            atomic_write(&self.state_path, &bytes)?;
            set_default(HANDLER)?;
            if query_default()?.as_deref() != Some(HANDLER) {
                return Err("desktop did not accept KR580 as the default handler".to_owned());
            }
            Ok(())
        })();
        result.map_err(|error| with_rollback(error, self.backup.restore()))
    }
}

pub(super) struct DefaultRestoration {
    state_path: PathBuf,
    files: Vec<PathBuf>,
    backup: Backup,
    previous: Option<String>,
}

impl DefaultRestoration {
    pub(super) fn prepare(data_home: &Path, owner: &Path) -> Result<Self, String> {
        let state_path = state_path(data_home);
        let state = read_state(&state_path)?;
        if let Some(state) = state.as_ref()
            && state.owner != owner
        {
            return Err("file association state belongs to another installation".to_owned());
        }
        let files = mimeapps_paths(data_home)?;
        let backup = Backup::capture(files.iter().cloned().chain([state_path.clone()]))?;
        let previous = if query_default()?.as_deref() == Some(HANDLER) {
            state.and_then(|state| state.previous)
        } else {
            None
        };
        Ok(Self {
            state_path,
            files,
            backup,
            previous,
        })
    }

    pub(super) fn apply(self) -> Result<(), String> {
        let result = (|| {
            for path in &self.files {
                let Some(bytes) = read_optional(path)? else {
                    continue;
                };
                let content = String::from_utf8(bytes)
                    .map_err(|error| format!("parse {}: {error}", path.display()))?;
                let updated = remove_handler_entries(&content);
                if updated != content {
                    atomic_write(path, updated.as_bytes())?;
                }
            }
            if let Some(previous) = self.previous.as_deref() {
                set_default(previous)?;
            }
            remove_file_if_exists(&self.state_path)
        })();
        result.map_err(|error| with_rollback(error, self.backup.restore()))
    }
}

fn query_default() -> Result<Option<String>, String> {
    let value = run_xdg_mime(&["query", "default", MIME_TYPE])?;
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    if !valid_desktop_id(value) {
        return Err("xdg-mime returned an invalid desktop file id".to_owned());
    }
    Ok(Some(value.to_owned()))
}

fn set_default(handler: &str) -> Result<(), String> {
    run_xdg_mime(&["default", handler, MIME_TYPE]).map(|_| ())
}

fn run_xdg_mime(arguments: &[&str]) -> Result<String, String> {
    let output = Command::new("xdg-mime")
        .args(arguments)
        .output()
        .map_err(|error| format!("xdg-mime: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "xdg-mime exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    String::from_utf8(output.stdout).map_err(|error| format!("xdg-mime output: {error}"))
}

fn valid_desktop_id(value: &str) -> bool {
    value.ends_with(".desktop")
        && !value.starts_with('-')
        && !value.contains(['/', '\\', ';'])
        && !value.chars().any(char::is_control)
}

fn state_path(data_home: &Path) -> PathBuf {
    data_home.join("kr580/file-association.json")
}

fn read_state(path: &Path) -> Result<Option<HandlerState>, String> {
    let Some(bytes) = read_optional(path)? else {
        return Ok(None);
    };
    let state: HandlerState = serde_json::from_slice(&bytes)
        .map_err(|error| format!("read association state {}: {error}", path.display()))?;
    if state
        .previous
        .as_deref()
        .is_some_and(|previous| !valid_desktop_id(previous) || previous == HANDLER)
    {
        return Err("invalid previous handler in association state".to_owned());
    }
    Ok(Some(state))
}

fn mimeapps_paths(data_home: &Path) -> Result<Vec<PathBuf>, String> {
    let config_home = crate::desktop_entry::config_home()?;
    let directories = [config_home, data_home.join("applications")];
    let mut files = Vec::new();
    for directory in directories {
        files.push(directory.join("mimeapps.list"));
        let desktops = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
        for desktop in desktops.split(':').filter(|name| !name.is_empty()) {
            if !desktop
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
            {
                return Err("invalid XDG_CURRENT_DESKTOP component".to_owned());
            }
            files.push(directory.join(format!("{}-mimeapps.list", desktop.to_ascii_lowercase())));
        }
        match std::fs::read_dir(&directory) {
            Ok(entries) => {
                for entry in entries {
                    let entry =
                        entry.map_err(|error| format!("read {}: {error}", directory.display()))?;
                    if entry
                        .file_name()
                        .to_str()
                        .is_some_and(|name| name.ends_with("-mimeapps.list"))
                    {
                        files.push(entry.path());
                    }
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(format!("read {}: {error}", directory.display())),
        }
    }
    files.sort();
    files.dedup();
    Ok(files)
}

pub(super) fn registration_files(data_home: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = mimeapps_paths(data_home)?;
    files.push(state_path(data_home));
    Ok(files)
}

fn remove_handler_entries(content: &str) -> String {
    let mut relevant = false;
    let mut output = String::with_capacity(content.len());
    for original in content.split_inclusive('\n') {
        let line = original.trim_end_matches(['\n', '\r']);
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            relevant = matches!(trimmed, "[Default Applications]" | "[Added Associations]");
        }
        let replacement = line
            .split_once('=')
            .filter(|(key, _)| relevant && key.trim() == MIME_TYPE)
            .and_then(|(key, value)| {
                let mut handlers = value
                    .split(';')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>();
                if !handlers.contains(&HANDLER) {
                    return None;
                }
                handlers.retain(|handler| *handler != HANDLER);
                if handlers.is_empty() {
                    return Some(String::new());
                }
                let ending = &original[line.len()..];
                Some(format!("{key}={};{ending}", handlers.join(";")))
            });
        output.push_str(replacement.as_deref().unwrap_or(original));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{remove_handler_entries, valid_desktop_id};

    #[test]
    fn cleanup_preserves_unrelated_settings_and_line_endings() {
        let input = "[Default Applications]\r\napplication/x-kr580=kr580-file-handler.desktop;other.desktop;\r\ntext/plain=editor.desktop;\r\n[Added Associations]\r\napplication/x-kr580=kr580-file-handler.desktop;\r\n";
        assert_eq!(
            remove_handler_entries(input),
            "[Default Applications]\r\napplication/x-kr580=other.desktop;\r\ntext/plain=editor.desktop;\r\n[Added Associations]\r\n"
        );
        assert_eq!(
            remove_handler_entries("[Default Applications]\r\napplication/x-kr580=other.desktop"),
            "[Default Applications]\r\napplication/x-kr580=other.desktop"
        );
    }

    #[test]
    fn invalid_desktop_ids_are_rejected() {
        assert!(valid_desktop_id("org.example.App.desktop"));
        for id in [
            "../app.desktop",
            "app\t.desktop",
            "--foo.desktop",
            "a;b.desktop",
        ] {
            assert!(!valid_desktop_id(id));
        }
    }
}
