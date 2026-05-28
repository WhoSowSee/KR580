use super::*;

fn fresh_state(byte: u8) -> Cpu8080State {
    let mut state = Cpu8080State::default();
    state.memory.write(0, byte);
    state
}

#[test]
fn text_pushes_coalesce_on_same_field() {
    let mut stack = UndoStack::default();
    stack.push_text("addr", "00".to_owned(), "0A".to_owned());
    stack.push_text("addr", "0A".to_owned(), "0AB".to_owned());
    assert_eq!(stack.undo.len(), 1, "consecutive same-field edits coalesce");
    match &stack.undo[0] {
        UndoEntry::Text {
            field,
            before,
            after,
        } => {
            assert_eq!(*field, "addr");
            assert_eq!(before, "00");
            assert_eq!(after, "0AB");
        }
        other => panic!("unexpected entry kind: {other:?}"),
    }
}

#[test]
fn text_pushes_split_across_fields() {
    let mut stack = UndoStack::default();
    stack.push_text("addr", "00".to_owned(), "01".to_owned());
    stack.push_text("value", "00".to_owned(), "FF".to_owned());
    assert_eq!(stack.undo.len(), 2);
}

#[test]
fn cpu_push_breaks_text_coalescing() {
    let mut stack = UndoStack::default();
    stack.push_text("addr", "00".to_owned(), "01".to_owned());
    stack.push_cpu(fresh_state(0), fresh_state(0xAB));
    stack.push_text("addr", "01".to_owned(), "02".to_owned());
    assert_eq!(stack.undo.len(), 3, "cpu push must break coalescing");
}

#[test]
fn redo_clears_on_new_push() {
    let mut stack = UndoStack::default();
    stack.push_text("addr", "00".to_owned(), "01".to_owned());
    stack.pop_undo();
    assert_eq!(stack.redo.len(), 1);
    stack.push_text("addr", "00".to_owned(), "0A".to_owned());
    assert!(stack.redo.is_empty());
}

#[test]
fn no_op_pushes_are_dropped() {
    let mut stack = UndoStack::default();
    stack.push_text("addr", "00".to_owned(), "00".to_owned());
    assert!(stack.undo.is_empty());
    let state = fresh_state(0x42);
    stack.push_cpu(state.clone(), state);
    assert!(stack.undo.is_empty());
}

#[test]
fn pop_undo_then_redo_round_trips() {
    let mut stack = UndoStack::default();
    stack.push_text("addr", "00".to_owned(), "01".to_owned());
    let undone = stack.pop_undo().expect("undo entry available");
    match undone {
        UndoEntry::Text { before, after, .. } => {
            assert_eq!(before, "00");
            assert_eq!(after, "01");
        }
        other => panic!("unexpected entry: {other:?}"),
    }
    let redone = stack.pop_redo().expect("redo entry available");
    match redone {
        UndoEntry::Text { before, after, .. } => {
            assert_eq!(before, "00");
            assert_eq!(after, "01");
        }
        other => panic!("unexpected entry: {other:?}"),
    }
    assert_eq!(stack.undo.len(), 1);
    assert!(stack.redo.is_empty());
}

#[test]
fn depth_limit_drops_oldest() {
    let mut stack = UndoStack::default();
    for i in 0..(UNDO_DEPTH_LIMIT as u32 + 5) {
        stack.push_text(
            "addr",
            format!("{i:04X}"),
            format!("{:04X}", i.wrapping_add(1)),
        );
        stack.break_coalescing();
    }
    assert_eq!(stack.undo.len(), UNDO_DEPTH_LIMIT);
}

/// `break_coalescing` is what every "logical edit ended" gesture calls —
/// focus change, Esc, Enter, snapshot load. Without it Esc would silently
/// glue post-Esc typing onto the pre-Esc run.
#[test]
fn break_coalescing_splits_same_field_runs() {
    let mut stack = UndoStack::default();
    stack.push_text("addr", "00".to_owned(), "01".to_owned());
    stack.break_coalescing();
    stack.push_text("addr", "01".to_owned(), "02".to_owned());
    assert_eq!(stack.undo.len(), 2);
}

#[test]
fn clear_wipes_both_stacks_and_coalesce_marker() {
    let mut stack = UndoStack::default();
    stack.push_text("addr", "00".to_owned(), "01".to_owned());
    stack.pop_undo();
    assert_eq!(stack.redo.len(), 1);
    stack.push_text("value", "00".to_owned(), "FF".to_owned());
    stack.clear();
    assert!(stack.undo.is_empty());
    assert!(stack.redo.is_empty());
    stack.push_text("addr", "00".to_owned(), "01".to_owned());
    assert_eq!(stack.undo.len(), 1);
}

/// Bug guard: type "0A", Ctrl+Z (entry moves to redo, top of undo is now an
/// older entry), type "B" — without resetting the coalesce marker, "B" would
/// extend the older entry's `after` and silently corrupt that edit.
#[test]
fn pop_undo_resets_coalesce_marker() {
    let mut stack = UndoStack::default();
    stack.push_text("addr", "00".to_owned(), "01".to_owned());
    stack.break_coalescing();
    stack.push_text("addr", "01".to_owned(), "02".to_owned());
    assert_eq!(stack.undo.len(), 2);
    stack.pop_undo();
    assert_eq!(stack.undo.len(), 1);
    stack.push_text("addr", "01".to_owned(), "0A".to_owned());
    assert_eq!(
        stack.undo.len(),
        2,
        "post-undo typing must not coalesce onto a re-exposed entry"
    );
}
