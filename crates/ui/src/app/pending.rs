use std::path::PathBuf;

use crate::app::subprogram_modal::SubprogramDialog;
use crate::backend::RequestId;
use k580_core::Cpu8080State;

pub(crate) enum PendingRequest {
    LoadProgram {
        path: PathBuf,
        display: String,
    },
    SaveProgram {
        path: PathBuf,
        display: String,
        state: Box<Cpu8080State>,
    },
    SaveSubprogram {
        path: PathBuf,
        display: String,
        start: u16,
        end: u16,
        state: Box<Cpu8080State>,
    },
    LoadSubprogram {
        dialog: SubprogramDialog,
        start: u16,
    },
    Export {
        display: String,
    },
    Import {
        display: String,
    },
}

pub(crate) type PendingRequests = std::collections::HashMap<RequestId, PendingRequest>;
