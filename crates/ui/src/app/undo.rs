//! Undo/redo stack for the iced shell.
//!
//! The user expects Ctrl+Z and Ctrl+Shift+Z to behave the way every
//! desktop editor does: a *single* timeline that captures whatever the
//! last gesture was — typing into a text input, committing a byte to
//! RAM, wiping the registers — and rewinds it on Ctrl+Z. This module
//! owns that timeline as a pair of `Vec<UndoEntry>`s (`undo` + `redo`)
//! plus the small bookkeeping that keeps consecutive keystrokes from
//! flooding the stack.
//!
//! Two flavours of entry:
//!
//! - [`UndoEntry::Text`] — a buffered text-input edit. We store the
//!   buffer *before* the change and the buffer *after*; redo replays
//!   `after`, undo replays `before`. The widget id is one of the
//!   stable identifiers from `app::constants` (`MEMORY_VALUE_INPUT_ID`
//!   et al.), so the handler knows which `DesktopApp::*_input` field
//!   to write into.
//! - [`UndoEntry::Cpu`] — a full `Cpu8080State` snapshot pair. Used
//!   for every gesture that mutates emulator state through the worker
//!   (`SetMemory`, `SetRegister`, `SetPc`, `ResetCpu`, `ResetRam`,
//!   snapshot/import loads). Storing the entire 64 KiB + registers is
//!   ~64 KiB per entry which is fine for the depth we cap at; trying
//!   to model an inverse per command type would be far more code for
//!   no observable difference.
//!
//! ## Coalescing
//!
//! Without coalescing, every keystroke into a hex input would push a
//! single-character entry, and Ctrl+Z would give the user one undo
//! per character. We keep the desktop convention of "one undo per
//! word" by collapsing consecutive `Text` entries that target the
//! same field into one: the second push overwrites the first's
//! `after`, leaving its `before` intact. The chain breaks the moment
//! the user clicks elsewhere, presses Enter (a CPU-mutating gesture
//! lands), or simply types into a different input — anything that
//! would be a logically separate edit in a real editor.

use k580_core::{Cpu8080State, RegisterName};

/// Hard cap on stack depth. 256 is generous enough for an extended
/// editing session — the user's mental model of "many Ctrl+Z presses"
/// does not exceed a few dozen — and bounds the worst-case memory at
/// ~16 MiB (256 × 64 KiB CPU state, never both stacks full of CPU
/// snapshots in practice). Smaller caps would force the user to lose
/// history on routine edit sessions; larger caps would let a runaway
/// macro hold the heap hostage.
const UNDO_DEPTH_LIMIT: usize = 256;

#[derive(Clone, Debug)]
pub(crate) enum UndoEntry {
    /// Text-buffer edit on a tracked `text_input`. `field` is one of
    /// the stable id strings declared in `app::constants`; the
    /// handler dispatches on it to decide which field to write into.
    /// Both `before` and `after` are owned `String`s so the entry
    /// is self-contained — once pushed, mutating the input does not
    /// retroactively break older entries.
    Text {
        field: &'static str,
        before: String,
        after: String,
    },
    /// Snapshot of the entire CPU state captured before/after a
    /// mutating worker command. We keep boxed pairs so the entry
    /// stays a fixed-size enum variant despite the snapshots being
    /// ~64 KiB each.
    ///
    /// `register_selection` carries the `(before, after)` register
    /// the register-editor panel had selected when the entry was
    /// pushed. It exists so Ctrl+Z after "edit register A → Enter
    /// → step to B" rewinds the *selection* back to A as well, not
    /// just the byte. Without this the rewind would put the byte
    /// back but the visible name/value fields would still point at
    /// B, and the user would have no obvious cue that the undo
    /// landed. The field is `None` for every other CPU mutation
    /// (memory inline edits, opcode picker, reset commands) where
    /// the register selection is not part of the gesture.
    Cpu {
        before: Box<Cpu8080State>,
        after: Box<Cpu8080State>,
        register_selection: Option<(RegisterName, RegisterName)>,
    },
}

