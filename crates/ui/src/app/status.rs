use crate::i18n::{Key, Lang};

/// Provenance of the live `status` string — used to re-render the
/// status bar when the UI language switches at runtime. Each variant
/// owns the raw values (numbers, paths, mnemonics) that the rendered
/// string was built from; the language-dependent prefix / unit comes
/// from `Lang::t` at render time inside [`StatusKind::render`].
#[derive(Clone, Debug)]
pub(crate) enum StatusKind {
    /// Set from a non-canonical source (raw error, port log line,
    /// search-error message). `render` returns `None` so the caller
    /// keeps the existing string verbatim.
    Custom,
    Ready,
    NewFile,
    CpuHalted,
    Stopped,
    TactProgress {
        tact_phase: u8,
        cycle_count: u64,
    },
    InstructionAt {
        mnemonic: String,
        pc_before: u16,
    },
    PortRead {
        port: u8,
        value: u8,
    },
    PortWrite {
        port: u8,
        value: u8,
    },
    NoProgramAt {
        pc: u16,
    },
    Opened {
        display: String,
        legacy: bool,
    },
    SavedTo {
        display: String,
        legacy: bool,
    },
    ExportTo {
        display: String,
    },
    MonitorImageSaved {
        display: String,
    },
    NothingToUndo,
    NothingToRedo,
    EnterHexPattern,
    PatternFound {
        pattern: String,
        address: u16,
    },
    NoMatchesFor {
        pattern: String,
    },
}

impl StatusKind {
    pub(crate) fn render(&self, lang: Lang) -> Option<String> {
        Some(match self {
            Self::Custom => return None,
            Self::Ready => lang.t(Key::StatusReady).to_owned(),
            Self::NewFile => lang.t(Key::StatusNewFile).to_owned(),
            Self::CpuHalted => lang.t(Key::StatusCpuHalted).to_owned(),
            Self::Stopped => lang.t(Key::StatusStopped).to_owned(),
            Self::TactProgress {
                tact_phase,
                cycle_count,
            } => format!(
                "{} {} {} {}",
                lang.t(Key::StatusTact),
                tact_phase,
                lang.t(Key::StatusCycle),
                cycle_count
            ),
            Self::InstructionAt {
                mnemonic,
                pc_before,
            } => format!("{mnemonic} at {pc_before:04X}"),
            Self::PortRead { port, value } => format!("IN {port:02X} -> {value:02X}"),
            Self::PortWrite { port, value } => format!("OUT {port:02X} <- {value:02X}"),
            Self::NoProgramAt { pc } => {
                format!("{} {pc:04X}", lang.t(Key::StatusNoProgramAt))
            }
            Self::Opened { display, legacy } => {
                if *legacy {
                    format!(
                        "{} {display} ({})",
                        lang.t(Key::StatusOpened),
                        lang.t(Key::LegacyFormatNote)
                    )
                } else {
                    format!("{} {display}", lang.t(Key::StatusOpened))
                }
            }
            Self::SavedTo { display, legacy } => {
                if *legacy {
                    format!(
                        "{} {display} ({})",
                        lang.t(Key::StatusSavedTo),
                        lang.t(Key::LegacyFormatNote)
                    )
                } else {
                    format!("{} {display}", lang.t(Key::StatusSavedTo))
                }
            }
            Self::ExportTo { display } => format!("{} {display}", lang.t(Key::StatusExportTo)),
            Self::MonitorImageSaved { display } => {
                format!("{}: {display}", lang.t(Key::MonitorImageSaved))
            }
            Self::NothingToUndo => lang.t(Key::StatusNothingToUndo).to_owned(),
            Self::NothingToRedo => lang.t(Key::StatusNothingToRedo).to_owned(),
            Self::EnterHexPattern => lang.t(Key::StatusEnterHexPattern).to_owned(),
            Self::PatternFound { pattern, address } => format!(
                "{} {pattern} {} {address:04X}",
                lang.t(Key::StatusPatternFound),
                lang.t(Key::StatusAtAddress)
            ),
            Self::NoMatchesFor { pattern } => {
                format!("{} {pattern}", lang.t(Key::StatusNoMatchesFor))
            }
        })
    }
}

const ELLIPSIS: char = '…';

/// Approximate width of one monospaced 13-pt glyph (the size used by the
/// status text widget). The status row spans the right half of the
/// header, so the available chars budget is derived from the window
/// width with a fixed reservation for the left-hand register strip and
/// the "Статус"/"Status" label.
const STATUS_LEFT_RESERVATION_PX: f32 = 600.0;
const STATUS_GLYPH_WIDTH_PX: f32 = 9.0;

pub(crate) fn shorten_status_for_width(text: &str, window_width: f32) -> String {
    let total_len = text.chars().count();
    let available_px = (window_width - STATUS_LEFT_RESERVATION_PX).max(0.0);
    let budget = (available_px / STATUS_GLYPH_WIDTH_PX).floor() as usize;
    if budget == 0 || total_len <= budget {
        return text.to_owned();
    }

    if let Some(path_start) = locate_path_start(text) {
        let prefix = &text[..path_start];
        let path = &text[path_start..];
        let prefix_len = prefix.chars().count();
        if prefix_len + 4 <= budget {
            let path_budget = budget - prefix_len;
            let shortened = shorten_path_segment(path, path_budget);
            return format!("{prefix}{shortened}");
        }
    }

    char_middle_truncate(text, budget)
}

