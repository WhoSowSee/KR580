use std::path::PathBuf;

use crate::app::{DesktopApp, ExportTab, StatusKind};
use crate::i18n::Key;
use crate::view::monitor_image::MonitorImageFormat;
use k580_app::AppCommand;

use super::parse::parse_hex_u16;

impl DesktopApp {
    pub(crate) fn open_program(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("KR580 file", &["580"])
            .pick_file()
        {
            self.load_program_from_path(path);
        }
    }

    pub(crate) fn load_program_from_path(&mut self, path: PathBuf) {
        self.clear_error_notice();
        let display = path.display().to_string();
        self.running = false;
        self.dispatch_sync(AppCommand::LoadProgram(path.clone()));
        if self.error_notice.is_some() {
            return;
        }
        self.current_snapshot_path = Some(path);
        self.undo_stack.clear();
        self.mark_saved();
        self.speed_tier = self.default_speed;
        let pc = self.snapshot.cpu.pc;
        self.set_memory_address(pc);
        self.set_status(StatusKind::Opened { display });
    }

    pub(crate) fn save_program(&mut self) {
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
        self.dispatch_sync(AppCommand::SaveProgram(path));
        if self.error_notice.is_some() {
            return;
        }
        self.mark_saved();
        self.set_status(StatusKind::SavedTo { display });
    }

    pub(crate) fn save_program_as(&mut self) {
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
        self.dispatch_sync(AppCommand::SaveProgram(path.clone()));
        if self.error_notice.is_some() {
            return;
        }
        self.current_snapshot_path = Some(path);
        self.mark_saved();
        self.set_status(StatusKind::SavedTo { display });
    }

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

    pub(crate) fn export_selected_file(
        &mut self,
        format: ExportTab,
        options: k580_persistence::ExportOptions,
    ) {
        self.commit_pending_inline_edit();
        let filter = match format {
            ExportTab::Xlsx => "KR580 spreadsheet export",
            ExportTab::Text => "KR580 text export",
        };
        let Some(path) = rfd::FileDialog::new()
            .add_filter(filter, &[format.extension()])
            .set_file_name(format.default_file_name())
            .save_file()
        else {
            return;
        };
        let path = normalise_export_path_for_format(path, format);
        self.clear_error_notice();
        let display = path.display().to_string();
        match format {
            ExportTab::Xlsx => self.dispatch_sync(AppCommand::ExportXlsxWithOptions(path, options)),
            ExportTab::Text => self.dispatch_sync(AppCommand::ExportTxtWithOptions(path, options)),
        }
        if self.error_notice.is_some() {
            return;
        }
        self.set_status(StatusKind::ExportTo { display });
    }
}

pub(super) fn normalise_export_path_for_format(path: PathBuf, format: ExportTab) -> PathBuf {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase());
    match extension.as_deref() {
        Some(ext) if ext == format.extension() => path,
        _ => {
            let mut as_string = path.into_os_string();
            as_string.push(format!(".{}", format.extension()));
            PathBuf::from(as_string)
        }
    }
}

impl DesktopApp {
    pub(crate) fn save_monitor_image(&mut self) {
        let mut dialog = rfd::FileDialog::new()
            .add_filter("PNG image", &["png"])
            .add_filter("JPEG image", &["jpg", "jpeg"])
            .add_filter("WebP image", &["webp"])
            .add_filter("BMP image", &["bmp"])
            .set_file_name("monitor.png");
        if let Some(current) = &self.current_snapshot_path
            && let Some(parent) = current.parent()
        {
            dialog = dialog.set_directory(parent);
        }

        let Some(path) = dialog.save_file() else {
            return;
        };

        let format = match path
            .extension()
            .and_then(|s| s.to_str())
            .map(|e| e.to_ascii_lowercase())
            .as_deref()
        {
            Some("jpg") | Some("jpeg") => MonitorImageFormat::Jpeg,
            Some("webp") => MonitorImageFormat::WebP,
            Some("bmp") => MonitorImageFormat::Bmp,
            _ => MonitorImageFormat::Png,
        };

        let path = normalise_image_path(path, format);

        let bytes = match crate::view::monitor_image::render_monitor_image(
            &self.snapshot.devices.monitor,
            format,
        ) {
            Ok(b) => b,
            Err(err) => {
                tracing::error!("save monitor image: render: {err}");
                self.set_status_custom(self.lang.t(Key::MonitorImageSaveFailed).to_owned());
                return;
            }
        };

        if let Err(err) = std::fs::write(&path, &bytes) {
            tracing::error!("save monitor image to {}: {err}", path.display());
            self.set_status_custom(self.lang.t(Key::MonitorImageSaveFailed).to_owned());
            return;
        }

        self.set_status(StatusKind::MonitorImageSaved {
            display: path.display().to_string(),
        });
    }
}

fn normalise_image_path(path: PathBuf, format: MonitorImageFormat) -> PathBuf {
    let ext = format.extension();
    match path.extension().and_then(|s| s.to_str()) {
        Some(e) if e.eq_ignore_ascii_case(ext) => path,
        _ => {
            let mut s = path.into_os_string();
            s.push(".");
            s.push(ext);
            PathBuf::from(s)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::normalise_export_path_for_format;
    use crate::app::ExportTab;
    use std::path::PathBuf;

    #[test]
    fn keeps_selected_extensions_intact() {
        for (already_ok, format) in [
            ("a.txt", ExportTab::Text),
            ("a.TXT", ExportTab::Text),
            ("a.xlsx", ExportTab::Xlsx),
            ("a.XLSX", ExportTab::Xlsx),
            ("deep/path/file.txt", ExportTab::Text),
        ] {
            assert_eq!(
                normalise_export_path_for_format(PathBuf::from(already_ok), format),
                PathBuf::from(already_ok),
            );
        }
    }

    #[test]
    fn appends_txt_to_unknown_extension() {
        assert_eq!(
            normalise_export_path_for_format(PathBuf::from("mysnap.foo"), ExportTab::Text),
            PathBuf::from("mysnap.foo.txt"),
        );
        assert_eq!(
            normalise_export_path_for_format(PathBuf::from("dump.png"), ExportTab::Text),
            PathBuf::from("dump.png.txt"),
        );
    }

    #[test]
    fn appends_txt_when_no_extension() {
        assert_eq!(
            normalise_export_path_for_format(PathBuf::from("plain"), ExportTab::Text),
            PathBuf::from("plain.txt"),
        );
    }

    #[test]
    fn appends_selected_export_extension_when_no_extension() {
        assert_eq!(
            normalise_export_path_for_format(PathBuf::from("plain"), ExportTab::Xlsx),
            PathBuf::from("plain.xlsx"),
        );
    }

    /// `Path::extension` reports `.bashrc` as having no extension, so
    /// the suffix gets appended (`.bashrc.txt`).
    #[test]
    fn dotfiles_get_txt_appended() {
        assert_eq!(
            normalise_export_path_for_format(PathBuf::from(".bashrc"), ExportTab::Text),
            PathBuf::from(".bashrc.txt"),
        );
    }
}