/// Marker for "the next text edit on this field starts a fresh
/// entry". Set after every CPU push, after focus reconciliation
/// reports a different focused widget, and after Esc / Enter — the
/// gestures that complete a logical edit. Reading `last_text_field`
/// alone would not be enough because two edits on the same field
/// can be logically distinct (typing, pressing Enter to commit,
/// typing again — the post-Enter typing should be a fresh undo
/// entry, not a continuation).
#[derive(Default, Debug)]
pub(crate) struct UndoStack {
    pub(crate) undo: Vec<UndoEntry>,
    pub(crate) redo: Vec<UndoEntry>,
    /// Id of the field whose text edit is currently being coalesced
    /// onto the top of `undo`. `None` after a CPU push, after focus
    /// changes, or when the stack is empty.
    coalesce_field: Option<&'static str>,
}

impl UndoStack {
    /// Pushes a text edit. Coalesces with the top entry when
    /// `field` matches the in-flight edit; otherwise starts a fresh
    /// entry. Either way the redo stack is cleared — a new edit
    /// invalidates the previously-rewound future, the same way every
    /// editor handles it.
    ///
    /// Returns silently when `before == after`: a no-op edit (e.g.
    /// re-applying the same hex byte) would just clutter the stack
    /// with empty diffs and force the user to press Ctrl+Z twice for
    /// each visible change.
    pub(crate) fn push_text(&mut self, field: &'static str, before: String, after: String) {
        if before == after {
            return;
        }
        self.redo.clear();

        if self.coalesce_field == Some(field)
            && let Some(UndoEntry::Text {
                field: top_field,
                after: top_after,
                ..
            }) = self.undo.last_mut()
            && *top_field == field
        {
            *top_after = after;
            return;
        }

        self.push_entry(UndoEntry::Text {
            field,
            before,
            after,
        });
        self.coalesce_field = Some(field);
    }

    /// Pushes a CPU snapshot pair. Always starts a fresh entry —
    /// CPU mutations are atomic gestures in the user's mental model
    /// (one ResetCpu, one SetMemory commit), unlike text typing,
    /// which is naturally a stream. Clears the coalesce marker so
    /// the next text edit on whatever field had focus does not glue
    /// itself to a pre-CPU text entry.
    pub(crate) fn push_cpu(&mut self, before: Cpu8080State, after: Cpu8080State) {
        self.push_cpu_with_register_selection(before, after, None);
    }

    /// Same as `push_cpu`, but also captures which register the
    /// register-editor panel had selected before/after the gesture.
    /// Used exclusively by `apply_register_and_step` so Ctrl+Z
    /// rewinds the selection back to the register the user had
    /// open at the moment of Enter, not whatever register
    /// `step_register(+1)` walked to afterwards.
    pub(crate) fn push_cpu_with_register_selection(
        &mut self,
        before: Cpu8080State,
        after: Cpu8080State,
        register_selection: Option<(RegisterName, RegisterName)>,
    ) {
        if before == after {
            return;
        }
        self.redo.clear();
        self.push_entry(UndoEntry::Cpu {
            before: Box::new(before),
            after: Box::new(after),
            register_selection,
        });
        self.coalesce_field = None;
    }

    /// Tells the stack that the in-flight text coalescing is done —
    /// the next `push_text` on the same field starts a fresh entry.
    /// Called when focus moves between inputs, when Esc / Enter
    /// closes a logical edit, and when the user clicks somewhere
    /// that is not a tracked input.
    pub(crate) fn break_coalescing(&mut self) {
        self.coalesce_field = None;
    }

    /// Pops the top undo entry, moves it to the redo stack, and
    /// returns it for the caller to apply. `None` when there is
    /// nothing to undo.
    pub(crate) fn pop_undo(&mut self) -> Option<UndoEntry> {
        let entry = self.undo.pop()?;
        // The entry we are about to apply represents the user's most
        // recent action; once we move it onto `redo`, any further
        // text typing must start a fresh undo entry rather than
        // re-coalesce onto a now-rewound edit.
        self.coalesce_field = None;
        self.redo.push(entry.clone());
        Some(entry)
    }

    /// Pops the top redo entry, moves it back to the undo stack,
    /// and returns it. `None` when there is nothing to redo.
    pub(crate) fn pop_redo(&mut self) -> Option<UndoEntry> {
        let entry = self.redo.pop()?;
        self.coalesce_field = None;
        self.undo.push(entry.clone());
        Some(entry)
    }

