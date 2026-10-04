#[cfg(test)]
mod tests;

use crate::backend::MemoryUpdate;
use crate::backend::{ChangeDirection as Direction, CpuChange};
#[cfg(test)]
use k580_core::Cpu8080State;
use k580_core::{CpuMetadata, RegisterName};
use std::collections::VecDeque;

const UNDO_DEPTH_LIMIT: usize = 256;

#[derive(Debug)]
enum UndoEntry {
    Pending(crate::backend::RequestId),
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
    pub(crate) fn has_text_undo(&self) -> bool {
        matches!(self.undo.back(), Some(UndoEntry::Text { .. }))
    }
    pub(crate) fn has_text_redo(&self) -> bool {
        matches!(self.redo.back(), Some(UndoEntry::Text { .. }))
    }
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

    #[cfg(test)]
    pub(crate) fn push_cpu(&mut self, before: Cpu8080State, after: Cpu8080State) {
        self.push_change(CpuChange::between(before, &after), None);
    }

    #[cfg(test)]
    pub(crate) fn push_change(
        &mut self,
        change: CpuChange,
        register_selection: Option<(RegisterName, RegisterName)>,
    ) {
        if change.is_empty() {
            return;
        }
        self.redo.clear();
        self.push_entry(UndoEntry::Cpu {
            change,
            register_selection,
        });
        self.coalesce_field = None;
    }

    pub(crate) fn break_coalescing(&mut self) {
        self.coalesce_field = None;
    }

    pub(crate) fn reserve_cpu(&mut self, id: crate::backend::RequestId) {
        self.push_entry(UndoEntry::Pending(id));
        self.coalesce_field = None;
    }

    pub(crate) fn complete_cpu(
        &mut self,
        id: crate::backend::RequestId,
        change: Option<CpuChange>,
        register_selection: Option<(RegisterName, RegisterName)>,
    ) {
        let Some(index) = self
            .undo
            .iter()
            .position(|entry| matches!(entry, UndoEntry::Pending(pending) if *pending==id))
        else {
            return;
        };
        if let Some(change) = change.filter(|change| !change.is_empty()) {
            self.redo.clear();
            self.undo[index] = UndoEntry::Cpu {
                change,
                register_selection,
            };
        } else {
            self.undo.remove(index);
        }
    }

    pub(crate) fn cancel_replay(&mut self, direction: Direction) {
        let (source, target) = match direction {
            Direction::Undo => (&mut self.redo, &mut self.undo),
            Direction::Redo => (&mut self.undo, &mut self.redo),
        };
        if let Some(entry) = source.pop_back() {
            target.push_back(entry);
        }
    }

    pub(crate) fn pop_undo(&mut self) -> Option<UndoReplay> {
        if matches!(self.undo.back(), Some(UndoEntry::Pending(_))) {
            return None;
        }
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
            Self::Pending(_) => unreachable!("pending CPU history is not replayable"),
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
