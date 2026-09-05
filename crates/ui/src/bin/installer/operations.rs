use super::platform;
use k580_ui::install_mode::{InstallManifest, InstallMode, InstallScope, write_manifest};
use std::path::{Path, PathBuf};

mod payload {
    include!(concat!(env!("OUT_DIR"), "/installer_payload.rs"));
}

pub(super) struct InstallRequest {
    pub(super) mode: InstallMode,
    pub(super) scope: InstallScope,
    pub(super) install_dir: PathBuf,
    pub(super) add_to_path: bool,
    pub(super) create_desktop_shortcut: bool,
    pub(super) associate_program_files: bool,
}

#[derive(Clone, Debug)]
pub(super) struct InstallReport {
    pub(super) mode: InstallMode,
    pub(super) install_dir: PathBuf,
    pub(super) kr580_path: PathBuf,
    pub(super) path_changed: bool,
    pub(super) system_integrated: bool,
    pub(super) desktop_shortcut_created: bool,
    pub(super) file_association_created: bool,
}

#[derive(Clone, Debug)]
pub(super) struct UninstallPlan {
    install_dir: PathBuf,
    manifest: InstallManifest,
}

pub(super) fn install(request: InstallRequest) -> Result<InstallReport, String> {
    let source = SourceBundle::discover()?;
    #[cfg(target_os = "macos")]
    if request.mode == InstallMode::System || request.associate_program_files {
        let bundle = if request.mode == InstallMode::Portable {
            request
                .install_dir
                .join(k580_ui::macos_bundle::APP_BUNDLE_NAME)
        } else {
            k580_ui::macos_bundle::applications_dir()?.join(k580_ui::macos_bundle::APP_BUNDLE_NAME)
        };
        k580_ui::macos_bundle::validate_launcher_bundle(
            &bundle,
            &request.install_dir.join("app/kr580"),
        )?;
    }
    let app_dir = request.install_dir.join("app");
    let bin_dir = request.install_dir.join("bin");

    std::fs::create_dir_all(&app_dir).map_err(|e| format!("create app dir: {e}"))?;
    std::fs::create_dir_all(&bin_dir).map_err(|e| format!("create bin dir: {e}"))?;
    if request.mode == InstallMode::Portable {
        std::fs::create_dir_all(request.install_dir.join("data"))
            .map_err(|e| format!("create data dir: {e}"))?;
    }

    let kr580_path = app_dir.join(binary_name("kr580"));
    let kr_path = bin_dir.join(binary_name("kr"));
    let uninstaller_path = app_dir.join(binary_name("uninstaller"));

    copy_executable(&source.kr580, &kr580_path)?;
    copy_executable(&source.kr, &kr_path)?;
    copy_executable(&source.uninstaller, &uninstaller_path)?;
    remove_legacy_gui_binary(&request.install_dir)?;
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
                #[cfg(windows)]
                scope: request.scope,
                #[cfg(windows)]
                install_dir: &request.install_dir,
                kr580_path: &kr580_path,
                #[cfg(windows)]
                uninstaller_path: &uninstaller_path,
                create_desktop_shortcut: request.create_desktop_shortcut,
            })?;
        Some(integration)
    } else {
        None
    };
    let file_association_created = if request.associate_program_files {
        k580_ui::file_assoc::register_for_executable(&kr580_path, request.scope)?;
        true
    } else {
        false
    };
    manifest = manifest.with_file_association(file_association_created);
    write_manifest(&request.install_dir, &manifest)?;

    Ok(InstallReport {
        mode: request.mode,
        install_dir: request.install_dir,
        kr580_path,
        path_changed,
        system_integrated: integration.is_some(),
        desktop_shortcut_created: integration
            .as_ref()
            .is_some_and(|report| report.desktop_shortcut_created),
        file_association_created,
    })
}

pub(super) fn remove_system_entries(install_dir: PathBuf) -> Result<UninstallPlan, String> {
    let manifest = read_manifest(&install_dir)?;
    if manifest.mode == InstallMode::System {
        platform::remove_system_integration(&install_dir, manifest.scope)?;
    }
    Ok(UninstallPlan {
        install_dir,
        manifest,
    })
}

pub(super) fn remove_links(plan: UninstallPlan) -> Result<(), String> {
    #[cfg(not(target_os = "macos"))]
    if plan.manifest.file_association {
        let kr580_path = plan.install_dir.join("app").join(binary_name("kr580"));
        k580_ui::file_assoc::unregister_for_executable(&kr580_path, plan.manifest.scope)?;
    }
    let _ = platform::remove_from_path(&plan.install_dir.join("bin"), plan.manifest.scope)?;
    Ok(())
}

pub(super) fn default_install_dir(mode: InstallMode, scope: InstallScope) -> PathBuf {
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

struct SourceBundle {
    kr: SourceBinary,
    kr580: SourceBinary,
    uninstaller: SourceBinary,
}

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
            kr580: SourceBinary::File(find_source_binary("kr580")?),
            uninstaller: SourceBinary::File(find_uninstaller_source()?),
        })
    }

    fn embedded() -> Result<Option<Self>, String> {
        match (
            payload::EMBEDDED_KR,
            payload::EMBEDDED_KR580,
            payload::EMBEDDED_UNINSTALLER,
        ) {
            (Some(kr), Some(kr580), Some(uninstaller)) => Ok(Some(Self {
                kr: SourceBinary::Embedded(kr),
                kr580: SourceBinary::Embedded(kr580),
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

fn remove_legacy_gui_binary(install_dir: &Path) -> Result<(), String> {
    let legacy = install_dir.join("app").join(binary_name("k580"));
    if !legacy.is_file() {
        return Ok(());
    }
    let manifest_path = install_dir.join(k580_ui::install_mode::MANIFEST_FILENAME);
    if !manifest_path.is_file() {
        return Ok(());
    }
    let manifest = read_manifest(install_dir)?;
    #[cfg(not(target_os = "macos"))]
    k580_ui::file_assoc::unregister_for_executable(&legacy, manifest.scope)?;
    #[cfg(target_os = "macos")]
    let _ = manifest;
    std::fs::remove_file(&legacy).map_err(|e| format!("remove {}: {e}", legacy.display()))
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
        assert_eq!(binary_name("kr580"), "kr580.exe");
        #[cfg(not(windows))]
        assert_eq!(binary_name("kr580"), "kr580");
    }

    #[test]
    fn unmanaged_legacy_binary_is_preserved() {
        let root = unique_temp_dir("unmanaged");
        let legacy = root.join("app").join(binary_name("k580"));
        std::fs::create_dir_all(legacy.parent().unwrap()).unwrap();
        std::fs::write(&legacy, b"unmanaged").unwrap();

        remove_legacy_gui_binary(&root).unwrap();

        assert!(legacy.is_file());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn managed_legacy_binary_without_association_is_removed() {
        let root = unique_temp_dir("managed-legacy");
        let legacy = root.join("app").join(binary_name("k580"));
        std::fs::create_dir_all(legacy.parent().unwrap()).unwrap();
        std::fs::write(&legacy, b"managed").unwrap();
        write_manifest(
            &root,
            &InstallManifest::new(InstallMode::Portable, InstallScope::User),
        )
        .unwrap();

        remove_legacy_gui_binary(&root).unwrap();

        assert!(!legacy.exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    fn unique_temp_dir(name: &str) -> PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("kr580-{name}-{}-{nonce}", std::process::id()))
    }
}
