use crate::install_mode::InstallScope;
use crate::macos_bundle::{
    APP_BUNDLE_NAME, applications_dir, bundle_owned_by, write_launcher_bundle,
};
use std::path::{Path, PathBuf};

pub fn register() -> Result<(), String> {
    let executable = std::env::current_exe().map_err(|error| format!("current_exe: {error}"))?;
    register_for_executable(&executable, InstallScope::User)
}

pub fn register_for_executable(executable: &Path, _scope: InstallScope) -> Result<(), String> {
    let executable = association_executable_from(executable.to_path_buf());
    let bundle = applications_dir()?.join(APP_BUNDLE_NAME);
    write_launcher_bundle(&bundle, &executable)?;
    register_bundle(&bundle)
}

pub fn unregister() -> Result<(), String> {
    let executable = std::env::current_exe().map_err(|error| format!("current_exe: {error}"))?;
    unregister_for_executable(&executable, InstallScope::User)
}

pub fn unregister_for_executable(executable: &Path, _scope: InstallScope) -> Result<(), String> {
    let executable = association_executable_from(executable.to_path_buf());
    let bundle = applications_dir()?.join(APP_BUNDLE_NAME);
    if bundle_owned_by(&bundle, &executable) {
        std::fs::remove_dir_all(bundle).map_err(|error| format!("remove app bundle: {error}"))?;
    }
    Ok(())
}

pub fn is_registered() -> bool {
    let Ok(executable) = std::env::current_exe() else {
        return false;
    };
    let executable = association_executable_from(executable);
    applications_dir()
        .map(|directory| directory.join(APP_BUNDLE_NAME))
        .is_ok_and(|bundle| bundle_owned_by(&bundle, &executable))
}

fn association_executable_from(executable: PathBuf) -> PathBuf {
    if executable.file_name().is_some_and(|name| name == "kr") {
        if let Some(directory) = executable.parent()
            && directory.file_name().is_some_and(|name| name == "bin")
            && let Some(root) = directory.parent()
            && root.join(crate::install_mode::MANIFEST_FILENAME).is_file()
        {
            return root.join("app/k580");
        }
        return executable.with_file_name("k580");
    }
    executable
}

fn register_bundle(bundle: &Path) -> Result<(), String> {
    let lsregister = "/System/Library/Frameworks/CoreServices.framework/Versions/A/Frameworks/LaunchServices.framework/Versions/A/Support/lsregister";
    let status = std::process::Command::new(lsregister)
        .args(["-f", &bundle.to_string_lossy()])
        .status()
        .map_err(|error| format!("lsregister: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("lsregister exited with {status}"))
    }
}

#[cfg(test)]
mod tests {
    use super::association_executable_from;
    use std::path::PathBuf;

    #[test]
    fn adjacent_launcher_resolves_to_gui() {
        assert_eq!(
            association_executable_from(PathBuf::from("/opt/kr580/kr")),
            PathBuf::from("/opt/kr580/k580")
        );
    }
}
