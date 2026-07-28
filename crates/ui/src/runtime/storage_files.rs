use crate::app::{DesktopApp, StatusKind};
use crate::backend::AppCommand;
use crate::i18n::Key;
use crate::settings_storage::{load_settings, save_settings};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FileStamp {
    path: PathBuf,
    modified: Option<SystemTime>,
    length: u64,
}

impl DesktopApp {
    pub(crate) fn refresh_open_image_contents(&mut self) {
        if self.floppy_open && self.floppy_show_image_contents {
            self.refresh_floppy_image_contents();
        }
        if self.hdd_open && self.hdd_show_image_contents {
            self.refresh_hdd_image_contents();
        }
    }

    pub(crate) fn open_floppy_image(&mut self) {
        let settings = load_settings();
        let mut dialog =
            rfd::FileDialog::new().add_filter("KR580 floppy image", &["kpd", "img", "bin"]);

        let preferred = self
            .snapshot
            .devices
            .floppy
            .path
            .as_ref()
            .or(settings.general.floppy_image_path.as_ref())
            .unwrap_or(&settings.storage.floppy_path);
        if let Some(parent) = preferred
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            dialog = dialog.set_directory(parent);
        }
        if let Some(name) = preferred.file_name() {
            dialog = dialog.set_file_name(name.to_string_lossy().as_ref());
        }

        let Some(path) = dialog.pick_file() else {
            return;
        };

        self.clear_error_notice();
        self.dispatch_sync(AppCommand::AttachFloppyImage(path.clone()));
        if self.error_notice.is_some() {
            return;
        }

        let mut settings = settings;
        settings.storage.floppy_path = path.clone();
        save_settings(&settings);
        self.refresh_hdd_file_exists();
        self.set_status(StatusKind::FloppyImageAttached {
            display: path.display().to_string(),
        });
        if self.floppy_show_image_contents {
            self.refresh_floppy_image_contents();
        }
    }

    pub(crate) fn save_floppy_buffer(&mut self) {
        let settings = load_settings();
        let mut dialog = rfd::FileDialog::new().set_file_name("floppy_buffer.kpd");
        for (name, extensions) in floppy_buffer_save_filters() {
            dialog = dialog.add_filter(name, extensions);
        }

        let preferred = self
            .snapshot
            .devices
            .floppy
            .path
            .as_ref()
            .unwrap_or(&settings.storage.floppy_path);
        if let Some(parent) = preferred
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            dialog = dialog.set_directory(parent);
        }

        let Some(path) = dialog.save_file() else {
            return;
        };

        match save_floppy_buffer_file(&path, &self.snapshot.devices.floppy.visible_buffer) {
            Ok(path) => self.set_status_custom(format!(
                "{}: {}",
                self.lang.t(Key::FloppyBufferSaved),
                path.display()
            )),
            Err(error) => {
                tracing::error!("save floppy buffer to {}: {error}", path.display());
                self.set_status_custom(self.lang.t(Key::ErrCannotWriteFile).to_owned());
            }
        }
    }

    pub(crate) fn refresh_floppy_image_contents(&mut self) {
        let Some(path) = self.snapshot.devices.floppy.path.as_ref() else {
            self.floppy_image_contents.clear();
            self.floppy_image_file_stamp = None;
            self.floppy_image_error = Some(self.lang.t(Key::FloppyPathMissing).into());
            return;
        };

        match read_file_if_changed(path, &mut self.floppy_image_file_stamp) {
            Ok(Some(bytes)) => {
                self.floppy_image_contents = bytes;
                self.floppy_image_error = None;
            }
            Ok(None) => {}
            Err(error) => {
                self.floppy_image_contents.clear();
                self.floppy_image_file_stamp = None;
                self.floppy_image_error =
                    Some(format!("{}: {error}", self.lang.t(Key::ErrCannotReadFile)));
            }
        }
    }
}

fn read_file_if_changed(
    path: &Path,
    known_stamp: &mut Option<FileStamp>,
) -> std::io::Result<Option<Vec<u8>>> {
    let metadata = std::fs::metadata(path)?;
    let current_stamp = FileStamp {
        path: path.to_path_buf(),
        modified: metadata.modified().ok(),
        length: metadata.len(),
    };
    if known_stamp.as_ref() == Some(&current_stamp) {
        return Ok(None);
    }

    let bytes = std::fs::read(path)?;
    *known_stamp = Some(current_stamp);
    Ok(Some(bytes))
}

