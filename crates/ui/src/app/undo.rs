//! Undo/redo stack: text edits and CPU snapshot pairs share one timeline.
//! Consecutive `Text` entries on the same field collapse – the chain
//! breaks on focus change, Enter, Esc, or any CPU push.

use k580_core::{Cpu8080State, RegisterName};

#[cfg(test)]
mod tests;

/// 256 entries × ~64 KiB CPU state ≈ 16 MiB worst case.
const UNDO_DEPTH_LIMIT: usize = 256;

#[derive(Clone, Debug)]
pub(crate) enum UndoEntry {
    Text {
        field: &'static str,
        before: String,
        after: String,
    },
    /// `register_selection` rewinds the register editor's active
    /// cell on Ctrl+Z – without it, undo of "edit A → Enter → step
    /// to B" puts the byte back into A while the visible name field
    /// still shows B.
    Cpu {
        before: Box<Cpu8080State>,
        after: Box<Cpu8080State>,
        register_selection: Option<(RegisterName, RegisterName)>,
    },
}

#[derive(Default, Debug)]
pub(crate) struct UndoStack {
    pub(crate) undo: Vec<UndoEntry>,
    pub(crate) redo: Vec<UndoEntry>,
    coalesce_field: Option<&'static str>,
}

impl UndoStack {
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

    pub(crate) fn push_cpu(&mut self, before: Cpu8080State, after: Cpu8080State) {
        self.push_cpu_with_register_selection(before, after, None);
    }

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

    pub(crate) fn break_coalescing(&mut self) {
        self.coalesce_field = None;
    }

    pub(crate) fn pop_undo(&mut self) -> Option<UndoEntry> {
        let entry = self.undo.pop()?;
        self.coalesce_field = None;
        self.redo.push(entry.clone());
        Some(entry)
    }

    pub(crate) fn pop_redo(&mut self) -> Option<UndoEntry> {
        let entry = self.redo.pop()?;
        self.coalesce_field = None;
        self.undo.push(entry.clone());
        Some(entry)
    }

    pub(crate) fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
        self.coalesce_field = None;
    }

    fn push_entry(&mut self, entry: UndoEntry) {
        self.undo.push(entry);
        if self.undo.len() > UNDO_DEPTH_LIMIT {
            self.undo.remove(0);
        }
    }
}
