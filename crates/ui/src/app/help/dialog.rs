use std::collections::BTreeSet;

use iced::widget::text_editor;

use super::markdown::{HelpMarkdownDocument, parse_help_markdown};
use super::search::HelpSearchIndex;
use super::{HelpDialog, HelpNode};
use crate::i18n::Lang;

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
            search_index: HelpSearchIndex::new(lang),
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
        self.rebuild_search_matches(lang);

        if self.results_query().is_empty() {
            return;
        }
        if let Some(first) = self.search_results.first().map(|result| result.node()) {
            self.select_node(first, lang);
        }
    }

    pub(crate) fn results_query(&self) -> &str {
        self.search.trim()
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

    pub(crate) fn perform_text_action(&mut self, action: text_editor::Action) {
        if !action.is_edit() {
            self.article_content.perform(action);
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

    fn rebuild_search_matches(&mut self, lang: Lang) {
        self.search_matches.clear();
        self.search_results.clear();
        if self.search_index.lang() != lang {
            self.search_index = HelpSearchIndex::new(lang);
        }

        let query = self.results_query().to_owned();
        if query.trim().is_empty() {
            return;
        }
        self.search_results = self.search_index.search(&query);
        let nodes = self
            .search_results
            .iter()
            .map(|result| result.node())
            .collect::<Vec<_>>();
        for node in nodes {
            self.collect_search_path(node);
        }
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
