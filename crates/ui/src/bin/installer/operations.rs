use super::platform;
use k580_ui::install_mode::{InstallManifest, InstallMode, InstallScope, write_manifest};
use std::path::{Path, PathBuf};

mod payload {
    include!(concat!(env!("OUT_DIR"), "/installer_payload.rs"));
}

#[derive(Clone, Debug)]
pub struct InstallRequest {
    pub mode: InstallMode,
    pub scope: InstallScope,
    pub install_dir: PathBuf,
    pub add_to_path: bool,
    pub create_desktop_shortcut: bool,
    pub associate_580_files: bool,
}

#[derive(Clone, Debug)]
pub struct InstallReport {
    pub mode: InstallMode,
    pub install_dir: PathBuf,
    pub k580_path: PathBuf,
    pub path_changed: bool,
    pub system_integrated: bool,
    pub desktop_shortcut_created: bool,
    pub file_association_created: bool,
}

pub fn install(request: InstallRequest) -> Result<InstallReport, String> {
    let source = SourceBundle::discover()?;
    let app_dir = request.install_dir.join("app");
    let bin_dir = request.install_dir.join("bin");

    std::fs::create_dir_all(&app_dir).map_err(|e| format!("create app dir: {e}"))?;
    std::fs::create_dir_all(&bin_dir).map_err(|e| format!("create bin dir: {e}"))?;
    if request.mode == InstallMode::Portable {
        std::fs::create_dir_all(request.install_dir.join("data"))
            .map_err(|e| format!("create data dir: {e}"))?;
    }

    let k580_path = app_dir.join(binary_name("k580"));
    let kr_path = bin_dir.join(binary_name("kr"));
    let uninstaller_path = app_dir.join(binary_name("uninstaller"));

    copy_executable(&source.k580, &k580_path)?;
    copy_executable(&source.kr, &kr_path)?;
    copy_executable(&source.uninstaller, &uninstaller_path)?;
    let mut manifest = InstallManifest::new(request.mode, request.scope);
    write_manifest(&request.install_dir, &manifest)?;

    let path_changed = if request.add_to_path {
        platform::add_to_path(&bin_dir, request.scope)?
    } else {
        false
    };
    let integration = if request.mode == InstallMode::System {
        let integration =
            platform::install_system_integration(&platform::SystemIntegrationRequest {
                scope: request.scope,
                install_dir: &request.install_dir,
                k580_path: &k580_path,
                uninstaller_path: &uninstaller_path,
                create_desktop_shortcut: request.create_desktop_shortcut,
            })?;
        Some(integration)
    } else {
        None
    };
    let file_association_created = if request.associate_580_files {
        k580_ui::file_assoc::register_for_executable(&k580_path, request.scope)?;
        true
    } else {
        false
    };
    manifest = manifest.with_file_association(file_association_created);
    write_manifest(&request.install_dir, &manifest)?;

    Ok(InstallReport {
        mode: request.mode,
        install_dir: request.install_dir,
        k580_path,
        path_changed,
        system_integrated: integration.is_some(),
        desktop_shortcut_created: integration
            .as_ref()
            .is_some_and(|report| report.desktop_shortcut_created),
        file_association_created,
    })
}

pub fn open_install_folder(path: PathBuf) -> Result<(), String> {
    platform::open_folder(&path)
}

pub fn launch_installed_app(path: PathBuf) -> Result<(), String> {
    platform::launch_app(&path)
}

pub fn prepare_uninstall(install_dir: &Path) -> Result<(), String> {
    let manifest = read_manifest(install_dir)?;
    let bin_dir = install_dir.join("bin");
    if manifest.file_association {
        let k580_path = install_dir.join("app").join(binary_name("k580"));
        k580_ui::file_assoc::unregister_for_executable(&k580_path, manifest.scope)?;
    }
    let _ = platform::remove_from_path(&bin_dir, manifest.scope)?;
    if manifest.mode == InstallMode::System {
        platform::remove_system_integration(install_dir, manifest.scope)?;
    }
    Ok(())
}

pub fn schedule_install_dir_removal(install_dir: &Path) -> Result<(), String> {
    platform::schedule_remove_install_dir(install_dir)
}

