use crate::app::{DesktopApp, Message, StatusKind, ToolWindowKind};
use crate::backend::AppCommand;
use iced::Task;
use std::path::{Path, PathBuf};

use super::file_dialog;

mod images;
pub(crate) use images::{ImagePreview, ImageRead};

impl DesktopApp {
    pub(crate) fn open_floppy_image(&self) -> Task<Message> {
        let settings = &self.preferences.stored;
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

        file_dialog::run(
            self.dialog_parent(Some(ToolWindowKind::Floppy)),
            dialog,
            rfd::FileDialog::pick_file,
            Message::FloppyImagePathSelected,
        )
    }

    pub(crate) fn attach_floppy_image(&mut self, path: PathBuf) {
        self.clear_error_notice();
        self.dispatch_action(
            AppCommand::AttachFloppyImage(path.clone()),
            crate::app::BackendAction::FloppyAttached(path),
        );
    }

    pub(crate) fn finish_floppy_attachment(&mut self, path: PathBuf) {
        if self.snapshot.devices.floppy.path.as_ref() != Some(&path) {
            return;
        }
        let saved_path = path.clone();
        self.queue_settings_change(
            crate::runtime::file_work::SettingsAction::Quiet,
            move |settings| {
                settings.storage.floppy_path = saved_path;
            },
        );
        self.refresh_hdd_file_exists();
        self.set_status(StatusKind::FloppyImageAttached {
            display: path.display().to_string(),
        });
        if self.panels.floppy_show_image_contents {
            self.refresh_floppy_image_contents();
        }
    }

    pub(crate) fn save_floppy_buffer(&self) -> Task<Message> {
        let settings = &self.preferences.stored;
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

        file_dialog::run(
            self.dialog_parent(Some(ToolWindowKind::Floppy)),
            dialog,
            rfd::FileDialog::save_file,
            Message::FloppyBufferPathSelected,
        )
    }

    pub(crate) fn save_floppy_buffer_to_path(&mut self, path: PathBuf) {
        let bytes = self.snapshot.devices.floppy.visible_buffer.clone();
        self.queue_file_work(
            crate::runtime::file_work::FileRequest::FloppySaved,
            move || {
                let path = save_floppy_buffer_file(&path, &bytes)?;
                Ok(crate::runtime::file_work::FileResult::Saved(path))
            },
        );
    }
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
    crate::persistence::write_file_atomic(&path, bytes)?;
    Ok(path)
}

pub(crate) fn hdd_default_path(settings: &crate::persistence::Settings) -> PathBuf {
    let dir = settings.general.hdd_directory.clone().unwrap_or_else(|| {
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
    pub(crate) fn choose_hdd_directory(&self) -> Task<Message> {
        let mut dialog = rfd::FileDialog::new();

        let preferred = self
            .snapshot
            .devices
            .hdd
            .path
            .as_ref()
            .cloned()
            .unwrap_or_else(|| hdd_default_path(&self.preferences.stored));
        if let Some(parent) = preferred
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            dialog = dialog.set_directory(parent);
        }

        file_dialog::run(
            self.dialog_parent(Some(ToolWindowKind::Hdd)),
            dialog,
            rfd::FileDialog::pick_folder,
            Message::HddDirectorySelected,
        )
    }

    pub(crate) fn attach_hdd_directory(&mut self, folder: PathBuf) {
        self.panels.hdd_generation = self.panels.hdd_generation.wrapping_add(1);
        self.clear_error_notice();
        let hdd_path = folder.join("hdd.kpd");
        self.dispatch_action(
            AppCommand::AttachHddFile(hdd_path.clone()),
            crate::app::BackendAction::HddAttached(hdd_path),
        );
    }

    pub(crate) fn delete_hdd_file(&mut self) {
        let Some(path) = self.snapshot.devices.hdd.path.clone() else {
            return;
        };
        self.panels.hdd_generation = self.panels.hdd_generation.wrapping_add(1);
        let generation = self.panels.hdd_generation;
        self.queue_file_work(
            crate::runtime::file_work::FileRequest::HddDeleted {
                path: path.clone(),
                generation,
            },
            move || {
                match std::fs::remove_file(&path) {
                    Ok(()) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => return Err(error.into()),
                }
                Ok(crate::runtime::file_work::FileResult::Deleted)
            },
        );
    }

    pub(crate) fn create_hdd_file(&mut self) {
        self.panels.hdd_generation = self.panels.hdd_generation.wrapping_add(1);
        let path = self
            .snapshot
            .devices
            .hdd
            .path
            .clone()
            .unwrap_or_else(|| hdd_default_path(&self.preferences.stored));
        self.dispatch_action(
            AppCommand::AttachHddFile(path.clone()),
            crate::app::BackendAction::HddAttached(path),
        );
    }
}
