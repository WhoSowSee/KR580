use std::collections::BTreeSet;
use std::sync::Arc;

use super::HelpNode;
use crate::i18n::Lang;

const MAX_PREVIEW_LINES: usize = 4;

#[derive(Clone)]
pub(crate) struct HelpSearchIndex {
    lang: Lang,
    records: Vec<HelpSearchRecord>,
}

#[derive(Clone)]
struct HelpSearchRecord {
    node: HelpNode,
    haystack: String,
    raw_lines: Vec<String>,
    lower_lines: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HelpSearchResult {
    node: HelpNode,
    preview_lines: Vec<String>,
}

#[derive(Clone)]
pub(crate) struct HelpSearchRequest {
    generation: u64,
    lang: Lang,
    query: String,
    index: Arc<HelpSearchIndex>,
}

#[derive(Clone, Debug)]
pub(crate) struct HelpSearchResponse {
    generation: u64,
    lang: Lang,
    query: String,
    results: Vec<HelpSearchResult>,
}

impl HelpSearchIndex {
    pub(crate) fn new(lang: Lang) -> Self {
        let mut records = Vec::new();
        let mut seen = BTreeSet::new();
        for node in HelpNode::ROOTS {
            collect_records(node, lang, &mut Vec::new(), &mut seen, &mut records);
        }
        Self { lang, records }
    }

    pub(crate) fn lang(&self) -> Lang {
        self.lang
    }

    pub(crate) fn search(&self, query: &str) -> Vec<HelpSearchResult> {
        let needle = query.trim().to_lowercase();
        if needle.is_empty() {
            return Vec::new();
        }

        self.records
            .iter()
            .filter(|record| record.haystack.contains(&needle))
            .map(|record| HelpSearchResult {
                node: record.node,
                preview_lines: record.preview_lines(&needle),
            })
            .collect()
    }
}

impl HelpSearchRequest {
    pub(crate) fn new(
        generation: u64,
        lang: Lang,
        query: String,
        index: Arc<HelpSearchIndex>,
    ) -> Self {
        Self {
            generation,
            lang,
            query,
            index,
        }
    }

    pub(crate) fn resolve(self) -> HelpSearchResponse {
        let results = self.index.search(&self.query);
        HelpSearchResponse {
            generation: self.generation,
            lang: self.lang,
            query: self.query,
            results,
        }
    }
}

impl HelpSearchResponse {
    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }

    pub(crate) fn lang(&self) -> Lang {
        self.lang
    }

    pub(crate) fn query(&self) -> &str {
        &self.query
    }

    pub(crate) fn into_results(self) -> Vec<HelpSearchResult> {
        self.results
    }
}

pub(crate) async fn run_help_search(request: HelpSearchRequest) -> HelpSearchResponse {
    request.resolve()
}

impl HelpSearchRecord {
    fn preview_lines(&self, needle: &str) -> Vec<String> {
        let mut lines = self
            .raw_lines
            .iter()
            .zip(&self.lower_lines)
            .filter(|(_, lower)| lower.contains(needle))
            .map(|(raw, _)| raw.clone())
            .take(MAX_PREVIEW_LINES)
            .collect::<Vec<_>>();

        if lines.is_empty() {
            lines = self
                .raw_lines
                .iter()
                .take(MAX_PREVIEW_LINES)
                .cloned()
                .collect();
        }
        lines
    }
}

impl HelpSearchResult {
    pub(crate) fn node(&self) -> HelpNode {
        self.node
    }

    pub(crate) fn preview_lines(&self) -> &[String] {
        &self.preview_lines
    }
}

fn collect_records(
    node: HelpNode,
    lang: Lang,
    path: &mut Vec<&'static str>,
    seen: &mut BTreeSet<HelpNode>,
    records: &mut Vec<HelpSearchRecord>,
) {
    path.push(lang.t(node.label_key()));

    if node.is_category() {
        for child in node.children() {
            collect_records(*child, lang, path, seen, records);
        }
    } else if seen.insert(node) {
        records.push(record_for(node, lang, path));
    }

    path.pop();
}

fn record_for(node: HelpNode, lang: Lang, path: &[&str]) -> HelpSearchRecord {
    let content = lang.t(node.content_key());
    let raw_lines = content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let lower_lines = raw_lines
        .iter()
        .map(|line| line.to_lowercase())
        .collect::<Vec<_>>();
    let mut haystack = path.join(" ").to_lowercase();
    haystack.push('\n');
    haystack.push_str(&content.to_lowercase());

    HelpSearchRecord {
        node,
        haystack,
        raw_lines,
        lower_lines,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_labels_match_child_topics_through_breadcrumbs() {
        let index = HelpSearchIndex::new(Lang::Ru);

        let results = index.search("главное");

        assert!(
            results
                .iter()
                .any(|result| result.node == HelpNode::TopicMenuView)
        );
    }

    #[test]
    fn previews_are_bounded_to_four_lines() {
        let index = HelpSearchIndex::new(Lang::Ru);

        let results = index.search("команда");

        assert!(results.iter().all(|result| result.preview_lines.len() <= 4));
    }
}
