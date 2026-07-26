/// Printer state reported by the spooler. Carries no display text: the
/// view layer renders it in the application language.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PrinterStatus {
    Ready,
    Paused,
    Error,
    PendingDeletion,
    PaperJam,
    PaperOut,
    ManualFeed,
    PaperProblem,
    Offline,
    Busy,
    Printing,
    OutputBinFull,
    NotAvailable,
    Waiting,
    Processing,
    Initializing,
    WarmingUp,
    TonerLow,
    NoToner,
    UserIntervention,
    OutOfMemory,
    DoorOpen,
    #[default]
    Unknown,
}
