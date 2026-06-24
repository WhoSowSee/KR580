use std::path::{Path, PathBuf};

use super::DesktopApp;

impl DesktopApp {
    pub(crate) fn print_printer_pdf(&mut self) {
        let mut dialog = rfd::FileDialog::new()
            .add_filter("PDF", &["pdf"])
            .set_file_name("printer.pdf");
        if let Some(parent) = self
            .snapshot
            .devices
            .printer
            .target_path
            .as_ref()
            .and_then(|path| path.parent())
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            dialog = dialog.set_directory(parent);
        }
        let Some(path) = dialog.save_file() else {
            return;
        };
        self.dispatch_sync(crate::backend::AppCommand::PrintPrinterPdf(
            printer_pdf_path(&path),
        ));
    }
}

fn printer_pdf_path(path: &Path) -> PathBuf {
    if path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("pdf"))
    {
        return path.to_path_buf();
    }
    let mut raw = path.as_os_str().to_os_string();
    raw.push(".pdf");
    PathBuf::from(raw)
}

#[cfg(test)]
mod tests {
    use super::printer_pdf_path;
    use std::path::Path;

    #[test]
    fn printer_pdf_path_adds_pdf_extension_once() {
        assert_eq!(
            printer_pdf_path(Path::new("printer")),
            Path::new("printer.pdf")
        );
        assert_eq!(
            printer_pdf_path(Path::new("printer.PDF")),
            Path::new("printer.PDF")
        );
    }
}
