use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::Child;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(all(test, not(target_os = "macos")))]
mod tests;

#[derive(Clone, Debug)]
pub(crate) struct PreparedUpdate {
    pub(crate) restart_flag: PathBuf,
    pub(crate) version: String,
}

pub(super) fn prepare(
    archive: &Path,
    directory: &Path,
    version: String,
) -> Result<PreparedUpdate, String> {
    #[cfg(target_os = "macos")]
    {
        macos::prepare(archive, directory, version)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let exe = std::env::current_exe().map_err(|error| error.to_string())?;
        let (root, _) = k580_ui::install_mode::manifest_for_executable(&exe)
            .map_err(|error| error.to_string())?
            .ok_or("installation manifest is missing")?;
        let helper = extract_installer(archive, directory, &version)?;
        let restart_flag = directory.join("restart.json");
        let log = std::fs::File::create(directory.join("update.log"))
            .map_err(|error| error.to_string())?;
        let mut command = std::process::Command::new(&helper);
        command
            .arg("--apply-update")
            .arg(&root)
            .arg(std::process::id().to_string())
            .arg(&restart_flag)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::from(log));
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(windows_sys::Win32::System::Threading::CREATE_NO_WINDOW);
        }
        let mut child = command.spawn().map_err(|error| error.to_string())?;
        wait_for_ready(&mut child, directory)?;
        Ok(PreparedUpdate {
            restart_flag,
            version,
        })
    }
}

fn wait_for_ready(child: &mut Child, directory: &Path) -> Result<(), String> {
    let output = child.stdout.take().ok_or("update helper has no output")?;
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut ready = String::new();
        let result = BufReader::new(output).read_line(&mut ready).map(|_| ready);
        let _ = sender.send(result);
    });
    let ready = match receiver.recv_timeout(std::time::Duration::from_secs(60)) {
        Ok(Ok(ready)) => ready,
        result => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!("update helper did not become ready: {result:?}"));
        }
    };
    if ready.trim() != "READY" {
        let _ = child.kill();
        let _ = child.wait();
        let details = std::fs::read_to_string(directory.join("update.log")).unwrap_or_default();
        return Err(format!("update preparation failed: {}", details.trim()));
    }
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn extract_installer(archive: &Path, directory: &Path, version: &str) -> Result<PathBuf, String> {
    let mut zip =
        zip::ZipArchive::new(std::fs::File::open(archive).map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())?;
    let filename = if cfg!(windows) {
        "KR580-Setup.exe"
    } else {
        "KR580-Setup"
    };
    let expected = format!(
        "KR580-Setup-{version}-{}{}",
        env!("KR580_BUILD_TARGET"),
        if cfg!(windows) { ".exe" } else { "" },
    );
    let mut candidates = Vec::new();
    for index in 0..zip.len() {
        let entry = zip.by_index(index).map_err(|error| error.to_string())?;
        if entry.enclosed_name().is_some_and(|path| {
            path.file_name()
                .is_some_and(|name| name == expected.as_str())
        }) && entry.is_file()
        {
            candidates.push(index);
        }
    }
    if candidates.len() != 1 {
        return Err("update archive must contain exactly one installer".into());
    }
    let mut entry = zip
        .by_index(candidates[0])
        .map_err(|error| error.to_string())?;
    if entry.size() > 1024 * 1024 * 1024 {
        return Err("installer is too large".into());
    }
    let destination = directory.join(filename);
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&destination)
        .map_err(|error| error.to_string())?;
    std::io::copy(&mut entry, &mut file).map_err(|error| error.to_string())?;
    file.sync_all().map_err(|error| error.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&destination, std::fs::Permissions::from_mode(0o755))
            .map_err(|error| error.to_string())?;
    }
    Ok(destination)
}
