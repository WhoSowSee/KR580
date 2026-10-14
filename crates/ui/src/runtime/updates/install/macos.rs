use super::PreparedUpdate;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

struct MountedImage(PathBuf);

impl Drop for MountedImage {
    fn drop(&mut self) {
        let _ = Command::new("/usr/bin/hdiutil")
            .args(["detach", "-quiet"])
            .arg(&self.0)
            .status();
    }
}

pub(super) fn prepare(
    archive: &Path,
    directory: &Path,
    version: String,
) -> Result<PreparedUpdate, String> {
    let exe = std::env::current_exe().map_err(|error| error.to_string())?;
    let bundle = exe
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .ok_or("application bundle is missing")?;
    if std::fs::symlink_metadata(bundle)
        .map_err(|error| error.to_string())?
        .file_type()
        .is_symlink()
    {
        return Err("cannot replace a symlinked application bundle".into());
    }
    let stage = tempfile::Builder::new()
        .prefix(".kr580-update-")
        .tempdir_in(bundle.parent().ok_or("bundle has no parent")?)
        .map_err(|error| error.to_string())?;
    let mount = directory.join("mount");
    std::fs::create_dir(&mount).map_err(|error| error.to_string())?;
    run(Command::new("/usr/bin/hdiutil")
        .args(["attach", "-quiet", "-readonly", "-nobrowse", "-mountpoint"])
        .arg(&mount)
        .arg(archive))?;
    let mounted = MountedImage(mount);
    let incoming = stage.path().join("KR580.app");
    run(Command::new("/usr/bin/ditto")
        .arg(mounted.0.join("KR580.app"))
        .arg(&incoming))?;
    let plist = incoming.join("Contents/Info.plist");
    for (key, expected) in [
        ("CFBundleIdentifier", "dev.kr580.emulator"),
        ("CFBundleShortVersionString", version.as_str()),
    ] {
        let output = Command::new("/usr/bin/plutil")
            .args(["-extract", key, "raw", "-o", "-"])
            .arg(&plist)
            .output()
            .map_err(|error| error.to_string())?;
        if !output.status.success() || String::from_utf8_lossy(&output.stdout).trim() != expected {
            return Err("downloaded application identity or version does not match".into());
        }
    }
    drop(mounted);
    let quote = |path: &Path| -> Result<String, String> {
        path.to_str()
            .map(k580_ui::shell_quote::single)
            .ok_or_else(|| "update path is not UTF-8".into())
    };
    let restart_flag = directory.join("restart.json");
    let backup = stage.path().join("previous.app");
    let script = format!(
        "#!/bin/sh\nset -eu\nprintf 'READY\\n'\nwhile /bin/kill -0 {pid} 2>/dev/null; do sleep 0.1; done\n/bin/mv {bundle} {backup}\nif /bin/mv {incoming} {bundle}; then /bin/rm -rf {backup}; else /bin/mv {backup} {bundle}; exit 1; fi\nif test -f {restart}; then\nif document=$(/usr/bin/plutil -extract 0 raw -o - {restart} 2>/dev/null); then /usr/bin/open -n {bundle} --args \"$document\"; else /usr/bin/open -n {bundle}; fi\nfi\n/bin/rmdir {stage}\n/bin/rm -rf {cache}\n",
        pid = std::process::id(),
        bundle = quote(bundle)?,
        backup = quote(&backup)?,
        incoming = quote(&incoming)?,
        restart = quote(&restart_flag)?,
        stage = quote(stage.path())?,
        cache = quote(directory)?,
    );
    let helper = directory.join("apply-update.sh");
    std::fs::write(&helper, script).map_err(|error| error.to_string())?;
    let log =
        std::fs::File::create(directory.join("update.log")).map_err(|error| error.to_string())?;
    let mut child = Command::new("/bin/sh")
        .arg(&helper)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::from(log))
        .spawn()
        .map_err(|error| error.to_string())?;
    super::wait_for_ready(&mut child, directory)?;
    let _ = stage.keep();
    Ok(PreparedUpdate {
        restart_flag,
        version,
    })
}

fn run(command: &mut Command) -> Result<(), String> {
    let output = command.output().map_err(|error| error.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_owned())
    }
}
