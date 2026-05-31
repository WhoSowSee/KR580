use std::path::PathBuf;

use crate::app::{DesktopApp, StatusKind};
use crate::i18n::Key;
use k580_app::{AppCommand, Snapshot580Flavour};

use super::parse::parse_hex_u16;

impl DesktopApp {
    pub(crate) fn open_snapshot(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("KR580 file", &["580"])
            .pick_file()
        {
            self.load_snapshot_from_path(path);
        }
    }

    /// Both `.580` flavours share the extension, so we dispatch
    /// `LoadAnySnapshot` and route the resolved flavour into the
    /// matching "current path" slot.
    pub(crate) fn load_snapshot_from_path(&mut self, path: PathBuf) {
        self.clear_error_notice();
        self.clear_info_notice();
        let display = path.display().to_string();
        self.running = false;
        self.pending_snapshot_flavour = None;
        self.dispatch_sync(AppCommand::LoadAnySnapshot(path.clone()));
        if self.error_notice.is_some() {
            return;
        }
        match self.pending_snapshot_flavour.take() {
            Some(Snapshot580Flavour::Modern) => {
                self.current_snapshot_path = Some(path);
                self.current_legacy_snapshot_path = None;
                self.set_status(StatusKind::Opened {
                    display: display.clone(),
                    legacy: false,
                });
            }
            Some(Snapshot580Flavour::Legacy) => {
                self.current_snapshot_path = None;
                self.current_legacy_snapshot_path = Some(path);
                self.set_status(StatusKind::Opened {
                    display: display.clone(),
                    legacy: true,
                });
                self.raise_info_notice(self.lang.t(Key::LegacyOpenedNotice).to_owned());
            }
            None => {
                // Worker accepted the load but failed to publish a
                // flavour — fall back to v1 so Ctrl+S still works.
                self.current_snapshot_path = Some(path);
                self.current_legacy_snapshot_path = None;
                self.set_status(StatusKind::Opened {
                    display: display.clone(),
                    legacy: false,
                });
            }
        }
        self.undo_stack.clear();
        self.dirty = false;
        self.speed_tier = self.default_speed;
        let pc = self.snapshot.cpu.pc;
        self.set_memory_address(pc);
    }

    pub(crate) fn save_snapshot(&mut self) {
        self.commit_pending_inline_edit();
        self.clear_error_notice();
        let path = match &self.current_snapshot_path {
            Some(path) => path.clone(),
            None => {
                let Some(path) = rfd::FileDialog::new()
                    .add_filter("KR580 file", &["580"])
                    .save_file()
                else {
                    return;
                };
                self.current_snapshot_path = Some(path.clone());
                path
            }
        };
        let display = path.display().to_string();
        self.dispatch_sync(AppCommand::SaveSnapshot(path));
        if self.error_notice.is_some() {
            return;
        }
        self.current_legacy_snapshot_path = None;
        self.dirty = false;
        self.set_status(StatusKind::SavedTo {
            display,
            legacy: false,
        });
    }

    pub(crate) fn save_snapshot_as(&mut self) {
        self.commit_pending_inline_edit();
        let mut dialog = rfd::FileDialog::new().add_filter("KR580 file", &["580"]);
        if let Some(current) = &self.current_snapshot_path {
            if let Some(parent) = current.parent() {
                dialog = dialog.set_directory(parent);
            }
            if let Some(name) = current.file_name() {
                dialog = dialog.set_file_name(name.to_string_lossy().as_ref());
            }
        }
        let Some(path) = dialog.save_file() else {
            return;
        };
        self.clear_error_notice();
        let display = path.display().to_string();
        self.dispatch_sync(AppCommand::SaveSnapshot(path.clone()));
        if self.error_notice.is_some() {
            return;
        }
        self.current_snapshot_path = Some(path);
        self.current_legacy_snapshot_path = None;
        self.dirty = false;
        self.set_status(StatusKind::SavedTo {
            display,
            legacy: false,
        });
    }

    /// v1 and legacy share the `.580` extension but not the wire
    /// format, so the two paths must stay separate.
    pub(crate) fn save_legacy_snapshot(&mut self) {
        self.commit_pending_inline_edit();
        let (path, picked_now) = match &self.current_legacy_snapshot_path {
            Some(path) => (path.clone(), false),
            None => {
                let mut dialog = rfd::FileDialog::new().add_filter("KR580 legacy file", &["580"]);
                if let Some(current) = &self.current_snapshot_path
                    && let Some(parent) = current.parent()
                {
                    dialog = dialog.set_directory(parent);
                }
                let Some(path) = dialog.save_file() else {
                    return;
                };
                (path, true)
            }
        };
        self.clear_error_notice();
        let display = path.display().to_string();
        self.dispatch_sync(AppCommand::SaveLegacySnapshot(path.clone()));
        if self.error_notice.is_some() {
            return;
        }
        if picked_now {
            self.current_legacy_snapshot_path = Some(path);
        }
        self.dirty = false;
        self.set_status(StatusKind::SavedTo {
            display,
            legacy: true,
        });
    }

    pub(crate) fn open_legacy_snapshot(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("KR580 legacy file", &["580"])
            .pick_file()
        else {
            return;
        };
        self.clear_error_notice();
        let display = path.display().to_string();
        self.running = false;
        self.dispatch_sync(AppCommand::LoadLegacySnapshot(path.clone()));
        if self.error_notice.is_some() {
            return;
        }
        self.undo_stack.clear();
        self.current_snapshot_path = None;
        self.current_legacy_snapshot_path = Some(path);
        self.dirty = false;
        self.speed_tier = self.default_speed;
        let pc = self.snapshot.cpu.pc;
        self.set_memory_address(pc);
        self.set_status(StatusKind::Opened {
            display,
            legacy: true,
        });
    }

