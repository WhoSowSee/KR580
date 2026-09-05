use std::path::{Path, PathBuf};

pub const APP_BUNDLE_NAME: &str = "KR580.app";
pub const BUNDLE_ID: &str = "dev.kr580.emulator";
pub const BUNDLE_EXECUTABLE: &str = "kr580";
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
    validate_launcher_bundle(bundle, executable)?;
    if !executable.is_file() {
        return Err(format!(
            "GUI executable not found: {}",
            executable.display()
        ));
    }
    let contents = bundle.join("Contents");
    let macos = contents.join("MacOS");
    let resources = contents.join("Resources");
    std::fs::create_dir_all(&macos).map_err(|error| format!("create app bundle: {error}"))?;
    std::fs::create_dir_all(&resources)
        .map_err(|error| format!("create app resources: {error}"))?;
    std::fs::write(contents.join("Info.plist"), info_plist())
        .map_err(|error| format!("write Info.plist: {error}"))?;
    std::fs::write(resources.join("KR580.icns"), APPLICATION_ICON)
        .map_err(|error| format!("write app icon: {error}"))?;
    std::fs::write(resources.join("KR580Document.icns"), DOCUMENT_ICON)
        .map_err(|error| format!("write document icon: {error}"))?;
    replace_launcher(&macos, executable)
}

pub fn validate_launcher_bundle(bundle: &Path, executable: &Path) -> Result<(), String> {
    if std::fs::symlink_metadata(bundle).is_ok()
        && !bundle_owned_by_current_install(bundle, executable)
    {
        return Err(format!(
            "{} is not managed by this KR580 installation",
            bundle.display()
        ));
    }
    Ok(())
}

fn bundle_owned_by(bundle: &Path, executable: &Path) -> bool {
    let Some(name) = bundle_executable(bundle) else {
        return false;
    };
    let paths = [
        bundle.to_path_buf(),
        bundle.join("Contents"),
        bundle.join("Contents/MacOS"),
        bundle.join("Contents/MacOS").join(name),
    ];
    if paths
        .iter()
        .any(|path| std::fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink()))
    {
        return false;
    }
    std::fs::read_to_string(paths.last().unwrap())
        .is_ok_and(|content| content == launcher_script(executable))
}

pub fn ensure_for_executable(executable: &Path) -> Result<PathBuf, String> {
    if let Some(bundle) = containing_application_bundle(executable)
        && bundle_has_identity(&bundle)
    {
        return Ok(bundle);
    }

    let bundle = managed_bundle_path(executable)?;
    if !bundle_owned_by(&bundle, executable) || !bundle_has_identity(&bundle) {
        write_launcher_bundle(&bundle, executable)?;
    }
    Ok(bundle)
}

pub fn remove_launcher_bundle(bundle: &Path, executable: &Path) -> Result<bool, String> {
    if !bundle_owned_by_current_install(bundle, executable) {
        return Ok(false);
    }
    std::fs::remove_dir_all(bundle)
        .map_err(|error| format!("remove {}: {error}", bundle.display()))?;
    Ok(true)
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
    bundle_executable(bundle).is_some_and(|name| name == BUNDLE_EXECUTABLE)
}

fn bundle_executable(bundle: &Path) -> Option<String> {
    let plist = std::fs::read_to_string(bundle.join("Contents/Info.plist")).ok()?;
    let document = roxmltree::Document::parse_with_options(
        &plist,
        roxmltree::ParsingOptions {
            allow_dtd: true,
            ..Default::default()
        },
    )
    .ok()?;
    let dictionary = document
        .root_element()
        .children()
        .find(|node| node.has_tag_name("dict"))?;
    let value = |key| {
        dictionary
            .children()
            .find(|node| node.has_tag_name("key") && node.text() == Some(key))
            .and_then(|node| node.next_sibling_element())
            .filter(|node| node.has_tag_name("string"))
            .and_then(|node| node.text())
    };
    if value("CFBundleIdentifier")? != BUNDLE_ID {
        return None;
    }
    let name = value("CFBundleExecutable")?;
    matches!(name, "kr580" | "KR580" | "kr580-launcher").then(|| name.to_owned())
}

fn managed_bundle_path(executable: &Path) -> Result<PathBuf, String> {
    if let Some((root, manifest)) = crate::install_mode::manifest_for_executable(executable)?
        && manifest.mode == crate::install_mode::InstallMode::Portable
    {
        return Ok(root.join(APP_BUNDLE_NAME));
    }
    Ok(applications_dir()?.join(APP_BUNDLE_NAME))
}

fn bundle_owned_by_current_install(bundle: &Path, executable: &Path) -> bool {
    bundle_owned_by(bundle, executable)
        || bundle_owned_by(bundle, &executable.with_file_name("k580"))
}

pub fn info_plist() -> String {
    include_str!("../assets/macos/Info.plist").replace("@VERSION@", env!("CARGO_PKG_VERSION"))
}

pub fn launcher_script(executable: &Path) -> String {
    format!(
        "#!/bin/sh\nexec {} \"$@\"\n",
        crate::shell_quote::single(&executable.display().to_string())
    )
}

fn make_executable(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = std::fs::metadata(path)
        .map_err(|error| format!("metadata: {error}"))?
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).map_err(|error| format!("set permissions: {error}"))
}

fn replace_launcher(macos: &Path, executable: &Path) -> Result<(), String> {
    let staged = macos.join(".kr580-launcher");
    std::fs::write(&staged, launcher_script(executable))
        .map_err(|error| format!("write app launcher: {error}"))?;
    make_executable(&staged)?;
    let result = remove_file_if_exists(&macos.join("KR580"))
        .and_then(|()| remove_file_if_exists(&macos.join("kr580-launcher")))
        .and_then(|()| remove_file_if_exists(&macos.join(BUNDLE_EXECUTABLE)))
        .and_then(|()| {
            std::fs::rename(&staged, macos.join(BUNDLE_EXECUTABLE))
                .map_err(|error| format!("install app launcher: {error}"))
        });
    if result.is_err() {
        let _ = std::fs::remove_file(staged);
    }
    result
}

