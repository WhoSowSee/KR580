use crate::app::{DesktopApp, Message};
use crate::backend::AppError;
use crate::backend::error::AppErrorKind;
use crate::i18n::Key;
use crate::runtime::file_work::{FileRequest, FileResult};
use iced::Task;
use k580_ui::devices::StorageKind;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

const PREVIEW_LIMIT: u64 = 65_536;

#[derive(Clone, Debug, PartialEq, Eq)]
struct FileStamp {
    path: PathBuf,
    modified: Option<SystemTime>,
    length: u64,
}

pub(crate) struct ImageRead {
    stamp: FileStamp,
    bytes: Option<Vec<u8>>,
}

#[derive(Default)]
pub(crate) struct ImagePreview {
    pub(crate) contents: Vec<u8>,
    pub(crate) error: Option<String>,
    stamp: Option<FileStamp>,
    generation: u64,
    pending: Option<(u64, PathBuf)>,
}

impl ImagePreview {
    fn invalidate(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.pending = None;
        self.stamp = None;
    }
}

impl DesktopApp {
    pub(crate) fn refresh_open_image_contents(&mut self) -> Task<Message> {
        if self.panels.floppy_open && self.panels.floppy_show_image_contents {
            self.schedule_image_read(StorageKind::Floppy);
        }
        if self.panels.hdd_open {
            self.schedule_image_read(StorageKind::Hdd);
        }
        Task::none()
    }

    fn schedule_image_read(&mut self, kind: StorageKind) {
        let path = self.image_path(kind).cloned();
        let read_contents = match kind {
            StorageKind::Floppy => self.panels.floppy_show_image_contents,
            StorageKind::Hdd => self.panels.hdd_show_image_contents,
        };
        let Some(path) = path else {
            let key = match kind {
                StorageKind::Floppy => Key::FloppyPathMissing,
                StorageKind::Hdd => Key::HddPathMissing,
            };
            let error = self.preferences.lang.t(key).to_owned();
            let preview = self.image_preview(kind);
            preview.invalidate();
            preview.contents.clear();
            preview.error = Some(error);
            if kind == StorageKind::Hdd {
                self.panels.hdd_file_exists = false;
            }
            return;
        };
        let preview = self.image_preview(kind);
        if let Some((_, pending_path)) = &preview.pending {
            if pending_path == &path {
                return;
            }
            preview.invalidate();
        }
        preview.generation = preview.generation.wrapping_add(1);
        let generation = preview.generation;
        preview.pending = Some((generation, path.clone()));
        let stamp = preview.stamp.clone();
        self.queue_file_work(
            FileRequest::Image {
                kind,
                generation,
                path: path.clone(),
            },
            move || {
                read_image(&path, stamp.as_ref(), read_contents)
                    .map(FileResult::Image)
                    .map_err(Into::into)
            },
        );
    }

    pub(crate) fn apply_image_contents(
        &mut self,
        kind: StorageKind,
        generation: u64,
        path: PathBuf,
        result: Result<ImageRead, AppError>,
    ) {
        if self.image_path(kind) != Some(&path) {
            return;
        }
        let preview = self.image_preview(kind);
        if preview.pending.as_ref() != Some(&(generation, path)) {
            return;
        }
        preview.pending = None;
        if result
            .as_ref()
            .is_err_and(|error| error.kind() == AppErrorKind::DeviceBusy)
        {
            return;
        }
        let exists = result.is_ok();
        match result {
            Ok(image) => {
                if let Some(bytes) = image.bytes {
                    preview.contents = bytes;
                    preview.stamp = Some(image.stamp);
                }
                preview.error = None;
            }
            Err(error) => {
                preview.contents.clear();
                preview.stamp = None;
                preview.error = Some(error.to_string());
            }
        }
        if kind == StorageKind::Hdd {
            self.panels.hdd_file_exists = exists;
        }
    }

    pub(crate) fn invalidate_image_preview(&mut self, kind: StorageKind) {
        self.image_preview(kind).invalidate();
    }

    pub(crate) fn refresh_floppy_image_contents(&mut self) {
        self.invalidate_image_preview(StorageKind::Floppy);
        self.schedule_image_read(StorageKind::Floppy);
    }
    pub(crate) fn refresh_hdd_image_contents(&mut self) {
        self.invalidate_image_preview(StorageKind::Hdd);
        self.schedule_image_read(StorageKind::Hdd);
    }
    pub(crate) fn refresh_hdd_file_exists(&mut self) {
        self.schedule_image_read(StorageKind::Hdd);
    }

    fn image_preview(&mut self, kind: StorageKind) -> &mut ImagePreview {
        match kind {
            StorageKind::Floppy => &mut self.panels.floppy_image,
            StorageKind::Hdd => &mut self.panels.hdd_image,
        }
    }
    fn image_path(&self, kind: StorageKind) -> Option<&PathBuf> {
        match kind {
            StorageKind::Floppy => self.snapshot.devices.floppy.path.as_ref(),
            StorageKind::Hdd => self.snapshot.devices.hdd.path.as_ref(),
        }
    }
}

fn read_image(
    path: &Path,
    known: Option<&FileStamp>,
    read_contents: bool,
) -> std::io::Result<ImageRead> {
    let file = std::fs::File::open(path)?;
    let metadata = file.metadata()?;
    let stamp = FileStamp {
        path: path.to_path_buf(),
        modified: metadata.modified().ok(),
        length: metadata.len(),
    };
    let bytes = if read_contents && known != Some(&stamp) {
        let mut bytes = Vec::with_capacity(metadata.len().min(PREVIEW_LIMIT) as usize);
        file.take(PREVIEW_LIMIT).read_to_end(&mut bytes)?;
        Some(bytes)
    } else {
        None
    };
    Ok(ImageRead { stamp, bytes })
}

#[cfg(test)]
mod tests;
