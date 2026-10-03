use crate::backend::MemoryUpdate;
use k580_core::{Cpu8080State, CpuMetadata, Memory64K};

#[derive(Debug)]
pub(super) struct CpuChange {
    before: CpuMetadata,
    after: CpuMetadata,
    memory: MemoryChange,
}

#[derive(Debug)]
enum MemoryChange {
    Cells(Vec<(u16, u8, u8)>),
    Full { before: Memory64K, after: Memory64K },
}

impl CpuChange {
    pub(super) fn between(before: Cpu8080State, after: Cpu8080State) -> Self {
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
                after: after.memory,
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

    pub(super) fn replay(&self, direction: super::Direction) -> (CpuMetadata, MemoryUpdate) {
        let forward = matches!(direction, super::Direction::Redo);
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