pub fn default_install_dir(mode: InstallMode, scope: InstallScope) -> PathBuf {
    if mode == InstallMode::Portable {
        return default_portable_install_dir();
    }
    platform::default_system_install_dir(scope)
}

fn default_portable_install_dir() -> PathBuf {
    #[cfg(windows)]
    let home = std::env::var_os("USERPROFILE");
    #[cfg(not(windows))]
    let home = std::env::var_os("HOME");

    home.map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        .join("KR580")
}

#[derive(Clone, Debug)]
struct SourceBundle {
    kr: SourceBinary,
    k580: SourceBinary,
    uninstaller: SourceBinary,
}

#[derive(Clone, Debug)]
enum SourceBinary {
    Embedded(&'static [u8]),
    File(PathBuf),
}

impl SourceBundle {
    fn discover() -> Result<Self, String> {
        if let Some(bundle) = Self::embedded()? {
            return Ok(bundle);
        }
        Ok(Self {
            kr: SourceBinary::File(find_source_binary("kr")?),
            k580: SourceBinary::File(find_source_binary("k580")?),
            uninstaller: SourceBinary::File(find_uninstaller_source()?),
        })
    }

    fn embedded() -> Result<Option<Self>, String> {
        match (
            payload::EMBEDDED_KR,
            payload::EMBEDDED_K580,
            payload::EMBEDDED_UNINSTALLER,
        ) {
            (Some(kr), Some(k580), Some(uninstaller)) => Ok(Some(Self {
                kr: SourceBinary::Embedded(kr),
                k580: SourceBinary::Embedded(k580),
                uninstaller: SourceBinary::Embedded(uninstaller),
            })),
            (None, None, None) => Ok(None),
            _ => Err("embedded installer payload is incomplete".to_owned()),
        }
    }
}

fn find_uninstaller_source() -> Result<PathBuf, String> {
    find_source_binary("k580-uninstaller")
        .or_else(|_| std::env::current_exe().map_err(|e| format!("current exe: {e}")))
}

fn find_source_binary(name: &str) -> Result<PathBuf, String> {
    let current = std::env::current_exe().map_err(|e| format!("current exe: {e}"))?;
    let binary = binary_name(name);
    let mut candidates = Vec::new();

    if let Some(dir) = current.parent() {
        candidates.push(dir.join(binary.clone()));
        if let Some(root) = dir.parent() {
            candidates.push(root.join("bin").join(binary.clone()));
            candidates.push(root.join("app").join(binary.clone()));
        }
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    candidates.push(
        manifest_dir
            .join("..")
            .join("..")
            .join("target")
            .join(profile)
            .join(binary),
    );

    candidates
        .into_iter()
        .find(|path| path.is_file())
        .ok_or_else(|| format!("{name} binary not found"))
}

fn copy_executable(source: &SourceBinary, destination: &Path) -> Result<(), String> {
    match source {
        SourceBinary::Embedded(bytes) => {
            std::fs::write(destination, bytes)
                .map_err(|e| format!("write {}: {e}", destination.display()))?;
        }
        SourceBinary::File(path) => {
            std::fs::copy(path, destination)
                .map_err(|e| format!("copy {}: {e}", path.display()))?;
        }
    }
    platform::make_executable(destination)
}

fn read_manifest(root: &Path) -> Result<InstallManifest, String> {
    let json = std::fs::read_to_string(root.join(k580_ui::install_mode::MANIFEST_FILENAME))
        .map_err(|e| format!("read install manifest: {e}"))?;
    serde_json::from_str(&json).map_err(|e| format!("parse install manifest: {e}"))
}

#[cfg(windows)]
fn binary_name(name: &str) -> String {
    format!("{name}.exe")
}

#[cfg(not(windows))]
fn binary_name(name: &str) -> String {
    name.to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portable_default_uses_user_kr580_folder() {
        let dir = default_install_dir(InstallMode::Portable, InstallScope::User);
        assert!(!dir.as_os_str().is_empty());
        assert_eq!(
            dir.file_name().and_then(|name| name.to_str()),
            Some("KR580")
        );
    }

    #[test]
    fn binary_names_match_platform_suffix() {
        #[cfg(windows)]
        assert_eq!(binary_name("kr"), "kr.exe");
        #[cfg(not(windows))]
        assert_eq!(binary_name("kr"), "kr");
    }
}
