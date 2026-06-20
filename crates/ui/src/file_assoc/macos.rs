use std::path::{Path, PathBuf};

pub fn register() -> Result<(), String> {
    let kr = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    let k580 = kr.with_file_name("k580");
    let bundle_dir = applications_dir().join("kr580.app");
    let contents = bundle_dir.join("Contents");
    let macos = contents.join("MacOS");
    let resources = contents.join("Resources");

    std::fs::create_dir_all(&macos).map_err(|e| format!("create MacOS: {e}"))?;
    std::fs::create_dir_all(&resources).map_err(|e| format!("create Resources: {e}"))?;

    std::fs::copy(&kr, macos.join("kr")).map_err(|e| format!("copy kr: {e}"))?;
    std::fs::copy(&k580, macos.join("k580")).map_err(|e| format!("copy k580: {e}"))?;

    let launcher = macos.join("kr580");
    std::fs::write(&launcher, launcher_script()).map_err(|e| format!("write launcher: {e}"))?;
    make_executable(&launcher)?;

    std::fs::write(contents.join("Info.plist"), info_plist())
        .map_err(|e| format!("write Info.plist: {e}"))?;

    if let Some(icon) = super::find_icon() {
        let _ = std::fs::copy(icon, resources.join("icon.png"));
    }

    register_bundle(&bundle_dir)
}

pub fn register_for_executable(
    exe: &Path,
    _scope: crate::install_mode::InstallScope,
) -> Result<(), String> {
    let bundle_dir = applications_dir().join("kr580.app");
    let contents = bundle_dir.join("Contents");
    let macos = contents.join("MacOS");
    let resources = contents.join("Resources");

    std::fs::create_dir_all(&macos).map_err(|e| format!("create MacOS: {e}"))?;
    std::fs::create_dir_all(&resources).map_err(|e| format!("create Resources: {e}"))?;

    let launcher = macos.join("kr580");
    std::fs::write(&launcher, target_launcher_script(exe))
        .map_err(|e| format!("write launcher: {e}"))?;
    make_executable(&launcher)?;

    std::fs::write(contents.join("Info.plist"), info_plist())
        .map_err(|e| format!("write Info.plist: {e}"))?;

    if let Some(icon) = super::find_icon() {
        let _ = std::fs::copy(icon, resources.join("icon.png"));
    }

    register_bundle(&bundle_dir)
}

pub fn unregister() -> Result<(), String> {
    let _ = std::fs::remove_dir_all(applications_dir().join("kr580.app"));
    Ok(())
}

pub fn unregister_for_executable(
    exe: &Path,
    _scope: crate::install_mode::InstallScope,
) -> Result<(), String> {
    let launcher = applications_dir()
        .join("kr580.app")
        .join("Contents")
        .join("MacOS")
        .join("kr580");
    let current = std::fs::read_to_string(&launcher).unwrap_or_default();
    if current == target_launcher_script(exe) {
        unregister()?;
    }
    Ok(())
}

pub fn is_registered() -> bool {
    applications_dir().join("kr580.app").is_dir()
}

fn applications_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("Applications")
}

fn launcher_script() -> &'static str {
    "#!/bin/bash\nDIR=\"$(cd \"$(dirname \"$0\")\" && pwd)\"\nexec \"$DIR/kr\" \"$@\"\n"
}

fn target_launcher_script(exe: &Path) -> String {
    format!(
        "#!/bin/bash\nexec {} \"$@\"\n",
        shell_single_quote(&exe.display().to_string())
    )
}

fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn make_executable(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path)
        .map_err(|e| format!("metadata: {e}"))?
        .permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).map_err(|e| format!("set permissions: {e}"))
}

fn info_plist() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>kr580</string>
    <key>CFBundleIdentifier</key>
    <string>com.kr580.emulator</string>
    <key>CFBundleName</key>
    <string>KR580</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeName</key>
            <string>KR580 Snapshot</string>
            <key>CFBundleTypeRole</key>
            <string>Editor</string>
            <key>LSItemContentTypes</key>
            <array>
                <string>com.kr580.snapshot</string>
            </array>
        </dict>
    </array>
    <key>UTExportedTypeDeclarations</key>
    <array>
        <dict>
            <key>UTTypeIdentifier</key>
            <string>com.kr580.snapshot</string>
            <key>UTTypeDescription</key>
            <string>KR580 Snapshot</string>
            <key>UTTypeConformsTo</key>
            <array>
                <string>public.data</string>
            </array>
            <key>UTTypeTagSpecification</key>
            <dict>
                <key>public.filename-extension</key>
                <array>
                    <string>580</string>
                </array>
            </dict>
        </dict>
    </array>
</dict>
</plist>
"#
}

fn register_bundle(bundle: &Path) -> Result<(), String> {
    let lsregister = "/System/Library/Frameworks/CoreServices.framework/Versions/A/Frameworks/LaunchServices.framework/Versions/A/Support/lsregister";
    let status = std::process::Command::new(lsregister)
        .args(["-f", &bundle.to_string_lossy()])
        .status()
        .map_err(|e| format!("lsregister failed: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err("lsregister returned non-zero".to_owned())
    }
}
