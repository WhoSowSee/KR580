use crate::i18n::{Key, Lang};

pub(crate) fn humanize(raw: &str, lang: Lang) -> String {
    let lower = raw.to_lowercase();

    if lower.contains("not a .580 file") {
        return lang.t(Key::ErrNotA580File).to_owned();
    }
    if lower.contains("file is empty") {
        return lang.t(Key::ErrFileEmpty).to_owned();
    }
    if lower.contains("expected ") && lower.contains(" bytes, got ") {
        return lang.t(Key::ErrWrong580Size).to_owned();
    }
    if lower.contains("legacy .580 trailer") {
        return lang.t(Key::ErrLegacyTrailerCorrupt).to_owned();
    }

    if lower.contains("unsupported settings version") {
        return lang.t(Key::ErrSettingsNewerVersion).to_owned();
    }
    if lower.contains("settings json error") {
        return lang.t(Key::ErrSettingsCorrupt).to_owned();
    }

    if lower.contains("malformed import file") || lower.contains("spreadsheet import error") {
        return lang.t(Key::ErrCannotReadFileFormat).to_owned();
    }
    if lower.contains("import i/o error") {
        return lang.t(Key::ErrCannotReadFile).to_owned();
    }
    if lower.contains("spreadsheet export error") {
        return lang.t(Key::ErrCannotWriteTable).to_owned();
    }
    if lower.contains("export i/o error") {
        return lang.t(Key::ErrCannotWriteFile).to_owned();
    }

    if lower.contains("not found") || lower.contains("no such file") || lower.contains("os error 2")
    {
        return lang.t(Key::ErrFileNotFound).to_owned();
    }
    if lower.contains("permission denied") || lower.contains("os error 5") {
        return lang.t(Key::ErrPermissionDenied).to_owned();
    }
    if lower.contains("already exists") {
        return lang.t(Key::ErrFileAlreadyExists).to_owned();
    }
    if lower.contains("disk") || lower.contains("space") {
        return lang.t(Key::ErrDiskFull).to_owned();
    }
    if lower.starts_with("i/o error") || lower.starts_with("io error") {
        return lang.t(Key::ErrIoGeneric).to_owned();
    }

    if lower.contains("address range") {
        return lang.t(Key::ErrAddressOutOfRange).to_owned();
    }
    if lower.contains("invalid register name") {
        return lang.t(Key::ErrUnknownRegister).to_owned();
    }
    if lower.contains("undocumented opcode") {
        return lang.t(Key::ErrUndocumentedOpcode).to_owned();
    }
    if lower.contains("device is not ready") {
        return lang.t(Key::ErrFloppyImageNotAttached).to_owned();
    }

    if lower.contains("worker stopped") {
        return lang.t(Key::ErrInternal).to_owned();
    }

    // Fallback keeps the original in parens so a screenshot still
    // helps when the user reports the issue.
    format!("{} ({raw})", lang.t(Key::ErrGenericFailed))
}

#[cfg(test)]
mod tests {
    use super::humanize;
    use crate::i18n::Lang;

    #[test]
    fn program_diagnostics_are_localized() {
        assert!(humanize("not a .580 file", Lang::Ru).contains(".580"));
        assert!(humanize("file is empty", Lang::Ru).contains("пуст"));
        assert!(humanize("expected 65549 bytes, got 70000", Lang::Ru).contains("65549"));
    }

    #[test]
    fn version_skew_has_its_own_message() {
        assert!(humanize("unsupported settings version 2", Lang::Ru).contains("новой версии"));
    }

    #[test]
    fn io_kind_phrases_are_translated() {
        assert_eq!(
            humanize(
                "The system cannot find the file specified. (os error 2)",
                Lang::Ru
            ),
            "Файл не найден"
        );
        assert_eq!(
            humanize("Access is denied. (os error 5)", Lang::Ru),
            "Нет доступа к файлу"
        );
        assert_eq!(humanize("entity not found", Lang::Ru), "Файл не найден");
        assert_eq!(
            humanize("permission denied", Lang::Ru),
            "Нет доступа к файлу"
        );
        assert_eq!(
            humanize("permission denied", Lang::En),
            "Permission denied for file"
        );
    }

    #[test]
    fn device_not_ready_points_to_missing_floppy_image() {
        assert_eq!(
            humanize("core error: device is not ready", Lang::Ru),
            "Файл образа дисковода не подключён"
        );
        assert_eq!(
            humanize("core error: device is not ready", Lang::En),
            "Floppy image file is not attached"
        );
    }

    #[test]
    fn unknown_messages_fall_back_with_suffix() {
        let raw = "totally novel error not seen before";
        let humanized = humanize(raw, Lang::Ru);
        assert!(humanized.starts_with("Не удалось выполнить операцию"));
        assert!(humanized.contains(raw));
    }
}
