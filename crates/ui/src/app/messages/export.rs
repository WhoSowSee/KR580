#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExportTab {
    Xlsx,
    Text,
}

impl ExportTab {
    pub(crate) fn extension(self) -> &'static str {
        match self {
            Self::Xlsx => "xlsx",
            Self::Text => "txt",
        }
    }

    pub(crate) fn default_file_name(self) -> &'static str {
        match self {
            Self::Xlsx => "kr580_export.xlsx",
            Self::Text => "kr580_export.txt",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExportMemoryColumn {
    Address,
    Value,
    Command,
    Comment,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExportRegister {
    Accumulator,
    W,
    Z,
    B,
    C,
    D,
    E,
    H,
    L,
    StackPointer,
    ProgramCounter,
    Cycles,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExportFlag {
    Sign,
    Zero,
    AuxiliaryCarry,
    Parity,
    Carry,
}
