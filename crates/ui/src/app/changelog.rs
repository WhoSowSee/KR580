use iced::widget::text_editor;

use crate::i18n::Lang;

use super::help::markdown::{HelpMarkdownDocument, HelpMarkdownHighlights, parse_help_markdown};

const CHANGELOG_SOURCE_RU: &str = include_str!("../../CHANGELOG.md");
const CHANGELOG_SOURCE_EN: &str = include_str!("../../CHANGELOG-EN.md");

pub(crate) struct ChangelogRelease {
    pub(crate) version: String,
    pub(crate) date: String,
    markdown: String,
}

pub(crate) struct ChangelogDialog {
    pub(crate) selected: usize,
    pub(crate) releases: Vec<ChangelogRelease>,
    pub(crate) article_content: text_editor::Content,
    pub(crate) article_highlights: HelpMarkdownHighlights,
    source: &'static str,
}

impl ChangelogDialog {
    pub(crate) fn new(lang: Lang) -> Self {
        let source = changelog_source(lang);
        let document = changelog_document(source);
        Self {
            selected: 0,
            releases: parse_releases(source),
            article_content: text_editor::Content::with_text(&document.text),
            article_highlights: document.highlights,
            source,
        }
    }

    pub(crate) fn select_release(&mut self, selected: usize) {
        if selected > self.releases.len() || self.selected == selected {
            return;
        }
        let markdown = if selected == 0 {
            self.source
        } else {
            &self.releases[selected - 1].markdown
        };
        let document = changelog_document(markdown);
        self.selected = selected;
        self.article_content = text_editor::Content::with_text(&document.text);
        self.article_highlights = document.highlights;
    }

    pub(crate) fn perform_text_action(&mut self, action: text_editor::Action) {
        super::read_only_text::perform_action(&mut self.article_content, action);
    }
}

fn changelog_source(lang: Lang) -> &'static str {
    match lang {
        Lang::Ru => CHANGELOG_SOURCE_RU,
        Lang::En => CHANGELOG_SOURCE_EN,
    }
}

fn parse_releases(source: &str) -> Vec<ChangelogRelease> {
    let mut releases = Vec::new();
    let mut current: Option<ChangelogRelease> = None;

    for line in source.lines() {
        if let Some((version, date)) = release_heading(line) {
            if let Some(release) = current.take() {
                releases.push(finish_release(release));
            }
            current = Some(ChangelogRelease {
                version: version.to_owned(),
                date: date.to_owned(),
                markdown: format!("{line}\n"),
            });
        } else if let Some(release) = current.as_mut() {
            release.markdown.push_str(line);
            release.markdown.push('\n');
        }
    }

    if let Some(release) = current {
        releases.push(finish_release(release));
    }
    releases
}

fn finish_release(mut release: ChangelogRelease) -> ChangelogRelease {
    release.markdown.truncate(release.markdown.trim_end().len());
    release
}

fn release_heading(line: &str) -> Option<(&str, &str)> {
    let heading = line.strip_prefix("## [")?;
    heading.split_once("] - ")
}

fn changelog_document(raw: &str) -> HelpMarkdownDocument {
    parse_help_markdown(&readable_changelog(raw))
}

fn readable_changelog(raw: &str) -> String {
    raw.lines()
        .map(|line| {
            let line = line.trim();
            if let Some((version, date)) = release_heading(line) {
                format!("**{version} · {date}**")
            } else if let Some(heading) = line.strip_prefix("### ") {
                format!("**{heading}**")
            } else if let Some(heading) = line.strip_prefix("# ") {
                format!("**{heading}**")
            } else if let Some(item) = line.strip_prefix("- ") {
                format!("• {item}")
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_index_follows_changelog_order() {
        let releases = parse_releases(CHANGELOG_SOURCE_EN);

        assert_eq!(releases[0].version, "1.1.0");
        assert_eq!(releases[0].date, "2026-07-21");
        assert_eq!(releases[1].version, "1.0.0");
    }

    #[test]
    fn selecting_release_limits_reader_to_that_release() {
        let mut dialog = ChangelogDialog::new(Lang::En);

        dialog.select_release(1);

        let text = dialog.article_content.text();
        assert!(text.contains("1.1.0 · 2026-07-21"));
        assert!(!text.contains("1.0.0 · 2026-06-23"));
    }

    #[test]
    fn reader_formats_markdown_without_editing_source() {
        let document = changelog_document("## [2.0.0] - 2026-08-01\n### Added\n- Fast");

        assert_eq!(document.text, "2.0.0 · 2026-08-01\nAdded\n• Fast");
    }

    #[test]
    fn text_actions_select_without_editing() {
        let mut dialog = ChangelogDialog::new(Lang::En);
        let before = dialog.article_content.text();

        dialog.perform_text_action(text_editor::Action::SelectAll);
        dialog.perform_text_action(text_editor::Action::Edit(text_editor::Edit::Delete));

        assert_eq!(dialog.article_content.text(), before);
        assert_eq!(dialog.article_content.selection(), Some(before));
    }

    #[test]
    fn localized_sources_keep_the_same_release_index() {
        let english = parse_releases(CHANGELOG_SOURCE_EN);
        let russian = parse_releases(CHANGELOG_SOURCE_RU);
        let english_index = english
            .iter()
            .map(|release| (&release.version, &release.date))
            .collect::<Vec<_>>();
        let russian_index = russian
            .iter()
            .map(|release| (&release.version, &release.date))
            .collect::<Vec<_>>();

        assert_eq!(russian_index, english_index);
    }

    #[test]
    fn russian_language_uses_the_localized_changelog() {
        let dialog = ChangelogDialog::new(Lang::Ru);
        let text = dialog.article_content.text();

        assert!(text.contains("История изменений"));
        assert!(text.contains("Новые возможности"));
        assert!(!text.contains("Bug Fixes"));
    }
}
