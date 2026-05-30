/// Identifier for a translatable string. Adding a new piece of UI text
/// means adding a variant here and a row per language in `ru.rs` /
/// `en.rs`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Key {
    // Top menu
    MenuFile,
    MenuMp,
    MenuView,
    MenuSettings,
    MenuHelp,

    // File dropdown
    FileNew,
    FileOpen,
    FileSave,
    FileSaveAs,
    FileImport,
    FileExport,
    LegacyFormatNote,

    // MP-System dropdown
    MpRunProgram,
    MpRunInstruction,
    MpRunTact,
    MpResetRam,
    MpResetCpu,
    MpClearHalt,

    // Discard modal
    DiscardCancel,
    DiscardBody,
    DiscardTitleOpen,
    DiscardTitleNew,
    DiscardTitleImport,
    DiscardTitleClose,
    DiscardConfirmOpen,
    DiscardConfirmNew,
    DiscardConfirmImport,
    DiscardConfirmClose,

    // Status / notices
    StatusReady,
    StatusNewFile,
    StatusCpuHalted,
    StatusStopped,
    StatusTact,
    StatusCycle,
    StatusOpened,
    StatusSavedTo,
    StatusExportTo,
    ErrorPrefix,
    LegacyOpenedNotice,
    HaltNotice,

    // Speed panel
    SpeedTitle,
    SpeedUnit,

    // Settings dialog
    SettingsTitle,
    SettingsSearchPlaceholder,
    SettingsCategoryGeneral,
    SettingsCategoryAppearance,
    SettingsCategoryShortcuts,
    SettingsLanguageLabel,
    SettingsLanguageHint,
    SettingsSpeedLabel,
    SettingsSpeedHint,
    SettingsThemeLabel,
    SettingsThemeHint,
    SettingsThemePlaceholder,
    SettingsShortcutsLabel,
    SettingsShortcutsHint,
    SettingsNoMatches,
    SettingsReset,
    SettingsResetConfirmTitle,
    SettingsResetConfirmBody,
    SettingsResetConfirmAction,
    LangRussian,
    LangEnglish,
    SpeedSlow,
    SpeedMedium,
    SpeedHigh,
    SpeedMax,

    // Schematic header
    HeaderStatus,
    HltOn,
    HltOff,

    // Schematic registers grid
    RegistersAndOperands,
    Accumulator,
    BufferRegister1,
    BufferRegister2,
    AddressBuffer,
    InstructionRegister,
    InstructionDecoder,
    ControlSignals,
    CurrentCommand,
    DataBuffer,
    FlagsRegister,
    StatusRegister,

    // Mux panel
    Multiplexer,
    TempStorageRegisters,
    GeneralPurposeRegisters,
    StackPointer,
    ProgramCounter,
    IncDec,

    // Cycles / timings
    CyclesAndTacts,
    CycleLabel,
    TactLabel,
    CycleTooltip,
    TactTooltip,
    InternalTimings,
    TotalTacts,
    InstructionTact,
    PhaseLabel,
    TotalTactsTooltip,
    InstructionTactTooltip,
    PhaseTooltip,

    // Memory list
    MemoryListTitle,
    ColumnAddress,
    ColumnValue,
    ColumnCommand,

    // Editors
    MemoryEditorTitle,
    RegisterEditorTitle,
    ActionPause,
    ActionRunProgram,
    ActionRestartProgram,
    ActionStepInstruction,
    ActionStepTact,
    ActionResetRam,
    ActionResetCpu,
    ExecutionPanel,
    ResetPanel,

    // Quick-access
    QuickAccess,
    DeviceMonitor,
    DeviceFloppy,
    DeviceHdd,
    DeviceNetwork,
    DevicePrinter,

    // Current command columns
    ColCmdCode,
    ColCmdMnemonic,
    ColCmdOperand,
    ColCmdLength,
    ColCmdKind,
    ColCmdAddressing,
    CmdLengthByte,
    CmdLengthBytes2,
    CmdLengthBytes3,
    CmdKindUnknown,
    CmdKindControl,
    CmdKindBranch,
    CmdKindStack,
    CmdKindIo,
    CmdKindMove,
    CmdKindLogic,
    CmdKindArithmetic,
    CmdAddrImplicit,
    CmdAddrImmediate,
    CmdAddrDirect,
    CmdAddrIndirect,
    CmdAddrRegister,

    // Opcode dropdown
    OpcodeSearchPlaceholder,

    // Status register tooltip
    StatusByteHeader,
    StatusPrefix,

    // Runtime status messages
    StatusNoProgramAt,
    StatusNothingToUndo,
    StatusNothingToRedo,
    StatusEnterHexPattern,
    StatusPatternFound,
    StatusAtAddress,
    StatusNoMatchesFor,

    // Humanize error
    ErrFileCorruptedOrUnsupported,
    ErrFileNewerVersion,
    ErrNotLegacyFormat,
    ErrLegacyTrailerCorrupt,
    ErrSettingsNewerVersion,
    ErrSettingsCorrupt,
    ErrCannotReadFileFormat,
    ErrCannotReadFile,
    ErrCannotWriteTable,
    ErrCannotWriteFile,
    ErrFileNotFound,
    ErrPermissionDenied,
    ErrFileAlreadyExists,
    ErrDiskFull,
    ErrIoGeneric,
    ErrAddressOutOfRange,
    ErrUnknownRegister,
    ErrUndocumentedOpcode,
    ErrInternal,
    ErrGenericFailed,
}
