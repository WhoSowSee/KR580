//! Translates English `thiserror` messages into short Russian
//! phrases for the floating error overlay. Pattern-matches on the
//! rendered string because `AppError::*` already carries a `String`
//! payload — typing the original variant would reshape the entire
//! error pipeline for a cosmetic concern.

pub(super) fn humanize(raw: &str) -> String {
    let lower = raw.to_lowercase();

    if lower.contains("invalid .580 magic")
        || lower.contains("snapshot data is truncated")
        || lower.contains("payload length does not match")
        || lower.contains("unsupported snapshot tlv tag")
        || lower.contains("invalid length")
        || lower.contains("required snapshot tag")
    {
        return "Файл повреждён или имеет неподдерживаемый формат".to_owned();
    }
    if lower.contains("unsupported .580 version") {
        return "Файл сохранён в более новой версии — обновите программу".to_owned();
    }
    if lower.contains("legacy .580 file must be exactly") {
        return "Файл не похож на сохранение в старом формате".to_owned();
    }
    if lower.contains("legacy .580 trailer") {
        return "Конец файла повреждён — это не сохранение в старом формате".to_owned();
    }

    if lower.contains("unsupported settings version") {
        return "Настройки сохранены в более новой версии — обновите программу".to_owned();
    }
    if lower.contains("settings json error") {
        return "Файл настроек повреждён".to_owned();
    }

    if lower.contains("malformed import file") || lower.contains("spreadsheet import error") {
        return "Не удалось прочитать файл — проверьте формат".to_owned();
    }
    if lower.contains("import i/o error") {
        return "Не удалось прочитать файл".to_owned();
    }
    if lower.contains("spreadsheet export error") {
        return "Не удалось записать таблицу".to_owned();
    }
    if lower.contains("export i/o error") {
        return "Не удалось записать файл".to_owned();
    }

    if lower.contains("not found") || lower.contains("no such file") || lower.contains("os error 2")
    {
        return "Файл не найден".to_owned();
    }
    if lower.contains("permission denied") || lower.contains("os error 5") {
        return "Нет доступа к файлу".to_owned();
    }
    if lower.contains("already exists") {
        return "Файл уже существует".to_owned();
    }
    if lower.contains("disk") || lower.contains("space") {
        return "На диске недостаточно места".to_owned();
    }
    if lower.starts_with("i/o error") || lower.starts_with("io error") {
        return "Ошибка чтения или записи файла".to_owned();
    }

    if lower.contains("address range") {
        return "Адрес вне допустимого диапазона памяти".to_owned();
    }
    if lower.contains("invalid register name") {
        return "Неизвестное имя регистра".to_owned();
    }
    if lower.contains("undocumented opcode") {
        return "Недокументированная команда".to_owned();
    }

    if lower.contains("worker stopped") {
        return "Внутренняя ошибка приложения".to_owned();
    }

    // Fallback keeps the original in parens so a screenshot still
    // helps when the user reports the issue.
    format!("Не удалось выполнить операцию ({raw})")
}

#[cfg(test)]
mod tests {
    use super::humanize;

    #[test]
    fn snapshot_format_diagnostics_are_localized() {
        for raw in [
            "invalid .580 magic header",
            "snapshot data is truncated",
            "payload length does not match the header",
            "unsupported snapshot TLV tag 0x09",
            "invalid length 5 for tag 0x01",
            "required snapshot tag 0x01 is missing",
        ] {
            let humanized = humanize(raw);
            assert!(
                humanized.contains("повреждён") || humanized.contains("неподдерживаемый формат"),
                "{raw} did not localize: {humanized}"
            );
        }
    }

    #[test]
    fn version_skew_has_its_own_message() {
        assert!(humanize("unsupported .580 version 2").contains("новой версии"));
        assert!(humanize("unsupported settings version 2").contains("новой версии"));
    }

    #[test]
    fn legacy_diagnostics_are_distinct() {
        assert!(
            humanize("legacy .580 file must be exactly 65549 bytes, got 1024")
                .contains("в старом формате")
        );
        assert!(humanize("legacy .580 trailer is missing the FF FF end marker").contains("Конец"));
    }

    #[test]
    fn io_kind_phrases_are_translated() {
        assert_eq!(
            humanize("The system cannot find the file specified. (os error 2)"),
            "Файл не найден"
        );
        assert_eq!(
            humanize("Access is denied. (os error 5)"),
            "Нет доступа к файлу"
        );
        assert_eq!(humanize("entity not found"), "Файл не найден");
        assert_eq!(humanize("permission denied"), "Нет доступа к файлу");
    }

    #[test]
    fn unknown_messages_fall_back_with_suffix() {
        let raw = "totally novel error not seen before";
        let humanized = humanize(raw);
        assert!(humanized.starts_with("Не удалось выполнить операцию"));
        assert!(humanized.contains(raw));
    }
}
