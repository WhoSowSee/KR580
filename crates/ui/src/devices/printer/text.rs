use crate::devices::decode_oem_text;

const COLUMNS_PER_LINE: usize = 80;

pub(super) fn printer_lines(spool: &[u8]) -> Vec<String> {
    let text = decode_oem_text(spool).replace('\t', "    ");
    let mut lines = Vec::new();
    for logical_line in text.split('\n') {
        let characters = logical_line.chars().collect::<Vec<_>>();
        if characters.is_empty() {
            lines.push(String::new());
            continue;
        }
        lines.extend(
            characters
                .chunks(COLUMNS_PER_LINE)
                .map(|chunk| chunk.iter().collect()),
        );
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::printer_lines;

    #[test]
    fn printer_lines_preserve_blank_lines_and_wrap_at_eighty_columns() {
        let source = format!("{}\r\n\r\nB", "A".repeat(81));
        let lines = printer_lines(source.as_bytes());

        assert_eq!(lines[0], "A".repeat(80));
        assert_eq!(lines[1], "A");
        assert_eq!(lines[2], "");
        assert_eq!(lines[3], "B");
    }
}