fn locate_path_start(text: &str) -> Option<usize> {
    if let Some(idx) = find_drive_start(text) {
        return Some(idx);
    }
    let mut search_from = 0;
    while let Some(rel) = text[search_from..].find('/') {
        let abs = search_from + rel;
        let bytes = text.as_bytes();
        if abs == 0 || bytes[abs - 1] == b' ' {
            return Some(abs);
        }
        search_from = abs + 1;
    }
    None
}

fn find_drive_start(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    for (idx, window) in bytes.windows(3).enumerate() {
        let letter = window[0];
        let colon = window[1];
        let sep = window[2];
        if !(letter.is_ascii_uppercase() || letter.is_ascii_lowercase()) {
            continue;
        }
        if colon != b':' {
            continue;
        }
        if sep != b'\\' && sep != b'/' {
            continue;
        }
        if idx > 0 && bytes[idx - 1] != b' ' {
            continue;
        }
        return Some(idx);
    }
    None
}

fn shorten_path_segment(path: &str, budget: usize) -> String {
    if path.chars().count() <= budget {
        return path.to_owned();
    }

    let separator = pick_separator(path);
    let segments: Vec<&str> = path.split(separator).collect();

    if segments.len() < 3 {
        return char_middle_truncate(path, budget);
    }

    let n = segments.len();
    let drive = segments[0];
    let file = segments[n - 1];
    let middle = &segments[1..n - 1];

    let frame_len = char_count(drive) + char_count(file) + 2;
    if frame_len >= budget {
        return char_middle_truncate(path, budget);
    }

    let collapsed_cost = 2;
    let mut tail_kept: Vec<&str> = Vec::with_capacity(middle.len());
    let mut remaining = budget - frame_len;

    for segment in middle.iter().rev() {
        let needs_collapse = tail_kept.len() < middle.len() - 1;
        let with_sep = char_count(segment) + 1;
        let reserve = if needs_collapse { collapsed_cost } else { 0 };
        if with_sep + reserve > remaining {
            break;
        }
        tail_kept.push(*segment);
        remaining -= with_sep;
    }
    tail_kept.reverse();

    let collapsed = tail_kept.len() < middle.len();
    let sep_str = separator.to_string();
    let mut out = String::new();
    out.push_str(drive);
    if collapsed {
        out.push_str(&sep_str);
        out.push(ELLIPSIS);
    }
    for segment in &tail_kept {
        out.push_str(&sep_str);
        out.push_str(segment);
    }
    out.push_str(&sep_str);
    out.push_str(file);

    let last_middle = middle.last().copied().unwrap_or("");
    let last_in_tail = tail_kept.last().copied() == Some(last_middle);
    if out.chars().count() <= budget && last_in_tail {
        return out;
    }

    let preface_collapse = middle.len() > tail_kept.len() + 1;
    let trailing_kept = if last_in_tail {
        &tail_kept[..tail_kept.len() - 1]
    } else {
        &tail_kept[..]
    };
    let mut reserved = char_count(drive) + char_count(file) + 2;
    if preface_collapse {
        reserved += collapsed_cost;
    }
    for kept in trailing_kept {
        reserved += char_count(kept) + 1;
    }
    let max_seg = budget.saturating_sub(reserved);
    if max_seg >= 2 {
        let truncated = truncate_tail(last_middle, max_seg);
        let mut out2 = String::new();
        out2.push_str(drive);
        if preface_collapse {
            out2.push_str(&sep_str);
            out2.push(ELLIPSIS);
        }
        for kept in trailing_kept {
            out2.push_str(&sep_str);
            out2.push_str(kept);
        }
        out2.push_str(&sep_str);
        out2.push_str(&truncated);
        out2.push_str(&sep_str);
        out2.push_str(file);
        if out2.chars().count() <= budget {
            return out2;
        }
    }

    if out.chars().count() <= budget {
        return out;
    }

    char_middle_truncate(path, budget)
}

fn pick_separator(s: &str) -> char {
    let backslashes = s.matches('\\').count();
    let forward = s.matches('/').count();
    if backslashes >= forward { '\\' } else { '/' }
}

fn char_count(s: &str) -> usize {
    s.chars().count()
}

fn truncate_tail(segment: &str, max: usize) -> String {
    let chars: Vec<char> = segment.chars().collect();
    if chars.len() <= max {
        return segment.to_owned();
    }
    let keep = max.saturating_sub(1);
    let mut out: String = chars.into_iter().take(keep).collect();
    out.push(ELLIPSIS);
    out
}

fn char_middle_truncate(text: &str, budget: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= budget {
        return text.to_owned();
    }
    let remaining = budget.saturating_sub(1);
    let head_len = remaining / 2;
    let tail_len = remaining - head_len;
    let head: String = chars.iter().take(head_len).collect();
    let tail: String = chars.iter().skip(chars.len() - tail_len).collect();
    format!("{head}{ELLIPSIS}{tail}")
}

#[cfg(test)]
mod tests {
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
}
