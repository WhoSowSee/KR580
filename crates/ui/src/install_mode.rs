use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const MANIFEST_FILENAME: &str = "install.json";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InstallMode {
    System,
    Portable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InstallScope {
    User,
    Machine,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallManifest {
    pub manifest_version: u8,
    pub mode: InstallMode,
    pub scope: InstallScope,
    #[serde(default)]
    pub file_association: bool,
}

impl InstallManifest {
    pub fn new(mode: InstallMode, scope: InstallScope) -> Self {
        Self {
            manifest_version: 1,
            mode,
            scope,
            file_association: false,
        }
    }

    pub fn with_file_association(mut self, file_association: bool) -> Self {
        self.file_association = file_association;
        self
    }
}

pub fn write_manifest(root: &Path, manifest: &InstallManifest) -> Result<(), String> {
    let json = serde_json::to_string_pretty(manifest).map_err(|e| format!("manifest json: {e}"))?;
    std::fs::write(root.join(MANIFEST_FILENAME), json).map_err(|e| format!("write manifest: {e}"))
}

pub fn manifest_for_executable(exe: &Path) -> Result<Option<(PathBuf, InstallManifest)>, String> {
    let Some(root) = install_root_from_executable(exe) else {
        return Ok(None);
    };
    let json = std::fs::read_to_string(root.join(MANIFEST_FILENAME))
        .map_err(|e| format!("read manifest: {e}"))?;
    let manifest = serde_json::from_str(&json).map_err(|e| format!("parse manifest: {e}"))?;
    Ok(Some((root, manifest)))
}

pub fn install_root_from_executable(exe: &Path) -> Option<PathBuf> {
    let dir = exe.parent()?;
    if dir.join(MANIFEST_FILENAME).is_file() {
        return Some(dir.to_path_buf());
    }
    let parent = dir.parent()?;
    let leaf = dir.file_name()?.to_string_lossy();
    if matches_ci(&leaf, "app") || matches_ci(&leaf, "bin") {
        let manifest = parent.join(MANIFEST_FILENAME);
        if manifest.is_file() {
            return Some(parent.to_path_buf());
        }
    }
    None
}

fn matches_ci(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_split_app_layout() {
        let root = unique_temp_dir("split-app");
        std::fs::create_dir_all(root.join("app")).unwrap();
        std::fs::write(root.join(MANIFEST_FILENAME), "{}").unwrap();

        assert_eq!(
            install_root_from_executable(&root.join("app").join(binary_name("k580"))),
            Some(root.clone())
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn detects_split_bin_layout() {
        let root = unique_temp_dir("split-bin");
        std::fs::create_dir_all(root.join("bin")).unwrap();
        std::fs::write(root.join(MANIFEST_FILENAME), "{}").unwrap();

        assert_eq!(
            install_root_from_executable(&root.join("bin").join(binary_name("kr"))),
            Some(root.clone())
        );

        let _ = std::fs::remove_dir_all(root);
    }

    fn unique_temp_dir(name: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("k580-install-mode-{nanos}-{name}"))
    }

    #[cfg(windows)]
    fn binary_name(name: &str) -> String {
        format!("{name}.exe")
    }

    #[cfg(not(windows))]
    fn binary_name(name: &str) -> String {
        name.to_owned()
    }
}
