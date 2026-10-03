mod cpu;
#[cfg(test)]
mod tests;

use crate::backend::MemoryUpdate;
use cpu::CpuChange;
use k580_core::{Cpu8080State, CpuMetadata, RegisterName};
use std::collections::VecDeque;

const UNDO_DEPTH_LIMIT: usize = 256;

#[derive(Debug)]
enum UndoEntry {
    Text {
        field: &'static str,
        before: String,
        after: String,
    },
    Cpu {
        change: CpuChange,
        register_selection: Option<(RegisterName, RegisterName)>,
    },
}

#[derive(Default, Debug)]
pub(crate) struct UndoStack {
    undo: VecDeque<UndoEntry>,
    redo: VecDeque<UndoEntry>,
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
            }) = self.undo.back_mut()
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
            change: CpuChange::between(before, after),
            register_selection,
        });
        self.coalesce_field = None;
    }

    pub(crate) fn break_coalescing(&mut self) {
        self.coalesce_field = None;
    }

    pub(crate) fn pop_undo(&mut self) -> Option<UndoReplay> {
        let entry = self.undo.pop_back()?;
        self.coalesce_field = None;
        let replay = entry.replay(Direction::Undo);
        self.redo.push_back(entry);
        Some(replay)
    }

    pub(crate) fn pop_redo(&mut self) -> Option<UndoReplay> {
        let entry = self.redo.pop_back()?;
        self.coalesce_field = None;
        let replay = entry.replay(Direction::Redo);
        self.undo.push_back(entry);
        Some(replay)
    }

    pub(crate) fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
        self.coalesce_field = None;
    }

    fn push_entry(&mut self, entry: UndoEntry) {
        self.undo.push_back(entry);
        if self.undo.len() > UNDO_DEPTH_LIMIT {
            self.undo.pop_front();
        }
    }
}

#[derive(Clone, Copy)]
enum Direction {
    Undo,
    Redo,
}

#[derive(Debug)]
pub(crate) enum UndoReplay {
    Text {
        field: &'static str,
        value: String,
    },
    Cpu {
        metadata: CpuMetadata,
        memory: MemoryUpdate,
        register_selection: Option<RegisterName>,
    },
}

impl UndoEntry {
    fn replay(&self, direction: Direction) -> UndoReplay {
        let forward = matches!(direction, Direction::Redo);
        match self {
            Self::Text {
                field,
                before,
                after,
            } => UndoReplay::Text {
                field,
                value: if forward {
                    after.clone()
                } else {
                    before.clone()
                },
            },
            Self::Cpu {
                change,
                register_selection,
            } => {
                let (metadata, memory) = change.replay(direction);
                UndoReplay::Cpu {
                    metadata,
                    memory,
                    register_selection: register_selection
                        .map(|(before, after)| if forward { after } else { before }),
                }
            }
        }
    }
}
