use std::collections::hash_map::DefaultHasher;
use std::fs::{self, OpenOptions};
use std::hash::Hasher;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_BACKUP: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, PartialEq, Eq)]
struct Fingerprint {
    length: u64,
    hash: u64,
    modified: Option<std::time::SystemTime>,
    permissions: u32,
}

struct SavedFile {
    requested: PathBuf,
    path: PathBuf,
    backup: Option<PathBuf>,
    original: Option<Fingerprint>,
    expected: Option<Fingerprint>,
    observation_failed: bool,
}

#[derive(Default)]
pub(super) struct FileJournal {
    files: Vec<SavedFile>,
    new_directories: Vec<PathBuf>,
    finished: bool,
}

impl FileJournal {
    pub(super) fn prepare(&mut self, paths: &[PathBuf]) -> Result<(), String> {
        for path in paths {
            let path = absolute(path)?;
            if self.files.iter().any(|saved| saved.requested == path) {
                continue;
            }
            self.track_parents(&path)?;
            let target = resolve(&path)?;
            let original = fingerprint(&target)?;
            let backup = if original.is_some() {
                let backup = sibling(&target, "backup")?;
                if let Err(error) = fs::copy(&target, &backup) {
                    let _ = fs::remove_file(&backup);
                    return Err(format!("backup {}: {error}", target.display()));
                }
                let current = fingerprint(&target);
                if current.as_ref().ok() != Some(&original) {
                    let _ = fs::remove_file(&backup);
                    return Err(current.err().unwrap_or_else(|| {
                        format!("file changed during backup: {}", target.display())
                    }));
                }
                Some(backup)
            } else {
                None
            };
            self.files.push(SavedFile {
                requested: path,
                path: target,
                backup,
                expected: original.clone(),
                original,
                observation_failed: false,
            });
        }
        Ok(())
    }

    pub(super) fn create_directory(&mut self, path: &Path) -> Result<(), String> {
        self.track_parents(&path.join("placeholder"))?;
        fs::create_dir_all(path).map_err(|error| format!("create {}: {error}", path.display()))
    }

    pub(super) fn change<T>(
        &mut self,
        paths: &[PathBuf],
        operation: impl FnOnce() -> Result<T, String>,
    ) -> Result<T, String> {
        self.prepare(paths)?;
        let requested = paths
            .iter()
            .map(|path| absolute(path))
            .collect::<Result<Vec<_>, _>>()?;
        for saved in self
            .files
            .iter()
            .filter(|saved| requested.contains(&saved.requested))
        {
            if !same_path(&resolve(&saved.requested)?, &saved.path)
                || fingerprint(&saved.path)? != saved.expected
            {
                return Err(format!(
                    "file changed before installation step: {}",
                    saved.requested.display()
                ));
            }
        }
        let result = operation();
        let mut observation_errors = Vec::new();
        for saved in self
            .files
            .iter_mut()
            .filter(|saved| requested.contains(&saved.requested))
        {
            match fingerprint(&saved.path) {
                Ok(current) => saved.expected = current,
                Err(error) => {
                    saved.observation_failed = true;
                    observation_errors.push(error);
                }
            }
        }
        if observation_errors.is_empty() {
            result
        } else {
            Err(format!(
                "{}; record file changes: {}",
                result
                    .err()
                    .unwrap_or_else(|| "installation step completed".into()),
                observation_errors.join("; ")
            ))
        }
    }

    pub(super) fn rollback(&mut self) -> Result<(), String> {
        let mut errors = Vec::new();
        for saved in self.files.iter_mut().rev() {
            if saved.original == saved.expected && !saved.observation_failed {
                continue;
            }
            let restore = (|| {
                if saved.observation_failed
                    || !same_path(&resolve(&saved.requested)?, &saved.path)
                    || fingerprint(&saved.path)? != saved.expected
                {
                    return Err(format!("rollback conflict: {}", saved.requested.display()));
                }
                match &saved.backup {
                    Some(backup) => fs::rename(backup, &saved.path).map_err(|error| {
                        format!(
                            "restore {} from {}: {error}",
                            saved.path.display(),
                            backup.display()
                        )
                    })?,
                    None => match fs::remove_file(&saved.path) {
                        Ok(()) => {}
                        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                        Err(error) => {
                            return Err(format!(
                                "remove new file {}: {error}",
                                saved.path.display()
                            ));
                        }
                    },
                }
                saved.expected = saved.original.clone();
                Ok(())
            })();
            if let Err(error) = restore {
                errors.push(error);
            }
        }
        self.finished = true;
        if errors.is_empty() {
            self.clean_backups();
            self.clean_directories();
            Ok(())
        } else {
            let backups = self
                .files
                .iter()
                .filter_map(|saved| saved.backup.as_ref())
                .filter(|path| path.exists())
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>();
            Err(format!(
                "{}; retained backups: {}",
                errors.join("; "),
                backups.join(", ")
            ))
        }
    }

