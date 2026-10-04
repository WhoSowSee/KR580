use crate::app::{DesktopApp, Message};
use crate::i18n::Key;
use iced::Task;
use k580_ui::devices::StorageKind;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FileStamp {
    path: PathBuf,
    modified: Option<SystemTime>,
    length: u64,
}

impl DesktopApp {
    pub(crate) fn refresh_open_image_contents(&mut self) -> Task<Message> {
        let mut tasks = Vec::new();
        if self.panels.floppy_open
            && self.panels.floppy_show_image_contents
            && let Some(task) = self.schedule_image_read(StorageKind::Floppy)
        {
            tasks.push(task);
        }
        if self.panels.hdd_open
            && self.panels.hdd_show_image_contents
            && let Some(task) = self.schedule_image_read(StorageKind::Hdd)
        {
            tasks.push(task);
        }
        Task::batch(tasks)
    }

    fn schedule_image_read(&mut self, kind: StorageKind) -> Option<Task<Message>> {
        let (path, stamp) = if kind == StorageKind::Floppy {
            (
                self.snapshot.devices.floppy.path.clone(),
                &mut self.panels.floppy_image_file_stamp,
            )
        } else {
            (
                self.snapshot.devices.hdd.path.clone(),
                &mut self.panels.hdd_image_file_stamp,
            )
        };
        let path = path?;
        let current = match file_stamp(&path) {
            Ok(stamp) => stamp,
            Err(error) => {
                if kind == StorageKind::Floppy {
                    self.panels.floppy_image_contents.clear();
                    self.panels.floppy_image_error = Some(error.to_string());
                    self.panels.floppy_image_file_stamp = None;
                } else {
                    self.panels.hdd_image_contents.clear();
                    self.panels.hdd_image_error = Some(error.to_string());
                    self.panels.hdd_image_file_stamp = None;
                }
                return None;
            }
        };
        if stamp.as_ref() == Some(&current) {
            return None;
        }
        *stamp = Some(current);
        let message_path = path.clone();
        let message = move |result| {
            if kind == StorageKind::Floppy {
                Message::FloppyImageContentsLoaded(message_path.clone(), result)
            } else {
                Message::HddImageContentsLoaded(message_path.clone(), result)
            }
        };
        Some(Task::perform(read_file(path), message))
    }

    pub(crate) fn apply_floppy_image_contents(
        &mut self,
        path: PathBuf,
        result: Result<Vec<u8>, String>,
    ) {
        if self.snapshot.devices.floppy.path.as_ref() != Some(&path) {
            return;
        }
        match result {
            Ok(bytes) => {
                self.panels.floppy_image_contents = bytes;
                self.panels.floppy_image_error = None;
            }
            Err(error) => {
                self.panels.floppy_image_contents.clear();
                self.panels.floppy_image_error = Some(error);
                self.panels.floppy_image_file_stamp = None;
            }
        }
    }

    pub(crate) fn apply_hdd_image_contents(
        &mut self,
        path: PathBuf,
        result: Result<Vec<u8>, String>,
    ) {
        if self.snapshot.devices.hdd.path.as_ref() != Some(&path) {
            return;
        }
        match result {
            Ok(bytes) => {
                self.panels.hdd_image_contents = bytes;
                self.panels.hdd_image_error = None;
            }
            Err(error) => {
                self.panels.hdd_image_contents.clear();
                self.panels.hdd_image_error = Some(error);
                self.panels.hdd_image_file_stamp = None;
            }
        }
    }

    pub(crate) fn refresh_floppy_image_contents(&mut self) {
        let Some(path) = self.snapshot.devices.floppy.path.as_ref() else {
            self.panels.floppy_image_contents.clear();
            self.panels.floppy_image_file_stamp = None;
            self.panels.floppy_image_error =
                Some(self.preferences.lang.t(Key::FloppyPathMissing).into());
            return;
        };

        match read_file_if_changed(path, &mut self.panels.floppy_image_file_stamp) {
            Ok(Some(bytes)) => {
                self.panels.floppy_image_contents = bytes;
                self.panels.floppy_image_error = None;
            }
            Ok(None) => {}
            Err(error) => {
                self.panels.floppy_image_contents.clear();
                self.panels.floppy_image_file_stamp = None;
                self.panels.floppy_image_error = Some(format!(
                    "{}: {error}",
                    self.preferences.lang.t(Key::ErrCannotReadFile)
                ));
            }
        }
    }
    pub(crate) fn refresh_hdd_image_contents(&mut self) {
        let Some(path) = self.snapshot.devices.hdd.path.as_ref() else {
            self.panels.hdd_image_contents.clear();
            self.panels.hdd_image_file_stamp = None;
            self.panels.hdd_image_error = Some(self.preferences.lang.t(Key::HddPathMissing).into());
            return;
        };

        match read_file_if_changed(path, &mut self.panels.hdd_image_file_stamp) {
            Ok(Some(bytes)) => {
                self.panels.hdd_image_contents = bytes;
                self.panels.hdd_image_error = None;
            }
            Ok(None) => {}
            Err(error) => {
                self.panels.hdd_image_contents.clear();
                self.panels.hdd_image_file_stamp = None;
                self.panels.hdd_image_error = Some(format!(
                    "{}: {error}",
                    self.preferences.lang.t(Key::ErrCannotReadFile)
                ));
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

async fn read_file(path: PathBuf) -> Result<Vec<u8>, String> {
    tokio::task::spawn_blocking(move || std::fs::read(path))
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())
}

fn file_stamp(path: &Path) -> std::io::Result<FileStamp> {
    let metadata = std::fs::metadata(path)?;
    Ok(FileStamp {
        path: path.to_path_buf(),
        modified: metadata.modified().ok(),
        length: metadata.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::super::save_floppy_buffer_file;
    use super::read_file_if_changed;
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
}
