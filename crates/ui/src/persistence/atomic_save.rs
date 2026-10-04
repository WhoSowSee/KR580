use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_FILE: AtomicU64 = AtomicU64::new(0);

/// Atomically replaces a file while preserving its permissions and symlink target.
pub fn write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    replace_with(path, |file| file.write_all(bytes))
}

fn replace_with(path: &Path, write: impl FnOnce(&mut File) -> io::Result<()>) -> io::Result<()> {
    let target = match fs::canonicalize(path) {
        Ok(target) => target,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            if fs::symlink_metadata(path).is_ok_and(|metadata| metadata.is_symlink()) {
                return Err(error);
            }
            path.to_path_buf()
        }
        Err(error) => return Err(error),
    };
    let permissions = match fs::metadata(&target) {
        Ok(metadata) => {
            if !metadata.is_file() {
                return Err(io::Error::other("save destination is not a file"));
            }
            if metadata.permissions().readonly() {
                return Err(io::Error::from(io::ErrorKind::PermissionDenied));
            }
            Some(metadata.permissions())
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(error) => return Err(error),
    };
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let (staged, mut file) = loop {
        let sequence = NEXT_FILE.fetch_add(1, Ordering::Relaxed);
        let staged = parent.join(format!(".kr580-{}-{sequence}.tmp", std::process::id()));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&staged)
        {
            Ok(file) => break (staged, file),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    };
    let result = (|| {
        write(&mut file)?;
        if let Some(permissions) = permissions {
            file.set_permissions(permissions)?;
        }
        file.sync_all()
    })();
    drop(file);
    let result = result.and_then(|()| fs::rename(&staged, &target));
    if result.is_err() {
        let _ = fs::remove_file(&staged);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_write_failure_preserves_existing_file_and_cleans_temporary_file() {
        let dir = std::env::temp_dir().join(format!("kr580-save-failure-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("program.krs");
        fs::write(&path, b"original").unwrap();
        let result = replace_with(&path, |file| {
            file.write_all(b"partial")?;
            Err(io::Error::other("simulated write failure"))
        });
        assert!(result.is_err());
        assert_eq!(fs::read(&path).unwrap(), b"original");
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
        write(&path, b"new").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"new");
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
        fs::remove_file(path).unwrap();
        fs::remove_dir(dir).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn replacement_failure_preserves_locked_destination() {
        use std::os::windows::fs::OpenOptionsExt;

        let dir = std::env::temp_dir().join(format!("kr580-save-locked-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("program.krs");
        fs::write(&path, b"original").unwrap();
        let lock = OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(&path)
            .unwrap();
        assert!(write(&path, b"new").is_err());
        assert_eq!(fs::read(&path).unwrap(), b"original");
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
        drop(lock);
        fs::remove_file(path).unwrap();
        fs::remove_dir(dir).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn dangling_symlink_is_not_replaced() {
        let dir = std::env::temp_dir().join(format!("kr580-save-symlink-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("program.krs");
        std::os::unix::fs::symlink("missing.krs", &path).unwrap();
        assert!(write(&path, b"new").is_err());
        assert_eq!(fs::read_link(&path).unwrap(), Path::new("missing.krs"));
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
        fs::remove_file(path).unwrap();
        fs::remove_dir(dir).unwrap();
    }
}
