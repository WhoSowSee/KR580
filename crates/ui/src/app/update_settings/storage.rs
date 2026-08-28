use super::super::messages::Message;
use super::super::state::DesktopApp;
use crate::runtime::file_dialog;
use iced::Task;
use std::path::PathBuf;

impl DesktopApp {
    pub(super) fn browse_settings_floppy_image(&self) -> Task<Message> {
        if self.settings_dialog.is_none() {
            return Task::none();
        }
        let preferred = self
            .settings_dialog
            .as_ref()
            .and_then(|d| d.draft_floppy_image_path.clone())
            .unwrap_or_else(home_path);
        let mut dialog =
            rfd::FileDialog::new().add_filter("KR580 floppy image", &["kpd", "img", "bin"]);
        if preferred.exists() && preferred.is_file() {
            if let Some(parent) = preferred.parent() {
                dialog = dialog.set_directory(parent);
            }
            if let Some(name) = preferred.file_name() {
                dialog = dialog.set_file_name(name.to_string_lossy().as_ref());
            }
        } else if preferred.exists() && preferred.is_dir() {
            dialog = dialog.set_directory(&preferred);
        } else if let Some(parent) = preferred.parent() {
            dialog = dialog.set_directory(parent);
        }
        file_dialog::run(
            self.dialog_parent(None),
            dialog,
            rfd::FileDialog::pick_file,
            Message::SettingsDraftFloppyImageSet,
        )
    }

    pub(super) fn browse_settings_hdd_directory(&self) -> Task<Message> {
        if self.settings_dialog.is_none() {
            return Task::none();
        }
        let preferred = self
            .settings_dialog
            .as_ref()
            .and_then(|d| d.draft_hdd_directory.clone())
            .unwrap_or_else(home_path);
        let mut dialog = rfd::FileDialog::new();
        if preferred.exists() && preferred.is_dir() {
            dialog = dialog.set_directory(&preferred);
        } else if let Some(parent) = preferred.parent() {
            dialog = dialog.set_directory(parent);
        }
        file_dialog::run(
            self.dialog_parent(None),
            dialog,
            rfd::FileDialog::pick_folder,
            Message::SettingsDraftHddDirectorySet,
        )
    }
}

fn home_path() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}
