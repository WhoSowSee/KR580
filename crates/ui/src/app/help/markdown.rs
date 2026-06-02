use std::ops::Range;

use iced::widget::text::Highlighter;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HelpMarkdownDocument {
    pub(crate) text: String,
    pub(crate) highlights: HelpMarkdownHighlights,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HelpMarkdownLine {
    pub(crate) text: String,
    pub(crate) bold_ranges: Vec<Range<usize>>,
}

impl HelpMarkdownLine {
    pub(crate) fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    pub(crate) fn is_all_bold(&self) -> bool {
        self.bold_ranges.len() == 1 && self.bold_ranges[0] == (0..self.text.len())
    }

    pub(crate) fn strip_prefix(&self, prefix: &str) -> Option<Self> {
        let text = self.text.strip_prefix(prefix)?;
        let offset = prefix.len();
        let bold_ranges = self
            .bold_ranges
            .iter()
            .filter(|range| range.end > offset)
            .map(|range| range.start.saturating_sub(offset)..range.end.saturating_sub(offset))
            .collect();
        Some(Self {
            text: text.to_owned(),
            bold_ranges,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HelpMarkdownHighlights {
    lines: Vec<Vec<Range<usize>>>,
}

impl HelpMarkdownHighlights {
    fn new(lines: Vec<Vec<Range<usize>>>) -> Self {
        Self { lines }
    }

    fn line(&self, index: usize) -> Vec<Range<usize>> {
        self.lines.get(index).cloned().unwrap_or_default()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HelpMarkdownHighlight {
    Bold,
}

pub(crate) struct HelpMarkdownHighlighter {
    settings: HelpMarkdownHighlights,
    current_line: usize,
}

impl Highlighter for HelpMarkdownHighlighter {
    type Settings = HelpMarkdownHighlights;
    type Highlight = HelpMarkdownHighlight;
    type Iterator<'a> = std::vec::IntoIter<(Range<usize>, Self::Highlight)>;

    fn new(settings: &Self::Settings) -> Self {
        Self {
            settings: settings.clone(),
            current_line: 0,
        }
    }

    fn update(&mut self, new_settings: &Self::Settings) {
        self.settings = new_settings.clone();
        self.current_line = 0;
    }

    fn change_line(&mut self, line: usize) {
        self.current_line = line;
    }

    fn highlight_line(&mut self, _line: &str) -> Self::Iterator<'_> {
        let line = self.current_line;
        self.current_line = self.current_line.saturating_add(1);
        self.settings
            .line(line)
            .into_iter()
            .map(|range| (range, HelpMarkdownHighlight::Bold))
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn current_line(&self) -> usize {
        self.current_line
    }
}

pub(crate) fn parse_help_markdown(raw: &str) -> HelpMarkdownDocument {
    let lines = raw
        .lines()
        .map(parse_help_markdown_line)
        .collect::<Vec<_>>();
    let text = lines
        .iter()
        .map(|line| line.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let highlights = HelpMarkdownHighlights::new(
        lines
            .into_iter()
            .map(|line| line.bold_ranges)
            .collect::<Vec<_>>(),
    );
    HelpMarkdownDocument { text, highlights }
}

pub(crate) fn parse_help_markdown_line(raw: &str) -> HelpMarkdownLine {
    let mut input = raw.trim();
    let mut text = String::new();
    let mut bold_ranges = Vec::new();

    while let Some(start) = input.find("**") {
        text.push_str(&input[..start]);
        let after_start = &input[start + 2..];
        let Some(end) = after_start.find("**") else {
            text.push_str(&input[start..]);
            input = "";
            break;
        };
        let bold = &after_start[..end];
        let range_start = text.len();
        text.push_str(bold);
        let range_end = text.len();
        if range_start < range_end {
            bold_ranges.push(range_start..range_end);
        }
        input = &after_start[end + 2..];
    }

    text.push_str(input);
    HelpMarkdownLine { text, bold_ranges }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_bold_markers_are_removed_and_ranged() {
        let line = parse_help_markdown_line("**NOP (00h)** – пустая операция");

        assert_eq!(line.text, "NOP (00h) – пустая операция");
        assert_eq!(line.bold_ranges, vec![0..9]);
    }

    #[test]
    fn multiple_bold_ranges_are_preserved() {
        let line = parse_help_markdown_line("A **BC** D **EF**");

        assert_eq!(line.text, "A BC D EF");
        assert_eq!(line.bold_ranges, vec![2..4, 7..9]);
    }

    #[test]
    fn unmatched_marker_stays_literal() {
        let line = parse_help_markdown_line("A **BC");

        assert_eq!(line.text, "A **BC");
        assert!(line.bold_ranges.is_empty());
    }

    #[test]
    fn document_highlights_follow_line_numbers() {
        let document = parse_help_markdown("A\n**B**");
        let mut highlighter = HelpMarkdownHighlighter::new(&document.highlights);

        assert_eq!(document.text, "A\nB");
        assert_eq!(highlighter.highlight_line("A").collect::<Vec<_>>(), vec![]);
        assert_eq!(
            highlighter.highlight_line("B").collect::<Vec<_>>(),
            vec![(0..1, HelpMarkdownHighlight::Bold)]
        );
    }
}
