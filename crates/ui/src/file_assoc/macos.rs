use crate::install_mode::InstallScope;
use crate::macos_bundle::find_for_executable;
use std::path::{Path, PathBuf};

pub fn register() -> Result<(), String> {
    let executable = std::env::current_exe().map_err(|error| format!("current_exe: {error}"))?;
    register_for_executable(&executable, InstallScope::User)
}

pub fn register_for_executable(executable: &Path, _scope: InstallScope) -> Result<(), String> {
    let executable = association_executable_from(executable.to_path_buf());
    let bundle = find_for_executable(&executable)?;
    crate::macos_launch_services::register_bundle(&bundle)
}

pub fn unregister() -> Result<(), String> {
    let executable = std::env::current_exe().map_err(|error| format!("current_exe: {error}"))?;
    unregister_for_executable(&executable, InstallScope::User)
}

pub fn unregister_for_executable(_executable: &Path, _scope: InstallScope) -> Result<(), String> {
    Ok(())
}

pub fn is_registered() -> bool {
    std::env::current_exe()
        .map(association_executable_from)
        .is_ok_and(|executable| find_for_executable(&executable).is_ok())
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
