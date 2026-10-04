use crate::backend::{AppCommand, CpuChange};
use k580_core::{Cpu8080State, CpuMetadata};

pub(super) enum CpuCheckpoint {
    Cells {
        metadata: CpuMetadata,
        cells: Vec<(u16, u8)>,
    },
    Full(Cpu8080State),
}

impl CpuCheckpoint {
    pub(super) fn capture(cpu: &Cpu8080State, command: &AppCommand) -> Option<Self> {
        let cells = match command {
            AppCommand::SetMemory(address, _) => vec![(*address, cpu.memory.read(*address))],
            AppCommand::SetMemoryBlock { start, values } => {
                let end = usize::from(*start).saturating_add(values.len()).min(65_536);
                (usize::from(*start)..end)
                    .map(|address| (address as u16, cpu.memory.read(address as u16)))
                    .collect()
            }
            AppCommand::SetRegister(_, _)
            | AppCommand::SetPc(_)
            | AppCommand::ResetCpu
            | AppCommand::ClearHalt
            | AppCommand::SetHalted(_) => Vec::new(),
            AppCommand::ToggleHalt => Vec::new(),
            AppCommand::ResetRam
            | AppCommand::ApplyCpuState(_)
            | AppCommand::ApplyCpuDelta { .. }
            | AppCommand::StepInstruction
            | AppCommand::StepTact
            | AppCommand::RunForTStates(_) => return Some(Self::Full(cpu.clone())),
            _ => return None,
        };
        Some(Self::Cells {
            metadata: cpu.metadata(),
            cells,
        })
    }

    pub(super) fn finish(self, after: &Cpu8080State) -> CpuChange {
        match self {
            Self::Cells { metadata, cells } => CpuChange::from_cells(metadata, cells, after),
            Self::Full(before) => CpuChange::between(before, after),
        }
    }
}
