use iced::widget::text_editor;

pub(super) fn perform_action(content: &mut text_editor::Content, action: text_editor::Action) {
    match action {
        text_editor::Action::Click(point) => {
            content.perform(text_editor::Action::Click(point));
            suppress_caret(content);
        }
        text_editor::Action::Drag(point) => {
            content.perform(text_editor::Action::Drag(point));
            suppress_empty_caret(content);
        }
        text_editor::Action::Edit(_) | text_editor::Action::Move(_) => {}
        action => content.perform(action),
    }
}

fn suppress_empty_caret(content: &mut text_editor::Content) {
    match content.selection() {
        Some(selection) if !selection.is_empty() => {}
        _ => suppress_caret(content),
    }
}

fn suppress_caret(content: &mut text_editor::Content) {
    let cursor = content.cursor();
    content.move_to(text_editor::Cursor {
        position: cursor.position,
        selection: Some(cursor.position),
    });
}
