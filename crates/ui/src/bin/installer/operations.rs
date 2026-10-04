mod journal;
mod native;
mod source;
mod transaction;
use super::platform;
use k580_ui::install_mode::{InstallManifest, InstallMode, InstallScope};
use source::SourceBundle;
use std::path::{Path, PathBuf};

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

pub(super) fn install(mut request: InstallRequest) -> Result<InstallReport, String> {
    request.install_dir = journal::absolute(&request.install_dir)?;
    let source = SourceBundle::discover()?;
    let mut integration = native::NativeIntegration::capture(&request)?;
    transaction::run(request, source, &mut integration, |_| Ok(()))
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
}
