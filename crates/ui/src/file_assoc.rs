#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
pub use linux::{
    is_registered, register, register_for_executable, unregister, unregister_for_executable,
};
#[cfg(target_os = "macos")]
pub use macos::{
    is_registered, register, register_for_executable, unregister, unregister_for_executable,
};
#[cfg(target_os = "windows")]
pub use windows::{
    is_registered, register, register_for_executable, unregister, unregister_for_executable,
};

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn is_registered() -> bool {
    false
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