    pub(super) fn commit(&mut self) {
        self.finished = true;
        self.clean_backups();
    }

    fn track_parents(&mut self, path: &Path) -> Result<(), String> {
        for parent in absolute(path)?.ancestors().skip(1) {
            match fs::metadata(parent) {
                Ok(metadata) if metadata.is_dir() => break,
                Ok(_) => return Err(format!("parent is not a directory: {}", parent.display())),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    if !self.new_directories.iter().any(|path| path == parent) {
                        self.new_directories.push(parent.to_path_buf());
                    }
                }
                Err(error) => return Err(format!("inspect {}: {error}", parent.display())),
            }
        }
        Ok(())
    }

    fn clean_backups(&self) {
        for backup in self.files.iter().filter_map(|saved| saved.backup.as_ref()) {
            if let Err(error) = fs::remove_file(backup)
                && error.kind() != std::io::ErrorKind::NotFound
            {
                tracing::warn!(path = %backup.display(), %error, "installer backup retained");
            }
        }
    }

    pub(super) fn clean_directories(&mut self) {
        self.new_directories
            .sort_by_key(|path| std::cmp::Reverse(path.components().count()));
        for path in &self.new_directories {
            if let Err(error) = fs::remove_dir(path)
                && !matches!(
                    error.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::DirectoryNotEmpty
                )
            {
                tracing::warn!(path = %path.display(), %error, "installer directory retained");
            }
        }
    }
}

impl Drop for FileJournal {
    fn drop(&mut self) {
        if !self.finished
            && let Err(error) = self.rollback()
        {
            tracing::error!(%error, "interrupted installer rollback failed");
        }
    }
}

pub(super) fn absolute(path: &Path) -> Result<PathBuf, String> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        std::env::current_dir()
            .map(|root| root.join(path))
            .map_err(|error| format!("resolve install path: {error}"))
    }
}

pub(super) fn sibling(path: &Path, purpose: &str) -> Result<PathBuf, String> {
    let parent = path
        .parent()
        .ok_or_else(|| "file has no parent".to_owned())?;
    loop {
        let candidate = parent.join(format!(
            ".kr580-{purpose}-{}-{}",
            std::process::id(),
            NEXT_BACKUP.fetch_add(1, Ordering::Relaxed)
        ));
        match OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&candidate)
        {
            Ok(_) => return Ok(candidate),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(format!("create {}: {error}", candidate.display())),
        }
    }
}

fn resolve(path: &Path) -> Result<PathBuf, String> {
    match fs::canonicalize(path) {
        Ok(path) => Ok(path),
        Err(error)
            if error.kind() == std::io::ErrorKind::NotFound
                && fs::symlink_metadata(path).is_err() =>
        {
            let parent = path
                .parent()
                .ok_or_else(|| format!("path has no existing root: {}", path.display()))?;
            let name = path
                .file_name()
                .ok_or_else(|| "path has no name".to_owned())?;
            Ok(resolve(parent)?.join(name))
        }
        Err(error) => Err(format!("resolve {}: {error}", path.display())),
    }
}

fn same_path(left: &Path, right: &Path) -> bool {
    #[cfg(windows)]
    {
        left.as_os_str()
            .to_string_lossy()
            .eq_ignore_ascii_case(&right.as_os_str().to_string_lossy())
    }
    #[cfg(target_os = "macos")]
    {
        use std::os::unix::fs::MetadataExt;
        left == right
            || (left
                .as_os_str()
                .to_string_lossy()
                .eq_ignore_ascii_case(&right.as_os_str().to_string_lossy())
                && std::fs::metadata(left)
                    .ok()
                    .zip(std::fs::metadata(right).ok())
                    .is_some_and(|(left, right)| {
                        left.dev() == right.dev() && left.ino() == right.ino()
                    }))
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        left == right
    }
}

fn fingerprint(path: &Path) -> Result<Option<Fingerprint>, String> {
    match fs::metadata(path) {
        Ok(metadata) if !metadata.is_file() => {
            return Err(format!("not a file: {}", path.display()));
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("metadata {}: {error}", path.display())),
    }
    let mut file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("read {}: {error}", path.display())),
    };
    let metadata = file
        .metadata()
        .map_err(|error| format!("metadata {}: {error}", path.display()))?;
    if !metadata.is_file() {
        return Err(format!("not a file: {}", path.display()));
    }
    let mut hasher = DefaultHasher::new();
    let mut buffer = [0u8; 65_536];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|error| format!("read {}: {error}", path.display()))?;
        if count == 0 {
            break;
        }
        hasher.write(&buffer[..count]);
    }
    #[cfg(unix)]
    let permissions = {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode()
    };
    #[cfg(windows)]
    let permissions = u32::from(metadata.permissions().readonly());
    Ok(Some(Fingerprint {
        length: metadata.len(),
        hash: hasher.finish(),
        modified: metadata.modified().ok(),
        permissions,
    }))
}
