use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

use iced::widget::text_editor;

use super::markdown::{HelpMarkdownDocument, parse_help_markdown};
use super::search::{HelpSearchIndex, HelpSearchRequest, HelpSearchResponse};
use super::{HelpDialog, HelpNode};
use crate::i18n::Lang;

const HELP_SEARCH_DEBOUNCE: Duration = Duration::from_millis(140);

#[derive(Clone)]
pub(super) struct PendingHelpSearch {
    generation: u64,
    due_at: Instant,
}

impl HelpDialog {
    pub(crate) fn new(lang: Lang) -> Self {
        let article = article_document(HelpNode::TopicAbout, lang);
        Self {
            selected: HelpNode::TopicAbout,
            expanded: BTreeSet::new(),
            search: String::new(),
            article_content: text_editor::Content::with_text(&article.text),
            article_highlights: article.highlights,
            article_content_node: HelpNode::TopicAbout,
            article_content_lang: lang,
            search_index: Arc::new(HelpSearchIndex::new(lang)),
            search_generation: 0,
            pending_search: None,
            search_results_query: String::new(),
            search_matches: BTreeSet::new(),
            search_results: Vec::new(),
        }
    }

    pub(crate) fn select_node(&mut self, node: HelpNode, lang: Lang) {
        self.selected = node;
        self.reveal_node(node);
        self.sync_article_content(node, lang);
    }

    pub(crate) fn toggle_expanded(&mut self, node: HelpNode, lang: Lang) {
        if !node.is_category() {
            self.select_node(node, lang);
            return;
        }
        if self.expanded.contains(&node) {
            self.expanded.remove(&node);
        } else {
            self.expanded.insert(node);
        }
    }

    pub(crate) fn expand_all(&mut self) {
        let mut all = BTreeSet::new();
        for node in HelpNode::ROOTS {
            collect_categories(node, &mut all);
        }
        self.expanded = all;
    }

    pub(crate) fn collapse_all(&mut self) {
        self.expanded.clear();
    }

    pub(crate) fn all_expanded(&self) -> bool {
        let mut all_cats = BTreeSet::new();
        for node in HelpNode::ROOTS {
            collect_categories(node, &mut all_cats);
        }
        self.expanded == all_cats
    }

    pub(crate) fn update_search_input(&mut self, query: String, lang: Lang) {
        self.search = query;
        self.search_generation = self.search_generation.wrapping_add(1);

        if self.search.trim().is_empty() {
            self.clear_search_results();
            return;
        }

        if self.search_index.lang() != lang {
            self.search_index = Arc::new(HelpSearchIndex::new(lang));
        }
        self.pending_search = Some(PendingHelpSearch {
            generation: self.search_generation,
            due_at: Instant::now() + HELP_SEARCH_DEBOUNCE,
        });
    }

    pub(crate) fn results_query(&self) -> &str {
        &self.search_results_query
    }

    pub(crate) fn search_results(&self) -> &[super::HelpSearchResult] {
        &self.search_results
    }

    pub(crate) fn node_matches_search(&self, node: HelpNode, _lang: Lang) -> bool {
        if self.results_query().is_empty() {
            return true;
        }
        self.search_matches.contains(&node)
    }

    pub(crate) fn take_due_search_request(
        &mut self,
        lang: Lang,
        now: Instant,
    ) -> Option<HelpSearchRequest> {
        let pending = self.pending_search.as_ref()?;
        if now < pending.due_at {
            return None;
        }

        let generation = pending.generation;
        let query = self.search.trim().to_owned();
        self.pending_search = None;

        if query.is_empty() {
            self.clear_search_results();
            return None;
        }
        if self.search_index.lang() != lang {
            self.search_index = Arc::new(HelpSearchIndex::new(lang));
        }
        Some(HelpSearchRequest::new(
            generation,
            lang,
            query,
            self.search_index.clone(),
        ))
    }

    pub(crate) fn apply_search_response(&mut self, response: HelpSearchResponse, lang: Lang) {
        if response.lang() != lang
            || response.generation() != self.search_generation
            || response.query() != self.search.trim()
        {
            return;
        }

        self.search_results_query = response.query().to_owned();
        self.search_matches.clear();
        self.search_results = response.into_results();
        let nodes = self
            .search_results
            .iter()
            .map(|result| result.node())
            .collect::<Vec<_>>();
        for node in nodes {
            self.collect_search_path(node);
        }

        if let Some(first) = self.search_results.first().map(|result| result.node()) {
            self.select_node(first, lang);
        }
    }

    fn reveal_node(&mut self, node: HelpNode) {
        for root in HelpNode::ROOTS {
            let mut path = Vec::new();
            if collect_path(root, node, &mut path) {
                for ancestor in path {
                    if ancestor != node && ancestor.is_category() {
                        self.expanded.insert(ancestor);
                    }
                }
                return;
            }
        }
    }

    fn sync_article_content(&mut self, node: HelpNode, lang: Lang) {
        if self.article_content_node == node && self.article_content_lang == lang {
            return;
        }
        let article = article_document(node, lang);
        self.article_content = text_editor::Content::with_text(&article.text);
        self.article_highlights = article.highlights;
        self.article_content_node = node;
        self.article_content_lang = lang;
    }

    pub(crate) fn perform_text_action(&mut self, action: text_editor::Action) {
        match action {
            text_editor::Action::Click(point) => {
                self.article_content
                    .perform(text_editor::Action::Click(point));
                self.suppress_article_caret();
            }
            text_editor::Action::Drag(point) => {
                self.article_content
                    .perform(text_editor::Action::Drag(point));
                self.suppress_empty_article_caret();
            }
            text_editor::Action::Edit(_) | text_editor::Action::Move(_) => {}
            action => self.article_content.perform(action),
        }
    }

    fn suppress_empty_article_caret(&mut self) {
        match self.article_content.selection() {
            Some(selection) if !selection.is_empty() => {}
            _ => self.suppress_article_caret(),
        }
    }

    fn suppress_article_caret(&mut self) {
        let cursor = self.article_content.cursor();
        self.article_content.move_to(text_editor::Cursor {
            position: cursor.position,
            selection: Some(cursor.position),
        });
    }

    fn clear_search_results(&mut self) {
        self.pending_search = None;
        self.search_results_query.clear();
        self.search_matches.clear();
        self.search_results.clear();
    }

    fn collect_search_path(&mut self, node: HelpNode) {
        for root in HelpNode::ROOTS {
            let mut path = Vec::new();
            if collect_path(root, node, &mut path) {
                self.search_matches.extend(path);
                return;
            }
        }
    }
}

fn collect_categories(node: HelpNode, expanded: &mut BTreeSet<HelpNode>) {
    if node.is_category() {
        expanded.insert(node);
        for child in node.children() {
            collect_categories(*child, expanded);
        }
    }
}

fn collect_path(current: HelpNode, target: HelpNode, path: &mut Vec<HelpNode>) -> bool {
    path.push(current);
    if current == target {
        return true;
    }
    for child in current.children() {
        if collect_path(*child, target, path) {
            return true;
        }
    }
    path.pop();
    false
}

fn article_document(node: HelpNode, lang: Lang) -> HelpMarkdownDocument {
    parse_help_markdown(lang.t(node.content_key()))
}
