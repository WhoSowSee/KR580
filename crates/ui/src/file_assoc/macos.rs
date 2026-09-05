use crate::install_mode::InstallScope;
use std::path::Path;

pub fn register() -> Result<(), String> {
    let executable = crate::install_mode::current_integration_executable()?;
    register_for_executable(&executable, InstallScope::User)
}

pub fn register_for_executable(executable: &Path, _scope: InstallScope) -> Result<(), String> {
    let executable = crate::install_mode::companion_executable_from_launcher(
        executable.to_path_buf(),
        "kr",
        "kr580",
    );
    let bundle = crate::macos_bundle::ensure_for_executable(&executable)?;
    crate::macos_launch_services::register_bundle(&bundle)?;
    crate::macos_launch_services::set_default_handlers()
}

pub fn unregister() -> Result<(), String> {
    Err("macOS cannot remove a default file handler without choosing a replacement".to_owned())
}

pub fn unregister_for_executable(_executable: &Path, _scope: InstallScope) -> Result<(), String> {
    unregister()
}
