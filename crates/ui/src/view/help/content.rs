use std::ops::Range;

use iced::widget::text::Span;
use iced::widget::{
    button, column, container, rich_text, row, scrollable, span, text, text_editor,
};
use iced::{Background, Color, Element, Font, Length};

use super::consts::CONTENT_PADDING;
use super::styles::{breadcrumb_button_style, help_text_editor_style, hidden_scrollbar_style};
use crate::app::{
    HelpDialog, HelpMarkdownHighlight, HelpMarkdownHighlighter, HelpMarkdownLine, HelpNode,
    Message, parse_help_markdown_line,
};
use crate::i18n::Lang;
use crate::view::theme::{UI_BOLD_FONT, UI_FONT, tokyo_muted, tokyo_surface, tokyo_text};

const ARTICLE_EDITOR_HEIGHT: Length = Length::Shrink;

pub(super) fn help_content<'a>(dialog: &'a HelpDialog, lang: Lang) -> Element<'a, Message> {
    let query = dialog.results_query();
    if query.is_empty() {
        single_article(dialog, lang)
    } else {
        all_matches(dialog, query, lang)
    }
}

fn single_article<'a>(dialog: &'a HelpDialog, _lang: Lang) -> Element<'a, Message> {
    let body = text_editor(&dialog.article_content)
        .highlight_with::<HelpMarkdownHighlighter>(
            dialog.article_highlights.clone(),
            format_help_highlight,
        )
        .on_action(Message::HelpTextAction)
        .font(UI_FONT)
        .padding(CONTENT_PADDING)
        .size(14.0)
        .height(ARTICLE_EDITOR_HEIGHT)
        .style(help_text_editor_style);
    let scrollable_body = scrollable(body)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(hidden_scrollbar_style);
    container(scrollable_body)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn format_help_highlight(
    highlight: &HelpMarkdownHighlight,
    _theme: &iced::Theme,
) -> iced::advanced::text::highlighter::Format<Font> {
    match highlight {
        HelpMarkdownHighlight::Bold => iced::advanced::text::highlighter::Format {
            color: None,
            font: Some(UI_BOLD_FONT),
        },
    }
}

fn all_matches<'a>(dialog: &'a HelpDialog, query: &str, lang: Lang) -> Element<'a, Message> {
    let mut items: Vec<Element<'a, Message>> = Vec::new();
    for result in dialog.search_results() {
        let node = result.node();
        let breadcrumb = find_breadcrumb(node, lang);
        items.push(
            button(text(breadcrumb).size(13.0))
                .on_press(Message::MenuBatch(vec![
                    Message::HelpSearchChanged(String::new()),
                    Message::HelpNodeSelected(node),
                ]))
                .padding([2, 4])
                .style(|_theme, status| breadcrumb_button_style(status))
                .into(),
        );
        items.push(text("").size(2.0).into());
        let body_text = result.preview_lines().join("\n");
        items.push(render_content(&body_text, Some(query)));
        items.push(text("").size(12.0).into());
    }

    if items.is_empty() {
        items.push(
            text(lang.t(crate::i18n::Key::SettingsNoMatches))
                .size(14.0)
                .color_maybe(Some(tokyo_muted()))
                .into(),
        );
    }

    let body = column(items).spacing(8).width(Length::Fill);
    let scrollable_body = scrollable(container(body).width(Length::Fill).padding(CONTENT_PADDING))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(hidden_scrollbar_style);
    container(scrollable_body)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn find_breadcrumb(node: HelpNode, lang: Lang) -> String {
    for root in HelpNode::ROOTS {
        if let Some(path) = find_path(root, node, lang, Vec::new()) {
            return path.join(" ▸ ");
        }
    }
    lang.t(node.label_key()).to_string()
}

fn find_path(
    current: HelpNode,
    target: HelpNode,
    lang: Lang,
    mut path: Vec<String>,
) -> Option<Vec<String>> {
    path.push(lang.t(current.label_key()).to_string());
    if current == target {
        return Some(path);
    }
    for child in current.children() {
        if let Some(found) = find_path(*child, target, lang, path.clone()) {
            return Some(found);
        }
    }
    None
}

fn render_content(raw: &str, query: Option<&str>) -> Element<'static, Message> {
    let mut items: Vec<Element<'static, Message>> = Vec::new();
    for line in raw.lines() {
        let line = parse_help_markdown_line(line);
        if line.is_empty() {
            items.push(text("").size(8.0).into());
        } else if line.is_all_bold() {
            items.push(render_line(&line, query, 16.0));
            items.push(text("").size(4.0).into());
        } else if let Some(bullet) = line.strip_prefix("• ") {
            let bullet_row = row![
                text("•").size(14.0).color_maybe(Some(tokyo_text())),
                render_line(&bullet, query, 14.0),
            ]
            .spacing(6);
            items.push(bullet_row.into());
        } else {
            items.push(render_line(&line, query, 14.0));
        }
    }
    column(items).spacing(2).width(Length::Fill).into()
}

fn render_line(
    line: &HelpMarkdownLine,
    query: Option<&str>,
    size: f32,
) -> Element<'static, Message> {
    let segments = line_segments(line, query);
    if segments
        .iter()
        .all(|segment| !segment.matched && !segment.bold)
    {
        return text(line.text.to_owned())
            .font(UI_FONT)
            .size(size)
            .color_maybe(Some(tokyo_text()))
            .into();
    }

    rich_text(line_spans(segments))
        .font(UI_FONT)
        .size(size)
        .color(tokyo_text())
        .width(Length::Fill)
        .into()
}

