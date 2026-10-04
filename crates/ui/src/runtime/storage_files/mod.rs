use crate::app::{DesktopApp, Message, StatusKind, ToolWindowKind};
use crate::backend::AppCommand;
use crate::i18n::Key;
use crate::settings_storage::{load_settings, save_settings};
use iced::Task;
use std::path::{Path, PathBuf};

use super::file_dialog;

mod images;
pub(crate) use images::*;

impl DesktopApp {
    pub(crate) fn open_floppy_image(&self) -> Task<Message> {
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
        let mut settings = load_settings();
        settings.storage.floppy_path = path.clone();
        if let Err(error) = save_settings(&settings) {
            let notice = crate::runtime::humanize_error::humanize(&error.into(), self.lang);
            self.show_error_notice(notice);
        }
        self.refresh_hdd_file_exists();
        self.set_status(StatusKind::FloppyImageAttached {
            display: path.display().to_string(),
        });
        if self.floppy_show_image_contents {
            self.refresh_floppy_image_contents();
        }
    }

    pub(crate) fn save_floppy_buffer(&self) -> Task<Message> {
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

        file_dialog::run(
            self.dialog_parent(Some(ToolWindowKind::Floppy)),
            dialog,
            rfd::FileDialog::save_file,
            Message::FloppyBufferPathSelected,
        )
    }

    pub(crate) fn save_floppy_buffer_to_path(&mut self, path: PathBuf) {
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
    pub(crate) fn choose_hdd_directory(&self) -> Task<Message> {
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

        file_dialog::run(
            self.dialog_parent(Some(ToolWindowKind::Hdd)),
            dialog,
            rfd::FileDialog::pick_folder,
            Message::HddDirectorySelected,
        )
    }

    pub(crate) fn attach_hdd_directory(&mut self, folder: PathBuf) {
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
        if !path.exists() {
            self.hdd_file_exists = false;
            return;
        }
        if let Err(error) = std::fs::remove_file(&path) {
            tracing::error!("failed to delete HDD file {}: {error}", path.display());
            self.set_status_custom(self.lang.t(Key::ErrCannotWriteFile).to_owned());
            return;
        }
        self.dispatch_action(
            AppCommand::DetachHddFile,
            crate::app::BackendAction::HddDeleted(path),
        );
    }

    pub(crate) fn create_hdd_file(&mut self) {
        let path = self
            .snapshot
            .devices
            .hdd
            .path
            .clone()
            .unwrap_or_else(hdd_default_path);
        self.dispatch_action(
            AppCommand::AttachHddFile(path.clone()),
            crate::app::BackendAction::HddAttached(path),
        );
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
}
