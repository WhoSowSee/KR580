use super::MemoryUpdate;
use k580_core::{Cpu8080State, CpuMetadata, Memory64K};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CpuChange {
    before: CpuMetadata,
    after: CpuMetadata,
    memory: MemoryChange,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum MemoryChange {
    Cells(Vec<(u16, u8, u8)>),
    Full { before: Memory64K, after: Memory64K },
}

impl CpuChange {
    pub fn between(before: Cpu8080State, after: &Cpu8080State) -> Self {
        let count = before
            .memory
            .as_slice()
            .iter()
            .zip(after.memory.as_slice())
            .filter(|(a, b)| a != b)
            .count();
        let before_metadata = before.metadata();
        let after_metadata = after.metadata();
        let memory = if count > Memory64K::SIZE / 2 {
            MemoryChange::Full {
                before: before.memory,
                after: after.memory.clone(),
            }
        } else {
            let cells = before
                .memory
                .as_slice()
                .iter()
                .zip(after.memory.as_slice())
                .enumerate()
                .filter_map(|(address, (&before, &after))| {
                    (before != after).then_some((address as u16, before, after))
                })
                .collect();
            MemoryChange::Cells(cells)
        };
        Self {
            before: before_metadata,
            after: after_metadata,
            memory,
        }
    }

    pub fn replay(&self, direction: ChangeDirection) -> (CpuMetadata, MemoryUpdate) {
        let forward = matches!(direction, ChangeDirection::Redo);
        let metadata = if forward {
            self.after.clone()
        } else {
            self.before.clone()
        };
        let memory = match &self.memory {
            MemoryChange::Cells(cells) => MemoryUpdate::Cells(
                cells
                    .iter()
                    .map(|&(address, before, after)| {
                        (address, if forward { after } else { before })
                    })
                    .collect(),
            ),
            MemoryChange::Full { before, after } => MemoryUpdate::Replace(if forward {
                after.clone()
            } else {
                before.clone()
            }),
        };
        (metadata, memory)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChangeDirection {
    Undo,
    Redo,
}

impl CpuChange {
    pub fn is_empty(&self) -> bool {
        self.before == self.after
            && matches!(&self.memory, MemoryChange::Cells(cells) if cells.is_empty())
    }

    pub fn before(&self) -> &CpuMetadata {
        &self.before
    }
    pub fn after(&self) -> &CpuMetadata {
        &self.after
    }

    pub(in crate::backend) fn from_cells(
        before: CpuMetadata,
        cells: Vec<(u16, u8)>,
        after: &Cpu8080State,
    ) -> Self {
        let cells = cells
            .into_iter()
            .filter_map(|(address, before)| {
                let value = after.memory.read(address);
                (before != value).then_some((address, before, value))
            })
            .collect();
        Self {
            before,
            after: after.metadata(),
            memory: MemoryChange::Cells(cells),
        }
    }
}
