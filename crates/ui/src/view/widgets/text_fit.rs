/// Shortens `value` to at most `budget` characters by replacing its
/// middle with `…`. Head and tail are both kept because the callers
/// display values whose distinguishing part is the tail (a file name at
/// the end of a path, the numeric suffix of a generated target name).
pub(in crate::view) fn shorten_middle(value: &str, budget: usize) -> String {
    let chars: Vec<char> = value.chars().collect();
    if chars.len() <= budget {
        return value.to_owned();
    }
    let remaining = budget.saturating_sub(1);
    let head_len = remaining / 2;
    let tail_len = remaining - head_len;
    let head: String = chars.iter().take(head_len).collect();
    let tail: String = chars.iter().skip(chars.len() - tail_len).collect();
    format!("{head}…{tail}")
}

#[cfg(test)]
mod tests {
    use super::shorten_middle;

    #[test]
    fn file_label_shortens_from_middle() {
        let out = shorten_middle("C:\\Users\\Long\\Folder\\import-file.xlsx", 18);

        assert!(out.chars().count() <= 18);
        assert!(out.contains('…'));
    }

    #[test]
    fn value_within_budget_is_returned_unchanged() {
        assert_eq!(shorten_middle("Subprogram 11", 20), "Subprogram 11");
    }

    #[test]
    fn shortened_target_name_keeps_head_and_distinguishing_tail() {
        let out = shorten_middle("Subprogram 11dsaddddddddd", 20);

        assert!(out.chars().count() <= 20);
        assert!(out.starts_with("Subpro"));
        assert!(out.ends_with("ddd"));
    }

    #[test]
    fn multibyte_value_is_split_on_character_boundaries() {
        let out = shorten_middle("Подпрограмма 11 очень длинная", 16);

        assert!(out.chars().count() <= 16);
        assert!(out.contains('…'));
    }
}
