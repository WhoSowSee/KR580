use std::path::{Path, PathBuf};

pub const APP_BUNDLE_NAME: &str = "KR580.app";
pub const BUNDLE_ID: &str = "dev.kr580.emulator";
pub const BUNDLE_EXECUTABLE: &str = "KR580";
pub const SNAPSHOT_UTI: &str = "dev.kr580.snapshot";
pub const SUBPROGRAM_UTI: &str = "dev.kr580.subprogram";

const APPLICATION_ICON: &[u8] = include_bytes!("../assets/icons/KR580.icns");
const DOCUMENT_ICON: &[u8] = include_bytes!("../assets/icons/KR580Document.icns");

pub fn applications_dir() -> Result<PathBuf, String> {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .ok_or_else(|| "HOME is missing or is not an absolute path".to_owned())?;
    Ok(home.join("Applications"))
}

pub fn write_launcher_bundle(bundle: &Path, executable: &Path) -> Result<(), String> {
    let contents = bundle.join("Contents");
    let macos = contents.join("MacOS");
    let resources = contents.join("Resources");
    std::fs::create_dir_all(&macos).map_err(|error| format!("create app bundle: {error}"))?;
    std::fs::create_dir_all(&resources)
        .map_err(|error| format!("create app resources: {error}"))?;
    std::fs::write(contents.join("Info.plist"), info_plist())
        .map_err(|error| format!("write Info.plist: {error}"))?;
    let launcher = macos.join(BUNDLE_EXECUTABLE);
    std::fs::write(&launcher, launcher_script(executable))
        .map_err(|error| format!("write app launcher: {error}"))?;
    std::fs::write(resources.join("KR580.icns"), APPLICATION_ICON)
        .map_err(|error| format!("write app icon: {error}"))?;
    std::fs::write(resources.join("KR580Document.icns"), DOCUMENT_ICON)
        .map_err(|error| format!("write document icon: {error}"))?;
    make_executable(&launcher)
}

pub fn bundle_owned_by(bundle: &Path, executable: &Path) -> bool {
    let launcher = bundle.join("Contents/MacOS").join(BUNDLE_EXECUTABLE);
    std::fs::read_to_string(launcher).is_ok_and(|content| content == launcher_script(executable))
}

pub fn find_for_executable(executable: &Path) -> Result<PathBuf, String> {
    if let Some(bundle) = containing_application_bundle(executable)
        && bundle_has_identity(&bundle)
    {
        return Ok(bundle);
    }

    let bundle = applications_dir()?.join(APP_BUNDLE_NAME);
    if bundle_owned_by(&bundle, executable) && bundle_has_identity(&bundle) {
        return Ok(bundle);
    }

    Err(format!(
        "KR580.app containing {} was not found",
        executable.display()
    ))
}

pub fn containing_application_bundle(executable: &Path) -> Option<PathBuf> {
    executable
        .ancestors()
        .find(|ancestor| {
            ancestor
                .extension()
                .is_some_and(|extension| extension == "app")
        })
        .map(Path::to_path_buf)
}

fn bundle_has_identity(bundle: &Path) -> bool {
    std::fs::read_to_string(bundle.join("Contents/Info.plist")).is_ok_and(|plist| {
        plist.contains(&format!("<string>{BUNDLE_ID}</string>"))
            && plist.contains(&format!("<string>{BUNDLE_EXECUTABLE}</string>"))
    })
}

pub fn info_plist() -> String {
    include_str!("../assets/macos/Info.plist").replace("@VERSION@", env!("CARGO_PKG_VERSION"))
}

fn launcher_script(executable: &Path) -> String {
    format!(
        "#!/bin/sh\nexec {} \"$@\"\n",
        shell_single_quote(&executable.display().to_string())
    )
}

fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn make_executable(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = std::fs::metadata(path)
        .map_err(|error| format!("metadata: {error}"))?
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).map_err(|error| format!("set permissions: {error}"))
}

#[cfg(test)]
mod tests {
    use super::{
        BUNDLE_EXECUTABLE, BUNDLE_ID, SNAPSHOT_UTI, SUBPROGRAM_UTI, containing_application_bundle,
        info_plist, launcher_script,
    };
    use std::path::Path;

    #[test]
    fn metadata_and_launcher_share_the_bundle_contract() {
        let plist = info_plist();
        assert!(plist.contains(&format!("<string>{BUNDLE_ID}</string>")));
        assert!(plist.contains(&format!("<string>{BUNDLE_EXECUTABLE}</string>")));
        assert!(plist.contains(&format!("<string>{SNAPSHOT_UTI}</string>")));
        assert!(plist.contains(&format!("<string>{SUBPROGRAM_UTI}</string>")));
        assert!(plist.contains("<string>KR580.icns</string>"));
        assert!(plist.contains("<string>KR580Document.icns</string>"));
        assert!(plist.contains("<string>580</string>"));
        assert!(plist.contains("<string>krs</string>"));
        assert!(!plist.contains("@VERSION@"));
        assert_eq!(
            launcher_script(Path::new("/Applications/Jack's KR580/k580")),
            "#!/bin/sh\nexec '/Applications/Jack'\\''s KR580/k580' \"$@\"\n"
        );
    }

    #[test]
    fn containing_bundle_is_resolved_from_macos_executable_path() {
        assert_eq!(
            containing_application_bundle(Path::new(
                "/Applications/KR580.app/Contents/MacOS/KR580"
            )),
            Some(Path::new("/Applications/KR580.app").to_path_buf())
        );
        assert_eq!(containing_application_bundle(Path::new("/opt/k580")), None);
    }
}
