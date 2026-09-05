use std::fs::{File, OpenOptions, Permissions};
use std::io::Write;
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

pub(super) fn acquire_lock(data_home: &Path) -> Result<File, String> {
    let directory = data_home.join("kr580");
    std::fs::create_dir_all(&directory).map_err(|e| format!("create lock directory: {e}"))?;
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(directory.join("file-association.lock"))
        .map_err(|e| format!("open association lock: {e}"))?;
    // SAFETY: The file descriptor is live and flock does not retain a Rust pointer.
    if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) } != 0 {
        return Err(format!(
            "lock file associations: {}",
            std::io::Error::last_os_error()
        ));
    }
    // Closing the owned File releases flock; keep it alive for the entire operation.
    Ok(file)
}

struct SavedFile {
    path: PathBuf,
    contents: Option<(Vec<u8>, Permissions)>,
}

pub(super) struct Backup(Vec<SavedFile>);

impl Backup {
    pub(super) fn capture(paths: impl IntoIterator<Item = PathBuf>) -> Result<Self, String> {
        let mut files = Vec::new();
        for path in paths {
            let contents = read_optional(&path)?
                .map(|bytes| {
                    std::fs::metadata(&path)
                        .map(|metadata| (bytes, metadata.permissions()))
                        .map_err(|e| format!("metadata {}: {e}", path.display()))
                })
                .transpose()?;
            files.push(SavedFile { path, contents });
        }
        Ok(Self(files))
    }

    pub(super) fn restore(&self) -> Result<(), String> {
        let mut errors = Vec::new();
        for SavedFile { path, contents } in &self.0 {
            let result = match contents {
                Some((bytes, permissions)) => atomic_write(path, bytes).and_then(|()| {
                    std::fs::set_permissions(path, permissions.clone())
                        .map_err(|e| format!("restore permissions {}: {e}", path.display()))
                }),
                None => remove_file_if_exists(path),
            };
            if let Err(error) = result {
                errors.push(error);
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("; "))
        }
    }
}

pub(super) fn read_optional(path: &Path) -> Result<Option<Vec<u8>>, String> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("read {}: {e}", path.display())),
    }
}

pub(super) fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let parent = path
        .parent()
        .ok_or_else(|| "file has no parent directory".to_owned())?;
    std::fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
    let staged = parent.join(format!(
        ".kr580-{}-{}.tmp",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&staged)
        .map_err(|e| format!("create {}: {e}", staged.display()))?;
    let result = (|| {
        if let Ok(metadata) = std::fs::metadata(path) {
            file.set_permissions(metadata.permissions())?;
        }
        file.write_all(bytes)?;
        file.sync_all()?;
        std::fs::rename(&staged, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&staged);
    }
    result.map_err(|e| format!("replace {}: {e}", path.display()))
}

pub(super) fn remove_file_if_exists(path: &Path) -> Result<(), String> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("remove {}: {e}", path.display())),
    }
}

pub(super) fn with_rollback(error: String, rollback: Result<(), String>) -> String {
    match rollback {
        Ok(()) => error,
        Err(rollback) => format!("{error}; rollback failed: {rollback}"),
    }
}
