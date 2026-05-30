use crate::i18n::{Key, Lang};

/// Provenance of the live `status` string — used to re-render the
/// status bar when the UI language switches at runtime. Each variant
/// owns the raw values (numbers, paths, mnemonics) that the rendered
/// string was built from; the language-dependent prefix / unit comes
/// from `Lang::t` at render time inside [`StatusKind::render`].
#[derive(Clone, Debug)]
pub(crate) enum StatusKind {
    /// Set from a non-canonical source (raw error, port log line,
    /// search-error message). `render` returns `None` so the caller
    /// keeps the existing string verbatim.
    Custom,
    Ready,
    NewFile,
    CpuHalted,
    Stopped,
    TactProgress {
        tact_phase: u8,
        cycle_count: u64,
    },
    InstructionAt {
        mnemonic: String,
        pc_before: u16,
    },
    PortRead {
        port: u8,
        value: u8,
    },
    PortWrite {
        port: u8,
        value: u8,
    },
    NoProgramAt {
        pc: u16,
    },
    Opened {
        display: String,
        legacy: bool,
    },
    SavedTo {
        display: String,
        legacy: bool,
    },
    ExportTo {
        display: String,
    },
    NothingToUndo,
    NothingToRedo,
    EnterHexPattern,
    PatternFound {
        pattern: String,
        address: u16,
    },
    NoMatchesFor {
        pattern: String,
    },
}

impl StatusKind {
    pub(crate) fn render(&self, lang: Lang) -> Option<String> {
        Some(match self {
            Self::Custom => return None,
            Self::Ready => lang.t(Key::StatusReady).to_owned(),
            Self::NewFile => lang.t(Key::StatusNewFile).to_owned(),
            Self::CpuHalted => lang.t(Key::StatusCpuHalted).to_owned(),
            Self::Stopped => lang.t(Key::StatusStopped).to_owned(),
            Self::TactProgress {
                tact_phase,
                cycle_count,
            } => format!(
                "{} {} {} {}",
                lang.t(Key::StatusTact),
                tact_phase,
                lang.t(Key::StatusCycle),
                cycle_count
            ),
            Self::InstructionAt {
                mnemonic,
                pc_before,
            } => format!("{mnemonic} at {pc_before:04X}"),
            Self::PortRead { port, value } => format!("IN {port:02X} -> {value:02X}"),
            Self::PortWrite { port, value } => format!("OUT {port:02X} <- {value:02X}"),
            Self::NoProgramAt { pc } => {
                format!("{} {pc:04X}", lang.t(Key::StatusNoProgramAt))
            }
            Self::Opened { display, legacy } => {
                if *legacy {
                    format!(
                        "{} {display} ({})",
                        lang.t(Key::StatusOpened),
                        lang.t(Key::LegacyFormatNote)
                    )
                } else {
                    format!("{} {display}", lang.t(Key::StatusOpened))
                }
            }
            Self::SavedTo { display, legacy } => {
                if *legacy {
                    format!(
                        "{} {display} ({})",
                        lang.t(Key::StatusSavedTo),
                        lang.t(Key::LegacyFormatNote)
                    )
                } else {
                    format!("{} {display}", lang.t(Key::StatusSavedTo))
                }
            }
            Self::ExportTo { display } => format!("{} {display}", lang.t(Key::StatusExportTo)),
            Self::NothingToUndo => lang.t(Key::StatusNothingToUndo).to_owned(),
            Self::NothingToRedo => lang.t(Key::StatusNothingToRedo).to_owned(),
            Self::EnterHexPattern => lang.t(Key::StatusEnterHexPattern).to_owned(),
            Self::PatternFound { pattern, address } => format!(
                "{} {pattern} {} {address:04X}",
                lang.t(Key::StatusPatternFound),
                lang.t(Key::StatusAtAddress)
            ),
            Self::NoMatchesFor { pattern } => {
                format!("{} {pattern}", lang.t(Key::StatusNoMatchesFor))
            }
        })
    }
}
