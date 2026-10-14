use super::journal::{FileJournal, absolute, sibling};
use super::source::{SourceBundle, copy_executable};
use super::{binary_name, read_manifest};
use std::io::Write;
use std::path::{Path, PathBuf};

#[cfg(test)]
mod tests;

struct PendingFiles {
    _lease: super::transaction::Lease,
    files: FileJournal,
    staged: Vec<(PathBuf, PathBuf)>,
}

impl PendingFiles {
    fn prepare(root: &Path, source: SourceBundle) -> Result<Self, String> {
        let root = absolute(root)?
            .canonicalize()
            .map_err(|error| error.to_string())?;
        if read_manifest(&root)?.manifest_version != 1 {
            return Err("unsupported install manifest".into());
        }
        let targets = [
            root.join("app").join(binary_name("kr580")),
            root.join("bin").join(binary_name("kr")),
            root.join("app").join(binary_name("uninstaller")),
        ];
        for target in &targets {
            let metadata = std::fs::symlink_metadata(target).map_err(|error| error.to_string())?;
            if !metadata.is_file() || metadata.file_type().is_symlink() {
                return Err("update target must be an existing regular file".into());
            }
        }
        let mut pending = Self {
            _lease: super::transaction::Lease::acquire(&root)?,
            files: FileJournal::default(),
            staged: Vec::new(),
        };
        pending.files.prepare(&targets)?;
        for (source, target) in [&source.kr580, &source.kr, &source.uninstaller]
            .into_iter()
            .zip(targets)
        {
            let stage = sibling(&target, "update")?;
            pending.staged.push((stage.clone(), target));
            copy_executable(source, &stage)?;
        }
        Ok(pending)
    }

    fn apply(&mut self) -> Result<(), String> {
        for (stage, target) in &self.staged {
            if let Err(error) = self.files.change(std::slice::from_ref(target), || {
                std::fs::rename(stage, target).map_err(|error| error.to_string())
            }) {
                return match self.files.rollback() {
                    Ok(()) => Err(error),
                    Err(rollback) => Err(format!("{error}; rollback: {rollback}")),
                };
            }
        }
        self.files.commit();
        Ok(())
    }
}

impl Drop for PendingFiles {
    fn drop(&mut self) {
        for (stage, _) in &self.staged {
            let _ = std::fs::remove_file(stage);
        }
    }
}

pub(in crate::installer) fn run(
    root: PathBuf,
    pid: u32,
    restart_flag: PathBuf,
) -> Result<(), String> {
    if pid == 0 || pid == std::process::id() {
        return Err("invalid parent process".into());
    }
    let wait = ParentProcess::open(pid)?;
    let mut pending = PendingFiles::prepare(&root, SourceBundle::for_update()?)?;
    println!("READY");
    std::io::stdout()
        .flush()
        .map_err(|error| error.to_string())?;
    wait.wait()?;
    let result = pending.apply();
    if let Err(error) = &result {
        let _ = std::fs::write(root.join("update-error.log"), error);
    }
    if restart_flag.is_file() {
        let bytes = std::fs::read(&restart_flag).map_err(|error| error.to_string())?;
        let paths: Vec<PathBuf> =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        if paths.len() > 1 {
            return Err("invalid restart arguments".into());
        }
        let mut command = std::process::Command::new(root.join("app").join(binary_name("kr580")));
        command.args(paths);
        command.spawn().map_err(|error| error.to_string())?;
    }
    if result.is_ok() {
        remove_cache(&restart_flag);
    }
    result
}

fn remove_cache(restart_flag: &Path) {
    let cache = restart_flag
        .parent()
        .and_then(|path| path.canonicalize().ok());
    let exe = std::env::current_exe().and_then(|path| path.canonicalize());
    let temporary = std::env::temp_dir().canonicalize();
    if let (Some(cache), Ok(exe), Ok(temporary)) = (cache, exe, temporary)
        && exe.parent() == Some(cache.as_path())
        && cache.parent() == Some(temporary.as_path())
        && cache
            .file_name()
            .is_some_and(|name| name.to_string_lossy().starts_with("kr580-update-"))
        && let Err(error) = crate::installer::platform::schedule_remove_install_dir(&cache)
    {
        tracing::warn!(%error, "update cache cleanup failed");
    }
}

#[cfg(windows)]
struct ParentProcess(windows_sys::Win32::Foundation::HANDLE);

#[cfg(windows)]
impl ParentProcess {
    fn open(pid: u32) -> Result<Self, String> {
        // SAFETY: OpenProcess receives a validated PID and only requests synchronization access.
        let handle =
            unsafe { windows_sys::Win32::System::Threading::OpenProcess(0x00100000, 0, pid) };
        if handle.is_null() {
            return Err(std::io::Error::last_os_error().to_string());
        }
        Ok(Self(handle))
    }
    fn wait(&self) -> Result<(), String> {
        // SAFETY: The owned process handle stays valid until Drop and is used only for waiting.
        let status =
            unsafe { windows_sys::Win32::System::Threading::WaitForSingleObject(self.0, u32::MAX) };
        if status == 0 {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error().to_string())
        }
    }
}

#[cfg(windows)]
impl Drop for ParentProcess {
    fn drop(&mut self) {
        // SAFETY: This handle is owned by ParentProcess and is closed exactly once.
        unsafe {
            windows_sys::Win32::Foundation::CloseHandle(self.0);
        }
    }
}

#[cfg(unix)]
struct ParentProcess(u32);

#[cfg(unix)]
impl ParentProcess {
    fn open(pid: u32) -> Result<Self, String> {
        if pid > i32::MAX as u32 {
            return Err("parent process ID is out of range".into());
        }
        Ok(Self(pid))
    }
    fn wait(&self) -> Result<(), String> {
        loop {
            // SAFETY: Signal zero checks existence for a validated positive PID without sending a signal.
            if unsafe { libc::kill(self.0 as i32, 0) } != 0 {
                if std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH) {
                    return Ok(());
                }
                return Err(std::io::Error::last_os_error().to_string());
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}
