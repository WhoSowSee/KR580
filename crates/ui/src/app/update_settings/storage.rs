use super::super::messages::Message;
use super::super::state::DesktopApp;
use super::network::is_directory_writable;
use crate::i18n::Key;
use iced::Task;
use std::path::PathBuf;
use std::time::{Duration, Instant};

impl DesktopApp {
    pub(super) fn browse_settings_floppy_image(&mut self) -> Task<Message> {
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
        dialog
            .pick_file()
            .map(Message::SettingsDraftFloppyImageSet)
            .map(Task::done)
            .unwrap_or_else(Task::none)
    }

    pub(super) fn browse_settings_hdd_directory(&mut self) -> Task<Message> {
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
        let Some(folder) = dialog.pick_folder() else {
            return Task::none();
        };
        if !is_directory_writable(&folder) {
            self.error_notice = Some(self.lang.t(Key::ErrHddDirectoryNotWritable).to_owned());
            self.error_notice_dismiss_at = Some(Instant::now() + Duration::from_secs(8));
            return Task::none();
        }
        Task::done(Message::SettingsDraftHddDirectorySet(folder))
    }
}

fn home_path() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}