fn line_spans(segments: Vec<TextSegment>) -> Vec<Span<'static, (), Font>> {
    segments
        .into_iter()
        .map(|segment| {
            let is_match = segment.matched;
            let color = if is_match { Color::WHITE } else { tokyo_text() };
            let mut text_span = span(segment.text).color(color);
            if segment.bold {
                text_span = text_span.font(UI_BOLD_FONT);
            }
            if is_match {
                text_span.background(Background::Color(tokyo_surface()))
            } else {
                text_span
            }
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TextSegment {
    text: String,
    matched: bool,
    bold: bool,
}

impl TextSegment {
    #[cfg(test)]
    fn new(text: &str, matched: bool, bold: bool) -> Self {
        Self {
            text: text.to_owned(),
            matched,
            bold,
        }
    }

    #[cfg(test)]
    fn text(&self) -> &str {
        &self.text
    }
}

fn line_segments(line: &HelpMarkdownLine, query: Option<&str>) -> Vec<TextSegment> {
    let match_ranges = query_match_ranges(&line.text, query);
    if match_ranges.is_empty() && line.bold_ranges.is_empty() {
        return vec![TextSegment {
            text: line.text.to_owned(),
            matched: false,
            bold: false,
        }];
    }
    split_segments(&line.text, &match_ranges, &line.bold_ranges)
}

fn query_match_ranges(text_str: &str, query: Option<&str>) -> Vec<Range<usize>> {
    let Some(query) = query.map(str::trim).filter(|query| !query.is_empty()) else {
        return Vec::new();
    };

    let lower = text_str.to_lowercase();
    let needle = query.to_lowercase();
    let mut ranges = Vec::new();
    let mut last = 0;

    for (start, _) in lower.match_indices(&needle) {
        let end = start + needle.len();
        if start < last || !text_str.is_char_boundary(start) || !text_str.is_char_boundary(end) {
            continue;
        }
        ranges.push(start..end);
        last = end;
    }

    ranges
}

fn split_segments(
    text_str: &str,
    match_ranges: &[Range<usize>],
    bold_ranges: &[Range<usize>],
) -> Vec<TextSegment> {
    let mut points = vec![0, text_str.len()];
    for range in match_ranges.iter().chain(bold_ranges) {
        if range.start <= range.end
            && text_str.is_char_boundary(range.start)
            && text_str.is_char_boundary(range.end)
        {
            points.push(range.start);
            points.push(range.end);
        }
    }

    points.sort_unstable();
    points.dedup();

    let segments = points
        .windows(2)
        .filter_map(|point| {
            let start = point[0];
            let end = point[1];
            (start < end).then(|| TextSegment {
                text: text_str[start..end].to_owned(),
                matched: range_covers(match_ranges, start, end),
                bold: range_covers(bold_ranges, start, end),
            })
        })
        .collect::<Vec<_>>();

    if segments.is_empty() {
        return vec![TextSegment {
            text: text_str.to_owned(),
            matched: false,
            bold: false,
        }];
    }
    segments
}

fn range_covers(ranges: &[Range<usize>], start: usize, end: usize) -> bool {
    ranges
        .iter()
        .any(|range| start >= range.start && end <= range.end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn article_editor_yields_wheel_scrolling_to_parent() {
        assert_eq!(ARTICLE_EDITOR_HEIGHT, Length::Shrink);
    }

    #[test]
    fn line_segments_mark_only_matching_letters() {
        let line = HelpMarkdownLine {
            text: "KR580 – эмулятор системы".to_owned(),
            bold_ranges: Vec::new(),
        };
        let segments = line_segments(&line, Some("ЭМУЛЯТОР"));

        assert_eq!(
            segments,
            vec![
                TextSegment::new("KR580 – ", false, false),
                TextSegment::new("эмулятор", true, false),
                TextSegment::new(" системы", false, false),
            ]
        );
    }

    #[test]
    fn line_segments_do_not_emit_arrow_markers() {
        let line = HelpMarkdownLine {
            text: "эмулятор и эмулятор".to_owned(),
            bold_ranges: Vec::new(),
        };
        let segments = line_segments(&line, Some("эмулятор"));
        let rendered = segments
            .iter()
            .map(TextSegment::text)
            .collect::<Vec<_>>()
            .join("");

        assert_eq!(rendered, "эмулятор и эмулятор");
        assert!(!rendered.contains(">>"));
        assert!(!rendered.contains("<<"));
    }

    #[test]
    fn markdown_bold_markers_are_removed_in_search_lines() {
        let line = parse_help_markdown_line("**NOP (00h)** – пустая операция");
        let segments = line_segments(&line, Some("nop"));
        let rendered = segments
            .iter()
            .map(TextSegment::text)
            .collect::<Vec<_>>()
            .join("");

        assert_eq!(rendered, "NOP (00h) – пустая операция");
        assert_eq!(segments[0], TextSegment::new("NOP", true, true));
        assert!(!rendered.contains("**"));
    }

    #[test]
    fn matching_segments_use_sidebar_selection_background() {
        let spans = line_spans(vec![TextSegment::new("эмулятор", true, false)]);
        let highlight = spans[0]
            .highlight
            .expect("match span should be highlighted");

        assert_eq!(
            highlight.background,
            Background::Color(crate::view::theme::tokyo_surface())
        );
    }

    #[test]
    fn bold_segments_use_bold_font() {
        let spans = line_spans(vec![TextSegment::new("NOP", false, true)]);

        assert_eq!(spans[0].font, Some(UI_BOLD_FONT));
        assert!(spans[0].highlight.is_none());
    }
}
