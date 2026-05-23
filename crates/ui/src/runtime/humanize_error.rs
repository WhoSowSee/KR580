//! Translates the English `thiserror` messages produced by the
//! domain crates (`k580-core`, `k580-persistence`, `std::io`) into
//! short, user-facing Russian phrases for the floating error
//! overlay. The user reported the raw English / code-shaped messages
//! ("malformed import file: …", "legacy .580 file must be exactly
//! 65549 bytes, got 1024", `os error 2`) as unreadable — what they
//! want is one plain sentence telling them what went wrong without
//! hex constants or implementation jargon.
//!
//! Why pattern-match on the rendered string instead of typing on
//! the original error enum: the worker hands the UI an
//! `AppError::{Core, Persistence, Io, WorkerStopped}` whose payload
//! is already a `String` (see `crates/app/src/error.rs`). Reaching
//! through that boundary to recover the original `SnapshotError`
//! variant would mean reshaping the entire error pipeline for a
//! purely cosmetic UI concern. Matching on prefixes/substrings here
//! keeps localization confined to the layer that actually displays
//! it, and the prefixes themselves are stable — they live in
//! `#[error(...)]` attributes inside source files this workspace
//! owns.
//!
//! The matcher is best-effort: an unrecognised message falls back
//! to a generic "Файл не удалось обработать" with the original
//! attached in parentheses so the technical text is still recoverable
//! when a user shows the overlay to a developer.
//!
//! Tested against the canonical Display strings inside
//! `tests` below — every domain variant the UI surfaces today has a
//! match arm, and the fallback path is also exercised so a future
//! variant addition surfaces as a UX-but-not-functional regression.

/// Maps a domain error message to a short Russian sentence the user
/// can act on. The input is the Display string of `AppError`,
/// already stripped of its `Ошибка: ` prefix at the call site (the
/// caller composes the prefix and the localized message together).
pub(super) fn humanize(raw: &str) -> String {
    // Lowercase view used purely for case-insensitive substring
    // probes. We still hand the original `raw` to the fallback
    // branch so the developer-visible suffix preserves casing of
    // any embedded paths or hex constants.
    let lower = raw.to_lowercase();

    // Snapshot / legacy file format diagnostics. The user just tried
    // to open a `.580` file and the parser bailed; the actionable
    // message is "this isn't a valid file of this kind", with the
    // specific reason a secondary cue.
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

    // Settings serialization. Almost no users touch this path
    // directly, but a corrupt `settings.json` from a crashed prior
    // run can surface here.
    if lower.contains("unsupported settings version") {
        return "Настройки сохранены в более новой версии — обновите программу".to_owned();
    }
    if lower.contains("settings json error") {
        return "Файл настроек повреждён".to_owned();
    }

    // Import / export pipelines. These wrap calamine errors and our
    // own malformed-input messages; collapsing them into two arms
    // (read failure vs. write failure) is enough for the UI.
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

    // Bare I/O errors come through `AppError::Io` and a generic
    // `std::io::Error::to_string`. The OS-level codes ("os error 2",
    // "os error 5", …) are noise; surface the typical Russian phrase
    // instead. Probe for the more specific kinds first.
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

    // Core-side guards: address out of range, bad register names,
    // bad opcodes. None of these are reachable from the file
    // handlers today, but the worker can route them through
    // `AppError::Core` if the user's gesture happens to land on a
    // ResetCpu mid-step or similar; covering them here keeps the
    // fallback branch from carrying the only Russian message for
    // the entire core domain.
    if lower.contains("address range") {
        return "Адрес вне допустимого диапазона памяти".to_owned();
    }
    if lower.contains("invalid register name") {
        return "Неизвестное имя регистра".to_owned();
    }
    if lower.contains("undocumented opcode") {
        return "Недокументированная команда".to_owned();
    }

    // Worker channel torn down — generally a developer-side bug, but
    // the user still deserves a non-technical line.
    if lower.contains("worker stopped") {
        return "Внутренняя ошибка приложения".to_owned();
    }

    // Fallback: unknown variant. Keep the original (it will surface
    // unhelpfully in English / hex but at least be recoverable from
    // a screenshot when the user reports the issue) attached in
    // parens after a generic line.
    format!("Не удалось выполнить операцию ({raw})")
}

#[cfg(test)]
mod tests {
    use super::humanize;

    /// Anchor: every Display string the UI surfaces today has a
    /// localized arm and never falls into the bracket-suffix
    /// branch. If a future error variant is added without a match
    /// arm, that variant's test addition will fail here and the
    /// developer is forced to localize before merging.
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

    /// Version-skew has its own arm because the actionable advice
    /// is different from "file is corrupt": the user can fix the
    /// version skew (update the app), but cannot fix a corrupt
    /// file beyond re-saving it.
    #[test]
    fn version_skew_has_its_own_message() {
        assert!(humanize("unsupported .580 version 2").contains("новой версии"));
        assert!(humanize("unsupported settings version 2").contains("новой версии"));
    }

    /// Legacy `.580` failures get distinct phrasing because the user
    /// just clicked a menu item that explicitly says "старый формат"
    /// — surfacing the same generic "файл повреждён" they would see
    /// for a v1 read would be confusing.
    #[test]
    fn legacy_diagnostics_are_distinct() {
        assert!(
            humanize("legacy .580 file must be exactly 65549 bytes, got 1024")
                .contains("в старом формате")
        );
        assert!(humanize("legacy .580 trailer is missing the FF FF end marker").contains("Конец"));
    }

    /// `std::io::Error` messages are matched by both kind name and
    /// the Windows OS-error code so neither platform leaks the raw
    /// number.
    #[test]
    fn io_kind_phrases_are_translated() {
        assert_eq!(humanize("The system cannot find the file specified. (os error 2)"), "Файл не найден");
        assert_eq!(humanize("Access is denied. (os error 5)"), "Нет доступа к файлу");
        assert_eq!(humanize("entity not found"), "Файл не найден");
        assert_eq!(humanize("permission denied"), "Нет доступа к файлу");
    }

    /// Anything outside the known set still lands on a Russian line
    /// — the fallback keeps the technical text in parentheses so a
    /// support thread can recover it from a screenshot.
    #[test]
    fn unknown_messages_fall_back_with_suffix() {
        let raw = "totally novel error not seen before";
        let humanized = humanize(raw);
        assert!(humanized.starts_with("Не удалось выполнить операцию"));
        assert!(humanized.contains(raw));
    }
}