    /// Push any uncommitted inline byte to the worker before saving.
    fn commit_pending_inline_edit(&mut self) {
        let Ok(address) = parse_hex_u16(&self.memory_address_input) else {
            return;
        };
        let Ok(value) = u8::from_str_radix(self.memory_inline_value_input.trim(), 16) else {
            return;
        };
        if self.snapshot.cpu.memory.read(address) == value {
            return;
        }
        self.undo_stack.break_coalescing();
        self.dispatch_with_undo(AppCommand::SetMemory(address, value));
    }

    pub(crate) fn export_file(&mut self) {
        self.commit_pending_inline_edit();
        let Some(path) = rfd::FileDialog::new()
            .add_filter("KR580 text export", &["txt"])
            .add_filter("KR580 spreadsheet export", &["xlsx"])
            .save_file()
        else {
            return;
        };
        // Routing is by extension on disk; anything not `.xlsx` is `.txt`.
        let path = normalise_export_path(path);
        self.clear_error_notice();
        let display = path.display().to_string();
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase());
        match extension.as_deref() {
            Some("xlsx") => self.dispatch_sync(AppCommand::ExportXlsx(path)),
            _ => self.dispatch_sync(AppCommand::ExportTxt(path)),
        }
        if self.error_notice.is_some() {
            return;
        }
        self.set_status(StatusKind::ExportTo { display });
    }

    pub(crate) fn import_file(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("KR580 file", &["txt", "xlsx"])
            .add_filter("KR580 txt file", &["txt"])
            .add_filter("KR580 spreadsheet file", &["xlsx"])
            .pick_file()
        else {
            return;
        };
        self.clear_error_notice();
        self.running = false;
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase());
        match extension.as_deref() {
            Some("xlsx") => self.dispatch_sync(AppCommand::ImportXlsx(path)),
            _ => self.dispatch_sync(AppCommand::ImportTxt(path)),
        }
        if self.error_notice.is_some() {
            return;
        }
        self.undo_stack.clear();
        self.current_legacy_snapshot_path = None;
        self.dirty = false;
    }
}

/// Appends `.txt` rather than replacing the existing extension so the
/// user's typed name stays visible (`mysnap.foo` → `mysnap.foo.txt`).
pub(super) fn normalise_export_path(path: PathBuf) -> PathBuf {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase());
    match extension.as_deref() {
        Some("txt") | Some("xlsx") => path,
        _ => {
            let mut as_string = path.into_os_string();
            as_string.push(".txt");
            PathBuf::from(as_string)
        }
    }
}

impl DesktopApp {
    pub(crate) fn save_monitor_image(&mut self) {
        let bytes =
            match crate::view::monitor_image::render_monitor_png(&self.snapshot.devices.monitor) {
                Ok(b) => b,
                Err(err) => {
                    tracing::error!("save monitor image: render: {err}");
                    self.set_status_custom(self.lang.t(Key::MonitorImageSaveFailed).to_owned());
                    return;
                }
            };

        let mut dialog = rfd::FileDialog::new()
            .add_filter("PNG image", &["png"])
            .set_file_name("monitor.png");
        if let Some(current) = &self.current_snapshot_path
            && let Some(parent) = current.parent()
        {
            dialog = dialog.set_directory(parent);
        }

        let Some(path) = dialog.save_file() else {
            return;
        };

        let path = match path.extension().and_then(|s| s.to_str()) {
            Some(ext) if ext.eq_ignore_ascii_case("png") => path,
            _ => {
                let mut s = path.into_os_string();
                s.push(".png");
                PathBuf::from(s)
            }
        };

        if let Err(err) = std::fs::write(&path, &bytes) {
            tracing::error!("save monitor image to {}: {err}", path.display());
            self.set_status_custom(self.lang.t(Key::MonitorImageSaveFailed).to_owned());
            return;
        }

        self.set_status_custom(format!(
            "{}: {}",
            self.lang.t(Key::MonitorImageSaved),
            path.display()
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::normalise_export_path;
    use std::path::PathBuf;

    #[test]
    fn keeps_supported_extensions_intact() {
        for already_ok in ["a.txt", "a.xlsx", "a.TXT", "a.XLSX", "deep/path/file.txt"] {
            assert_eq!(
                normalise_export_path(PathBuf::from(already_ok)),
                PathBuf::from(already_ok),
            );
        }
    }

    #[test]
    fn appends_txt_to_unknown_extension() {
        assert_eq!(
            normalise_export_path(PathBuf::from("mysnap.foo")),
            PathBuf::from("mysnap.foo.txt"),
        );
        assert_eq!(
            normalise_export_path(PathBuf::from("dump.png")),
            PathBuf::from("dump.png.txt"),
        );
    }

    #[test]
    fn appends_txt_when_no_extension() {
        assert_eq!(
            normalise_export_path(PathBuf::from("plain")),
            PathBuf::from("plain.txt"),
        );
    }

    /// `Path::extension` reports `.bashrc` as having no extension, so
    /// the suffix gets appended (`.bashrc.txt`).
    #[test]
    fn dotfiles_get_txt_appended() {
        assert_eq!(
            normalise_export_path(PathBuf::from(".bashrc")),
            PathBuf::from(".bashrc.txt"),
        );
    }
}