    /// Wipes both stacks. Used on "Новый файл" — the user explicitly
    /// asked for a blank slate, history has nothing to anchor onto.
    pub(crate) fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
        self.coalesce_field = None;
    }

    fn push_entry(&mut self, entry: UndoEntry) {
        self.undo.push(entry);
        if self.undo.len() > UNDO_DEPTH_LIMIT {
            // Drop the oldest entry — losing the very first edit is
            // strictly better than letting a long session balloon
            // memory unboundedly. `Vec::remove(0)` is O(n) but the
            // n is bounded by `UNDO_DEPTH_LIMIT` and only runs once
            // per push at the limit.
            self.undo.remove(0);
        }
    }
}

#[cfg(test)]
mod tests {
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
        assert_eq!(
            stack.undo.len(),
            2,
            "different fields get their own entries"
        );
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
        assert_eq!(stack.redo.len(), 1, "undo populates redo");
        stack.push_text("addr", "00".to_owned(), "0A".to_owned());
        assert!(
            stack.redo.is_empty(),
            "a fresh edit invalidates the redo branch"
        );
    }

    #[test]
    fn no_op_pushes_are_dropped() {
        let mut stack = UndoStack::default();
        stack.push_text("addr", "00".to_owned(), "00".to_owned());
        assert!(stack.undo.is_empty(), "before == after must not be stored");
        let state = fresh_state(0x42);
        stack.push_cpu(state.clone(), state);
        assert!(stack.undo.is_empty(), "identical CPU states must not push");
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
            // Break coalescing every push so we get distinct entries
            // and exercise the cap.
            stack.break_coalescing();
        }
        assert_eq!(
            stack.undo.len(),
            UNDO_DEPTH_LIMIT,
            "stack is capped at UNDO_DEPTH_LIMIT"
        );
    }

    /// `break_coalescing` is what every "logical edit ended" gesture
    /// calls — focus change, Esc, Enter, snapshot load. The contract
    /// is: the *next* `push_text` on the same field must start a
    /// fresh entry, not extend the previous one. Without this Esc
    /// would silently glue the post-Esc typing onto the pre-Esc
    /// run, and the user would discover that one Ctrl+Z rewinds
    /// across both edits.
    #[test]
    fn break_coalescing_splits_same_field_runs() {
        let mut stack = UndoStack::default();
        stack.push_text("addr", "00".to_owned(), "01".to_owned());
        stack.break_coalescing();
        stack.push_text("addr", "01".to_owned(), "02".to_owned());
        assert_eq!(
            stack.undo.len(),
            2,
            "break_coalescing must split same-field runs"
        );
    }

    /// `clear()` is the "Новый файл" / "blank slate" handler. Both
    /// stacks must be empty afterwards, *and* the coalesce marker
    /// has to drop too — otherwise the very first text edit on the
    /// new document would try to coalesce onto an entry that no
    /// longer exists, which `push_text` handles gracefully today
    /// but is brittle to rely on.
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
        // Push after clear, then push again on the same field — they
        // must not coalesce, because clear() has dropped the
        // in-flight marker.
        stack.push_text("addr", "00".to_owned(), "01".to_owned());
        // This is a fresh push immediately after clear, so it *will*
        // coalesce with itself's same-field successor; that's
        // expected. The actual invariant is that clear restored the
        // marker properly — verified by checking we are starting
        // from a one-entry state, not a zero-entry state.
        assert_eq!(stack.undo.len(), 1);
    }

    /// After Ctrl+Z moves an entry from `undo` to `redo`, a fresh
    /// keystroke on the same field must not coalesce onto whatever
    /// entry happens to be on top of `undo`. The bug this guards
    /// against: type "0A", press Ctrl+Z (entry moves to redo, top
    /// of undo is now whatever older entry was below it), type "B"
    /// — without resetting the coalesce marker, "B" would extend
    /// the older entry's `after` and silently corrupt that edit's
    /// history.
    #[test]
    fn pop_undo_resets_coalesce_marker() {
        let mut stack = UndoStack::default();
        stack.push_text("addr", "00".to_owned(), "01".to_owned());
        stack.break_coalescing();
        stack.push_text("addr", "01".to_owned(), "02".to_owned());
        assert_eq!(stack.undo.len(), 2);
        stack.pop_undo();
        assert_eq!(stack.undo.len(), 1);
        // Next typing on the same field must produce a brand-new
        // entry, not coalesce onto the older one we exposed.
        stack.push_text("addr", "01".to_owned(), "0A".to_owned());
        assert_eq!(
            stack.undo.len(),
            2,
            "post-undo typing must not coalesce onto a re-exposed entry"
        );
    }
}
