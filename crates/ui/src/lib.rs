#[doc(hidden)]
pub mod backend;
#[doc(hidden)]
pub mod desktop_entry;
#[doc(hidden)]
pub mod devices;
#[doc(hidden)]
pub mod persistence;

pub mod file_assoc;
pub mod install_mode;
#[doc(hidden)]
pub mod integration_assets;
pub mod system_locale;

#[cfg(target_os = "macos")]
#[doc(hidden)]
pub mod macos_bundle;
#[cfg(target_os = "macos")]
#[doc(hidden)]
pub mod macos_launch_services;
