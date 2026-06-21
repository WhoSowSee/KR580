use super::{HelpDialog, HelpNode};
use crate::i18n::Lang;
use iced::Point;
use iced::widget::text_editor;
use std::time::{Duration, Instant};

#[test]
fn search_input_updates_text_before_results() {
    let mut dialog = HelpDialog::new(Lang::Ru);

    dialog.update_search_input("эмулятор".to_owned(), Lang::Ru);

    assert_eq!(dialog.search, "эмулятор");
    assert_eq!(dialog.results_query(), "");
    assert!(dialog.search_results().is_empty());
    assert!(dialog.node_matches_search(HelpNode::TopicAbout, Lang::Ru));
}

#[test]
fn clearing_search_clears_cached_results() {
    let mut dialog = HelpDialog::new(Lang::Ru);

    dialog.update_search_input("эмулятор".to_owned(), Lang::Ru);
    apply_due_search(&mut dialog, Lang::Ru);
    dialog.update_search_input(String::new(), Lang::Ru);

    assert_eq!(dialog.results_query(), "");
    assert!(dialog.node_matches_search(HelpNode::TopicAbout, Lang::Ru));
}

#[test]
fn search_results_use_bounded_previews() {
    let mut dialog = HelpDialog::new(Lang::Ru);

    dialog.update_search_input("команда".to_owned(), Lang::Ru);
    apply_due_search(&mut dialog, Lang::Ru);

    assert!(!dialog.search_results().is_empty());
    assert!(
        dialog
            .search_results()
            .iter()
            .all(|result| result.preview_lines().len() <= 4)
    );
}

#[test]
fn search_result_preview_keeps_matching_line_without_full_article() {
    let mut dialog = HelpDialog::new(Lang::Ru);

    dialog.update_search_input("NOP".to_owned(), Lang::Ru);
    apply_due_search(&mut dialog, Lang::Ru);
    let result = dialog
        .search_results()
        .iter()
        .find(|result| result.node() == HelpNode::TopicProcessorControlCommands)
        .expect("processor control commands should match NOP");

    let preview = result.preview_lines().join("\n");
    assert!(preview.contains("NOP"));
    assert!(result.preview_lines().len() < 6);
}

#[test]
fn article_click_keeps_empty_selection_anchor() {
    let mut dialog = HelpDialog::new(Lang::Ru);

    dialog.perform_text_action(text_editor::Action::Click(Point::new(32.0, 20.0)));
    let cursor = dialog.article_content.cursor();

    assert_eq!(cursor.selection, Some(cursor.position));
    assert_eq!(dialog.article_content.selection(), Some(String::new()));
}

#[test]
fn article_noop_drag_after_click_keeps_empty_selection_anchor() {
    let mut dialog = HelpDialog::new(Lang::Ru);
    let point = Point::new(32.0, 20.0);

    dialog.perform_text_action(text_editor::Action::Click(point));
    dialog.perform_text_action(text_editor::Action::Drag(point));
    let cursor = dialog.article_content.cursor();

    assert_eq!(cursor.selection, Some(cursor.position));
    assert_eq!(dialog.article_content.selection(), Some(String::new()));
}

#[test]
fn article_text_actions_select_without_editing() {
    let mut dialog = HelpDialog::new(Lang::Ru);
    let before = dialog.article_content.text();

    dialog.perform_text_action(text_editor::Action::SelectAll);
    dialog.perform_text_action(text_editor::Action::Edit(text_editor::Edit::Delete));

    assert_eq!(dialog.article_content.text(), before);
    assert_eq!(dialog.article_content.selection(), Some(before));
}

#[test]
fn search_request_waits_for_debounce_deadline() {
    let mut dialog = HelpDialog::new(Lang::Ru);

    dialog.update_search_input("команда".to_owned(), Lang::Ru);

    assert!(
        dialog
            .take_due_search_request(Lang::Ru, Instant::now())
            .is_none()
    );
}

#[test]
fn stale_search_response_does_not_replace_newer_input() {
    let mut dialog = HelpDialog::new(Lang::Ru);

    dialog.update_search_input("эмулятор".to_owned(), Lang::Ru);
    let stale = due_search_response(&mut dialog, Lang::Ru);
    dialog.update_search_input("NOP".to_owned(), Lang::Ru);
    dialog.apply_search_response(stale, Lang::Ru);

    assert_eq!(dialog.search, "NOP");
    assert_eq!(dialog.results_query(), "");

    apply_due_search(&mut dialog, Lang::Ru);

    assert_eq!(dialog.results_query(), "NOP");
    assert!(
        dialog
            .search_results()
            .iter()
            .any(|result| result.node() == HelpNode::TopicProcessorControlCommands)
    );
}

#[test]
fn selecting_subtopic_reveals_parent_category() {
    let mut dialog = HelpDialog::new(Lang::Ru);

    dialog.select_node(HelpNode::TopicMenuHelp, Lang::Ru);

    assert_eq!(dialog.selected, HelpNode::TopicMenuHelp);
    assert!(dialog.expanded.contains(&HelpNode::CatMainMenu));
}

#[test]
fn selecting_subtopic_keeps_existing_expansion() {
    let mut dialog = HelpDialog::new(Lang::Ru);
    dialog.expanded.insert(HelpNode::CatSettings);

    dialog.select_node(HelpNode::TopicExport, Lang::Ru);

    assert!(dialog.expanded.contains(&HelpNode::CatSettings));
    assert!(dialog.expanded.contains(&HelpNode::CatFilesExport));
}

#[test]
fn article_content_renders_inline_bold_without_markers() {
    let mut dialog = HelpDialog::new(Lang::Ru);

    dialog.select_node(HelpNode::TopicProcessorControlCommands, Lang::Ru);

    assert!(!dialog.article_content.text().contains("**"));
    assert!(dialog.article_content.text().contains("NOP (00h)"));
}

fn apply_due_search(dialog: &mut HelpDialog, lang: Lang) {
    let response = due_search_response(dialog, lang);
    dialog.apply_search_response(response, lang);
}

fn due_search_response(dialog: &mut HelpDialog, lang: Lang) -> super::search::HelpSearchResponse {
    dialog
        .take_due_search_request(lang, Instant::now() + Duration::from_secs(1))
        .expect("search request should be due")
        .resolve()
}
