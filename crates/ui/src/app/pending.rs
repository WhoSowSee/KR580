use std::path::PathBuf;

use crate::app::subprogram_modal::SubprogramDialog;
use crate::backend::RequestId;

#[derive(Clone, Copy)]
pub(crate) enum UndoPolicy {
    Record,
    Skip,
}

pub(crate) enum BackendAction {
    None,
    Instruction,
    KeepCursor,
    Tact,
    Restart,
    Replay(Option<k580_core::RegisterName>),
    NewFile {
        edit_epoch: u64,
    },
    Register {
        source: k580_core::RegisterName,
        value: u8,
        target: RegisterCompletion,
        replacing: bool,
    },
    Memory {
        address: u16,
        value: u8,
        target: MemoryCompletion,
    },
    FloppyAttached(PathBuf),
    FloppyDetached,
    HddAttached(PathBuf),
    HddDeleted(PathBuf),
    NetworkConfigured {
        mode: crate::backend::NetworkMode,
        host: String,
        port: u16,
    },
}

pub(crate) enum RegisterCompletion {
    Inline {
        source: super::RegisterInlineTarget,
        next: Option<super::RegisterInlineTarget>,
    },
    Field {
        next: k580_core::RegisterName,
        input: &'static str,
        focus: Option<&'static str>,
    },
}

pub(crate) enum MemoryCompletion {
    Inline { delta: i32, replacing: bool },
    ValueStep { backward: bool },
    Jump,
}

pub(crate) enum PendingRequest {
    Command {
        action: BackendAction,
    },
    CpuEdit {
        undo: UndoPolicy,
        register_selection: Option<(k580_core::RegisterName, k580_core::RegisterName)>,
        action: BackendAction,
    },
    LoadProgram {
        path: PathBuf,
        display: String,
    },
    SaveProgram {
        path: PathBuf,
        display: String,
    },
    SaveSubprogram {
        path: PathBuf,
        display: String,
        start: u16,
        end: u16,
    },
    LoadSubprogram {
        dialog: SubprogramDialog,
        start: u16,
        edit_epoch: u64,
    },
    Export {
        display: String,
    },
    Import {
        display: String,
        edit_epoch: u64,
    },
}

pub(crate) type PendingRequests = std::collections::HashMap<RequestId, PendingRequest>;