fn floppy_buffer_save_filters() -> [(&'static str, &'static [&'static str]); 3] {
    [
        ("KR580 floppy buffer (*.kpd)", &["kpd"]),
        ("KR580 floppy image (*.img)", &["img"]),
        ("Raw binary buffer (*.bin)", &["bin"]),
    ]
}

fn save_floppy_buffer_file(path: &Path, bytes: &[u8]) -> std::io::Result<PathBuf> {
    let path = floppy_buffer_save_path(path);
    std::fs::write(&path, bytes)?;
    Ok(path)
}

pub(crate) fn hdd_default_path() -> PathBuf {
    let settings = load_settings();
    let dir = settings.general.hdd_directory.unwrap_or_else(|| {
        std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
    });
    dir.join("hdd.kpd")
}

fn floppy_buffer_save_path(path: &Path) -> PathBuf {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("kpd" | "img" | "bin") => path.to_path_buf(),
        _ => {
            let mut raw = path.as_os_str().to_os_string();
            raw.push(".kpd");
            PathBuf::from(raw)
        }
    }
}
impl DesktopApp {
    pub(crate) fn choose_hdd_directory(&mut self) {
        let mut dialog = rfd::FileDialog::new();

        let preferred = self
            .snapshot
            .devices
            .hdd
            .path
            .as_ref()
            .cloned()
            .unwrap_or_else(hdd_default_path);
        if let Some(parent) = preferred
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            dialog = dialog.set_directory(parent);
        }

        let Some(folder) = dialog.pick_folder() else {
            return;
        };

        self.clear_error_notice();
        let hdd_path = folder.join("hdd.kpd");
        self.hdd_file_exists = true;
        self.dispatch_sync(crate::backend::AppCommand::AttachHddFile(hdd_path.clone()));
        if self.error_notice.is_some() {
            self.hdd_file_exists = false;
            return;
        }

        self.set_status(StatusKind::HddImageAttached {
            display: hdd_path.display().to_string(),
        });
    }
    pub(crate) fn delete_hdd_file(&mut self) {
        let Some(path) = self.snapshot.devices.hdd.path.clone() else {
            return;
        };
        if !path.exists() {
            self.hdd_file_exists = false;
            return;
        }
        if let Err(error) = std::fs::remove_file(&path) {
            tracing::error!("failed to delete HDD file {}: {error}", path.display());
            self.set_status_custom(self.lang.t(Key::ErrCannotWriteFile).to_owned());
            return;
        }
        self.hdd_file_exists = false;
        self.dispatch_sync(crate::backend::AppCommand::DetachHddFile);
        self.set_status(StatusKind::HddFileDeleted {
            display: path.display().to_string(),
        });
    }

    pub(crate) fn create_hdd_file(&mut self) {
        let path = self
            .snapshot
            .devices
            .hdd
            .path
            .clone()
            .unwrap_or_else(hdd_default_path);
        self.dispatch_sync(crate::backend::AppCommand::AttachHddFile(path.clone()));
        if self.error_notice.is_some() {
            return;
        }
        self.hdd_file_exists = true;
        self.set_status(StatusKind::HddImageAttached {
            display: path.display().to_string(),
        });
    }

    pub(crate) fn refresh_hdd_file_exists(&mut self) {
        self.hdd_file_exists = self
            .snapshot
            .devices
            .hdd
            .path
            .as_ref()
            .is_some_and(|p| p.exists());
    }

    pub(crate) fn refresh_hdd_image_contents(&mut self) {
        let Some(path) = self.snapshot.devices.hdd.path.as_ref() else {
            self.hdd_image_contents.clear();
            self.hdd_image_file_stamp = None;
            self.hdd_image_error = Some(self.lang.t(Key::HddPathMissing).into());
            return;
        };

        match read_file_if_changed(path, &mut self.hdd_image_file_stamp) {
            Ok(Some(bytes)) => {
                self.hdd_image_contents = bytes;
                self.hdd_image_error = None;
            }
            Ok(None) => {}
            Err(error) => {
                self.hdd_image_contents.clear();
                self.hdd_image_file_stamp = None;
                self.hdd_image_error =
                    Some(format!("{}: {error}", self.lang.t(Key::ErrCannotReadFile)));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::floppy_buffer_save_filters;
    use super::read_file_if_changed;
    use super::save_floppy_buffer_file;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn read_file_if_changed_skips_unchanged_image_and_reads_new_bytes() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("kr580-floppy-refresh-{stamp}.kpd"));
        let mut known_stamp = None;
        fs::write(&path, b"before").unwrap();

        assert_eq!(
            read_file_if_changed(&path, &mut known_stamp).unwrap(),
            Some(b"before".to_vec())
        );
        assert_eq!(read_file_if_changed(&path, &mut known_stamp).unwrap(), None);

        fs::write(&path, b"after image").unwrap();

        assert_eq!(
            read_file_if_changed(&path, &mut known_stamp).unwrap(),
            Some(b"after image".to_vec())
        );
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn save_floppy_buffer_file_writes_bytes_and_defaults_to_kpd() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let base = std::env::temp_dir().join(format!("kr580-floppy-buffer-{stamp}"));

        let path = save_floppy_buffer_file(&base, &[b'A', 0x80]).unwrap();

        let bytes = fs::read(&path).unwrap();
        fs::remove_file(&path).unwrap();
        assert_eq!(path.extension().and_then(|ext| ext.to_str()), Some("kpd"));
        assert_eq!(bytes, [b'A', 0x80]);
    }

    #[test]
    fn floppy_buffer_save_filters_are_separate_and_kpd_first() {
        let filters = floppy_buffer_save_filters();

        assert_eq!(
            filters,
            [
                ("KR580 floppy buffer (*.kpd)", &["kpd"][..]),
                ("KR580 floppy image (*.img)", &["img"][..]),
                ("Raw binary buffer (*.bin)", &["bin"][..]),
            ]
        );
    }
}