fn remove_file_if_exists(path: &Path) -> Result<(), String> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("remove {}: {error}", path.display())),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        APP_BUNDLE_NAME, BUNDLE_EXECUTABLE, BUNDLE_ID, SNAPSHOT_UTI, SUBPROGRAM_UTI,
        bundle_owned_by, containing_application_bundle, info_plist, launcher_script,
        remove_launcher_bundle, write_launcher_bundle,
    };
    use std::path::{Path, PathBuf};

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
            launcher_script(Path::new("/Applications/Jack's KR580/kr580")),
            "#!/bin/sh\nexec '/Applications/Jack'\\''s KR580/kr580' \"$@\"\n"
        );
    }

    #[test]
    fn containing_bundle_is_resolved_from_macos_executable_path() {
        assert_eq!(
            containing_application_bundle(Path::new(
                "/Applications/KR580.app/Contents/MacOS/kr580"
            )),
            Some(Path::new("/Applications/KR580.app").to_path_buf())
        );
        assert_eq!(containing_application_bundle(Path::new("/opt/kr580")), None);
    }

    #[test]
    fn writer_and_remover_preserve_foreign_bundles() {
        let root = unique_temp_dir();
        let bundle = root.join(APP_BUNDLE_NAME);
        let launcher = bundle.join("Contents/MacOS/kr580");
        std::fs::create_dir_all(launcher.parent().unwrap()).unwrap();
        std::fs::write(&launcher, "foreign").unwrap();
        let executable = root.join("install/app/kr580");

        assert!(write_launcher_bundle(&bundle, &executable).is_err());
        assert!(!remove_launcher_bundle(&bundle, &executable).unwrap());
        assert_eq!(std::fs::read_to_string(launcher).unwrap(), "foreign");

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn owned_bundle_can_be_replaced_and_removed() {
        let root = unique_temp_dir();
        let bundle = root.join(APP_BUNDLE_NAME);
        let executable = root.join("install/app/kr580");

        std::fs::create_dir_all(executable.parent().unwrap()).unwrap();
        std::fs::write(&executable, b"fixture").unwrap();

        write_launcher_bundle(&bundle, &executable).unwrap();
        assert!(bundle_owned_by(&bundle, &executable));
        let other = root.join("other/app/kr580");
        assert!(write_launcher_bundle(&bundle, &other).is_err());
        assert!(!remove_launcher_bundle(&bundle, &other).unwrap());
        assert!(bundle_owned_by(&bundle, &executable));
        assert!(remove_launcher_bundle(&bundle, &executable).unwrap());
        assert!(!bundle.exists());

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn portable_association_creates_a_bundle_inside_its_install_root() {
        let root = unique_temp_dir();
        let executable = root.join("app/kr580");
        std::fs::create_dir_all(executable.parent().unwrap()).unwrap();
        std::fs::write(&executable, b"fixture").unwrap();
        crate::install_mode::write_manifest(
            &root,
            &crate::install_mode::InstallManifest::new(
                crate::install_mode::InstallMode::Portable,
                crate::install_mode::InstallScope::User,
            ),
        )
        .unwrap();
        let bundle = super::ensure_for_executable(&executable).unwrap();
        assert_eq!(bundle, root.join(APP_BUNDLE_NAME));
        assert_eq!(super::ensure_for_executable(&executable).unwrap(), bundle);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn standalone_bundle_with_leftover_owned_script_is_preserved() {
        let root = unique_temp_dir();
        let executable = root.join("app/kr580");
        let bundle = root.join(APP_BUNDLE_NAME);
        std::fs::create_dir_all(bundle.join("Contents/MacOS")).unwrap();
        std::fs::write(bundle.join("Contents/Info.plist"), info_plist()).unwrap();
        std::fs::write(bundle.join("Contents/MacOS/kr580"), b"standalone payload").unwrap();
        std::fs::write(
            bundle.join("Contents/MacOS/kr580-launcher"),
            launcher_script(&executable),
        )
        .unwrap();
        assert!(!bundle_owned_by(&bundle, &executable));
        assert!(write_launcher_bundle(&bundle, &executable).is_err());
        assert!(!remove_launcher_bundle(&bundle, &executable).unwrap());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn legacy_owned_bundle_is_upgraded_to_current_launcher() {
        let root = unique_temp_dir();
        let bundle = root.join(APP_BUNDLE_NAME);
        let executable = root.join("app/kr580");
        let macos = bundle.join("Contents/MacOS");
        std::fs::create_dir_all(&macos).unwrap();
        std::fs::create_dir_all(executable.parent().unwrap()).unwrap();
        std::fs::write(&executable, b"fixture").unwrap();
        std::fs::write(
            bundle.join("Contents/Info.plist"),
            info_plist().replace("<string>kr580</string>", "<string>kr580-launcher</string>"),
        )
        .unwrap();
        std::fs::write(
            macos.join("kr580-launcher"),
            launcher_script(&executable.with_file_name("k580")),
        )
        .unwrap();
        write_launcher_bundle(&bundle, &executable).unwrap();
        assert!(bundle_owned_by(&bundle, &executable));
        assert!(!macos.join("kr580-launcher").exists());
        assert_eq!(super::bundle_executable(&bundle).as_deref(), Some("kr580"));
        std::fs::remove_dir_all(root).unwrap();
    }

    fn unique_temp_dir() -> PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("kr580-macos-bundle-{}-{nonce}", std::process::id()))
    }
}
