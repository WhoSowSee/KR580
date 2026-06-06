use super::*;

fn px_for_chars(chars: usize) -> f32 {
    STATUS_LEFT_RESERVATION_PX + chars as f32 * STATUS_GLYPH_WIDTH_PX
}

#[test]
fn fits_in_wide_window_passes_through_unchanged() {
    let text = "Открыто C:\\Users\\me\\file.580";
    let out = shorten_status_for_width(text, px_for_chars(200));
    assert_eq!(out, text);
}

#[test]
fn narrow_window_path_aware_truncation_keeps_separators_and_filename() {
    let text = "Открыто C:\\Users\\whosowsee\\Downloads\\monitor.png";
    let out = shorten_status_for_width(text, px_for_chars(40));
    assert!(out.chars().count() <= 40, "got: {out}");
    assert!(out.starts_with("Открыто C:\\"));
    assert!(out.ends_with("\\monitor.png"));
    assert!(out.contains('…'));
}

#[test]
fn middle_segment_truncates_from_the_end() {
    let text = "Saved C:\\AveryLongFolderName\\file.txt";
    let out = shorten_status_for_width(text, px_for_chars(30));
    assert!(out.chars().count() <= 30, "got: {out}");
    assert!(out.starts_with("Saved C:\\"));
    assert!(out.ends_with("\\file.txt"));
}

#[test]
fn forward_slash_paths_preserve_their_separator() {
    let text = "Saved /home/whosowsee/Downloads/screenshots/monitor.png";
    let out = shorten_status_for_width(text, px_for_chars(40));
    assert!(out.chars().count() <= 40, "got: {out}");
    assert!(!out.contains('\\'));
    assert!(out.ends_with("/monitor.png"));
}

#[test]
fn cyrillic_segments_are_truncated_safely() {
    let text = "Saved C:\\Пользователи\\Документы\\Проект\\Подпапка\\Файл с длинным именем.580";
    let out = shorten_status_for_width(text, px_for_chars(40));
    assert!(out.chars().count() <= 40, "got: {out}");
    assert!(out.ends_with(".580"));
}

#[test]
fn extremely_narrow_window_falls_back_to_middle_truncate() {
    let text = "Saved C:\\Users\\whosowsee\\Downloads\\monitor.png";
    let out = shorten_status_for_width(text, px_for_chars(8));
    assert!(out.chars().count() <= 8, "got: {out}");
    assert!(out.contains('…'));
}

#[test]
fn zero_budget_does_not_panic() {
    let text = "Saved C:\\Users\\whosowsee\\Downloads\\monitor.png";
    let out = shorten_status_for_width(text, 0.0);
    assert_eq!(out, text);
}

#[test]
fn legacy_style_status_without_colon_still_truncates_path() {
    let text = "Открыто C:\\Users\\whosowsee\\rybenich\\MPS\\Practical5\\task_palette7.580";
    let out = shorten_status_for_width(text, px_for_chars(56));
    assert!(out.chars().count() <= 56, "got: {out}");
    assert!(out.starts_with("Открыто C:\\"));
    assert!(out.ends_with("\\task_palette7.580"));
    let path_part = out.strip_prefix("Открыто ").unwrap_or("");
    let separator_count = path_part.matches('\\').count();
    assert!(
        separator_count >= 2,
        "path should keep at least two separators: {path_part}"
    );
}
