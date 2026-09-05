#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
mod linux_default;
#[cfg(target_os = "linux")]
mod linux_files;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
pub use linux::{register, register_for_executable, unregister, unregister_for_executable};
#[cfg(target_os = "macos")]
pub use macos::{register, register_for_executable, unregister, unregister_for_executable};
#[cfg(target_os = "windows")]
pub use windows::{
    is_registered, register, register_for_executable, unregister, unregister_for_executable,
};

#[cfg(target_os = "linux")]
pub fn is_user_configurable() -> bool {
    snap_environment_allows_configuration(std::env::var_os("SNAP"))
}

pub const fn can_unregister() -> bool {
    cfg!(any(target_os = "windows", target_os = "linux"))
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
pub fn is_user_configurable() -> bool {
    true
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn is_user_configurable() -> bool {
    false
}

#[cfg(any(target_os = "linux", test))]
fn snap_environment_allows_configuration(root: Option<std::ffi::OsString>) -> bool {
    root.is_none_or(|root| root.is_empty())
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn register() -> Result<(), String> {
    Err("file-type association is not supported on this platform".to_owned())
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn unregister() -> Result<(), String> {
    Err("file-type association is not supported on this platform".to_owned())
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn register_for_executable(
    _exe: &std::path::Path,
    _scope: crate::install_mode::InstallScope,
) -> Result<(), String> {
    Err("file-type association is not supported on this platform".to_owned())
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn unregister_for_executable(
    _exe: &std::path::Path,
    _scope: crate::install_mode::InstallScope,
) -> Result<(), String> {
    Err("file-type association is not supported on this platform".to_owned())
}

#[cfg(test)]
mod tests {
    use super::snap_environment_allows_configuration;

    #[test]
    fn snap_environment_disables_host_association_changes() {
        assert!(snap_environment_allows_configuration(None));
        assert!(snap_environment_allows_configuration(Some("".into())));
        assert!(!snap_environment_allows_configuration(Some(
            "/snap/kr580/current".into()
        )));
    }
}
