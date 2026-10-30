use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const MANIFEST_FILENAME: &str = "install.json";

/// Failures resolving an installation or reading and writing its manifest.
#[derive(Debug, thiserror::Error)]
pub enum InstallModeError {
    #[error("manifest json: {0}")]
    SerializeManifest(#[source] serde_json::Error),
    #[error("write manifest: {0}")]
    WriteManifest(#[source] std::io::Error),
    #[error("read manifest: {0}")]
    ReadManifest(#[source] std::io::Error),
    #[error("parse manifest: {0}")]
    ParseManifest(#[source] serde_json::Error),
    #[error("KR580_GUI_EXECUTABLE does not name an absolute executable file")]
    InvalidExecutableOverride,
    #[error("current_exe: {0}")]
    CurrentExecutable(#[source] std::io::Error),
}

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

/// # Errors
/// Preserves serialization and atomic file-write failures as `InstallModeError` causes.
pub fn write_manifest(root: &Path, manifest: &InstallManifest) -> Result<(), InstallModeError> {
    let json =
        serde_json::to_string_pretty(manifest).map_err(InstallModeError::SerializeManifest)?;
    crate::persistence::write_file_atomic(&root.join(MANIFEST_FILENAME), json.as_bytes())
        .map_err(InstallModeError::WriteManifest)
}

/// Unmanaged executable layouts return `Ok(None)`.
///
/// # Errors
/// Preserves file-read and JSON-parse failures from an existing installation manifest.
pub fn manifest_for_executable(
    exe: &Path,
) -> Result<Option<(PathBuf, InstallManifest)>, InstallModeError> {
    let Some(root) = install_root_from_executable(exe) else {
        return Ok(None);
    };
    let json = std::fs::read_to_string(root.join(MANIFEST_FILENAME))
        .map_err(InstallModeError::ReadManifest)?;
    let manifest = serde_json::from_str(&json).map_err(InstallModeError::ParseManifest)?;
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

pub fn installed_binary_from_launcher(launcher: &Path, binary_name: &str) -> Option<PathBuf> {
    let directory = launcher.parent()?;
    if !path_component_matches(directory.file_name()?, "bin") {
        return None;
    }
    let (root, manifest) = manifest_for_executable(launcher).ok()??;
    if manifest.manifest_version != 1 {
        return None;
    }
    Some(root.join("app").join(binary_name))
}

pub fn companion_executable_from_launcher(
    executable: PathBuf,
    launcher_name: &str,
    companion_name: &str,
) -> PathBuf {
    if !path_component_matches(executable.file_name().unwrap_or_default(), launcher_name) {
        return executable;
    }
    installed_binary_from_launcher(&executable, companion_name)
        .unwrap_or_else(|| executable.with_file_name(companion_name))
}

/// # Errors
/// Rejects invalid overrides and preserves failure to resolve the current executable.
pub fn current_integration_executable() -> Result<PathBuf, InstallModeError> {
    if let Some(path) = std::env::var_os("KR580_GUI_EXECUTABLE") {
        let path = PathBuf::from(path);
        if path.is_absolute() && path.is_file() {
            return Ok(path);
        }
        return Err(InstallModeError::InvalidExecutableOverride);
    }
    std::env::current_exe().map_err(InstallModeError::CurrentExecutable)
}

fn path_component_matches(left: &std::ffi::OsStr, right: &str) -> bool {
    #[cfg(windows)]
    return left
        .to_str()
        .is_some_and(|left| left.eq_ignore_ascii_case(right));
    #[cfg(not(windows))]
    return left == right;
}

fn matches_ci(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn manifest_failures_preserve_io_and_json_causes() {
        let directory = tempfile::tempdir().unwrap();
        let executable = directory.path().join(binary_name("kr580"));
        let path = directory.path().join(MANIFEST_FILENAME);
        std::fs::write(&path, b"{}").unwrap();
        let error = manifest_for_executable(&executable).unwrap_err();
        assert!(matches!(error, InstallModeError::ParseManifest(_)));
        assert!(
            error
                .source()
                .unwrap()
                .downcast_ref::<serde_json::Error>()
                .is_some()
        );
        std::fs::write(&path, [0xFF]).unwrap();
        let error = manifest_for_executable(&executable).unwrap_err();
        assert!(matches!(error, InstallModeError::ReadManifest(_)));
        assert_eq!(
            error
                .source()
                .unwrap()
                .downcast_ref::<std::io::Error>()
                .unwrap()
                .kind(),
            std::io::ErrorKind::InvalidData
        );
        let error = write_manifest(
            &directory.path().join("missing"),
            &InstallManifest::new(InstallMode::Portable, InstallScope::User),
        )
        .unwrap_err();
        assert!(matches!(error, InstallModeError::WriteManifest(_)));
        assert!(
            error
                .source()
                .unwrap()
                .downcast_ref::<std::io::Error>()
                .is_some()
        );
    }

    #[test]
    fn detects_split_app_layout() {
        let root = unique_temp_dir("split-app");
        std::fs::create_dir_all(root.join("app")).unwrap();
        std::fs::write(root.join(MANIFEST_FILENAME), "{}").unwrap();

        assert_eq!(
            install_root_from_executable(&root.join("app").join(binary_name("kr580"))),
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

    #[test]
    fn installed_launcher_resolves_companion_under_app() {
        let root = unique_temp_dir("companion");
        std::fs::create_dir_all(root.join("bin")).unwrap();
        write_manifest(
            &root,
            &InstallManifest::new(InstallMode::System, InstallScope::User),
        )
        .unwrap();
        let launcher = root.join("bin").join(binary_name("kr"));

        assert_eq!(
            companion_executable_from_launcher(launcher, &binary_name("kr"), &binary_name("kr580")),
            root.join("app").join(binary_name("kr580"))
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn unmanaged_bin_launcher_uses_adjacent_companion() {
        let launcher = PathBuf::from("cargo").join("bin").join(binary_name("kr"));

        assert_eq!(
            companion_executable_from_launcher(
                launcher.clone(),
                &binary_name("kr"),
                &binary_name("kr580")
            ),
            launcher.with_file_name(binary_name("kr580"))
        );
    }

    #[test]
    fn malformed_manifest_does_not_turn_cargo_bin_into_installer_layout() {
        let root = unique_temp_dir("invalid-manifest");
        std::fs::create_dir_all(root.join("bin")).unwrap();
        std::fs::write(root.join(MANIFEST_FILENAME), "{}").unwrap();
        let launcher = root.join("bin").join(binary_name("kr"));
        assert!(installed_binary_from_launcher(&launcher, &binary_name("kr580")).is_none());
        assert_eq!(
            companion_executable_from_launcher(
                launcher.clone(),
                &binary_name("kr"),
                &binary_name("kr580")
            ),
            launcher.with_file_name(binary_name("kr580"))
        );
        std::fs::remove_dir_all(root).unwrap();
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
