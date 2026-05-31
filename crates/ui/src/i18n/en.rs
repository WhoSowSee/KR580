use super::keys::Key;

pub(super) fn translate(key: Key) -> &'static str {
    match key {
        // Top menu
        Key::MenuFile => "File",
        Key::MenuMp => "MP-System",
        Key::MenuView => "View",
        Key::MenuSettings => "Settings",
        Key::MenuHelp => "Help",

        // File menu
        Key::FileNew => "New file",
        Key::FileOpen => "Open",
        Key::FileSave => "Save",
        Key::FileSaveAs => "Save as",
        Key::FileImport => "Import",
        Key::FileExport => "Export",
        Key::LegacyFormatNote => "legacy format",

        // MP-System menu
        Key::MpRunProgram => "Run program",
        Key::MpRunInstruction => "Run instruction",
        Key::MpRunTact => "Run tact",
        Key::MpResetRam => "Clear RAM",
        Key::MpResetCpu => "Clear registers",
        Key::MpClearHalt => "Clear HLT flag",

        // Discard modal
        Key::DiscardCancel => "Cancel",
        Key::DiscardBody => "Unsaved changes will be lost.",
        Key::DiscardTitleOpen => "Open file",
        Key::DiscardTitleNew => "New file",
        Key::DiscardTitleImport => "Import",
        Key::DiscardTitleClose => "Close application",
        Key::DiscardConfirmOpen => "Open",
        Key::DiscardConfirmNew => "Create",
        Key::DiscardConfirmImport => "Import",
        Key::DiscardConfirmClose => "Close",

        // Status & notices
        Key::StatusReady => "Ready",
        Key::StatusNewFile => "New file",
        Key::StatusCpuHalted => "CPU halted",
        Key::StatusStopped => "Stopped",
        Key::StatusTact => "Tact",
        Key::StatusCycle => "cycle",
        Key::StatusOpened => "Opened",
        Key::StatusSavedTo => "Saved to",
        Key::StatusExportTo => "Exported to",
        Key::ErrorPrefix => "Error",
        Key::LegacyOpenedNotice => "Opened a legacy-format file",
        Key::HaltNotice => {
            "CPU halted by the HLT instruction\nReset registers or clear the HLT flag"
        }

        // Speed panel
        Key::SpeedTitle => "Speed",
        Key::SpeedUnit => "instr/sec",

        // Settings dialog
        Key::SettingsTitle => "Settings",
        Key::SettingsSearchPlaceholder => "Search settings",
        Key::SettingsCategoryGeneral => "General",
        Key::SettingsCategoryAppearance => "Appearance",
        Key::SettingsCategoryShortcuts => "Shortcuts",
        Key::SettingsLanguageLabel => "Language",
        Key::SettingsLanguageHint => "Application interface language",
        Key::SettingsSpeedLabel => "Speed",
        Key::SettingsSpeedHint => "Default speed for all files",
        Key::SettingsThemeLabel => "Theme",
        Key::SettingsThemeHint => "Interface theme",
        Key::SettingsThemePlaceholder => "Coming soon",
        Key::SettingsShortcutsLabel => "Keyboard shortcuts",
        Key::SettingsShortcutsHint => "Customize keyboard shortcuts",
        Key::SettingsNoMatches => "No matches",
        Key::SettingsReset => "Reset",
        Key::SettingsResetConfirmTitle => "Reset settings?",
        Key::SettingsResetConfirmBody => "All settings will be restored to their defaults",
        Key::SettingsResetConfirmAction => "Reset",
        Key::LangRussian => "Russian",
        Key::LangEnglish => "English",
        Key::SpeedSlow => "Slow",
        Key::SpeedMedium => "Medium",
        Key::SpeedHigh => "Fast",
        Key::SpeedMax => "Max",

        // Schematic header
        Key::HeaderStatus => "Status",
        Key::HltOn => "HLT ON",
        Key::HltOff => "HLT OFF",

        // Schematic registers grid
        Key::RegistersAndOperands => "Registers and operands",
        Key::Accumulator => "Accumulator",
        Key::BufferRegister1 => "Buffer register 1",
        Key::BufferRegister2 => "Buffer register 2",
        Key::AddressBuffer => "Address buffer",
        Key::InstructionRegister => "Instruction register",
        Key::InstructionDecoder => "Decoder",
        Key::ControlSignals => "Control signals",
        Key::CurrentCommand => "Current command",
        Key::DataBuffer => "Data buffer",
        Key::FlagsRegister => "Flags register",
        Key::StatusRegister => "Status register",

        // Mux panel
        Key::Multiplexer => "Multiplexer",
        Key::TempStorageRegisters => "Temporary storage registers",
        Key::GeneralPurposeRegisters => "General-purpose registers",
        Key::StackPointer => "Stack pointer (SP)",
        Key::ProgramCounter => "Program counter (PC)",
        Key::IncDec => "Increment-decrement",

        // Cycles / timings
        Key::CyclesAndTacts => "Cycle and tact",
        Key::CycleLabel => "Cycle",
        Key::TactLabel => "Tact",
        Key::CycleTooltip => "Which step the current instruction is executing. Counts from one.",
        Key::TactTooltip => "Tact number within the current step. Counts from one.",
        Key::InternalTimings => "Internal timings",
        Key::TotalTacts => "Tacts",
        Key::InstructionTact => "Instruction tact",
        Key::PhaseLabel => "Phase",
        Key::TotalTactsTooltip => {
            "Total tacts elapsed since the start of the program. Counts from zero."
        }
        Key::InstructionTactTooltip => {
            "Tact number within the current instruction on the full scale (T1, T2, ...). Counts from one."
        }
        Key::PhaseTooltip => "Same as 'Instruction tact' but counts from zero.",

        // Memory list
        Key::MemoryListTitle => "RAM contents",
        Key::ColumnAddress => "Address",
        Key::ColumnValue => "Value",
        Key::ColumnCommand => "Command",

        // Editors panels
        Key::MemoryEditorTitle => "RAM cell and value",
        Key::RegisterEditorTitle => "Register and value",
        Key::ActionPause => "Pause",
        Key::ActionRunProgram => "Run program",
        Key::ActionRestartProgram => "Restart program",
        Key::ActionStepInstruction => "Step instruction",
        Key::ActionStepTact => "Step tact",
        Key::ActionResetRam => "Reset RAM",
        Key::ActionResetCpu => "Reset registers",
        Key::ExecutionPanel => "Execution",
        Key::ResetPanel => "Reset",

        // Quick access devices
        Key::QuickAccess => "Quick access",
        Key::DeviceMonitor => "Show monitor",
        Key::DeviceFloppy => "Show floppy buffer",
        Key::DeviceHdd => "Show HDD buffer",
        Key::DeviceNetwork => "Show network buffer",
        Key::DevicePrinter => "Show printer buffer",

        // Monitor window
        Key::MonitorUnifiedScreen => "KR580 screen",
        Key::MonitorTextLayer => "Text layer",
        Key::MonitorPixelLayer => "Graphics layer",
        Key::MonitorHexBuffer => "Byte stream",
        Key::MonitorClose => "Close",
        Key::MonitorViewSplit => "Split",
        Key::MonitorViewUnified => "Unified",
        Key::MonitorClearBuffer => "Clear buffer",
        Key::MonitorSaveImage => "Save image",
        Key::MonitorImageSaved => "Monitor image saved",
        Key::MonitorImageSaveFailed => "Failed to save monitor image",
        Key::MonitorHexFilterAll => "Filter: all",
        Key::MonitorHexFilterGraphics => "Filter: graphics",
        Key::MonitorHexFilterText => "Filter: text",

        // Current command columns
        Key::ColCmdCode => "Code",
        Key::ColCmdMnemonic => "Command",
        Key::ColCmdOperand => "Operand",
        Key::ColCmdLength => "Length",
        Key::ColCmdKind => "Kind",
        Key::ColCmdAddressing => "Addressing",
        Key::CmdLengthByte => "1 byte",
        Key::CmdLengthBytes2 => "2 bytes",
        Key::CmdLengthBytes3 => "3 bytes",
        Key::CmdKindUnknown => "unknown",
        Key::CmdKindControl => "control",
        Key::CmdKindBranch => "branch",
        Key::CmdKindStack => "stack",
        Key::CmdKindIo => "I/O",
        Key::CmdKindMove => "move",
        Key::CmdKindLogic => "logic",
        Key::CmdKindArithmetic => "arithmetic",
        Key::CmdAddrImplicit => "implicit",
        Key::CmdAddrImmediate => "immediate",
        Key::CmdAddrDirect => "direct",
        Key::CmdAddrIndirect => "indirect",
        Key::CmdAddrRegister => "register",

        // Opcode dropdown
        Key::OpcodeSearchPlaceholder => "Search: hex or mnemonic",

        // Status register tooltip
        Key::StatusByteHeader => "Status byte T1: what the CPU is doing on this tact.",
        Key::StatusPrefix => "Status:",

        // Runtime status messages
        Key::StatusNoProgramAt => "No program at address",
        Key::StatusNothingToUndo => "Nothing to undo",
        Key::StatusNothingToRedo => "Nothing to redo",
        Key::StatusEnterHexPattern => "Enter a hex pattern to search",
        Key::StatusPatternFound => "Found pattern",
        Key::StatusAtAddress => "at address",
        Key::StatusNoMatchesFor => "No addresses match",

        // Humanize error
        Key::ErrFileCorruptedOrUnsupported => "File is corrupted or has an unsupported format",
        Key::ErrFileNewerVersion => {
            "File was saved by a newer version — please update the application"
        }
        Key::ErrNotLegacyFormat => "File does not look like a legacy-format save",
        Key::ErrLegacyTrailerCorrupt => {
            "File trailer is corrupted — this is not a legacy-format save"
        }
        Key::ErrSettingsNewerVersion => {
            "Settings file was saved by a newer version — please update the application"
        }
        Key::ErrSettingsCorrupt => "Settings file is corrupted",
        Key::ErrCannotReadFileFormat => "Failed to read the file — check the format",
        Key::ErrCannotReadFile => "Failed to read the file",
        Key::ErrCannotWriteTable => "Failed to write the table",
        Key::ErrCannotWriteFile => "Failed to write the file",
        Key::ErrFileNotFound => "File not found",
        Key::ErrPermissionDenied => "Permission denied for file",
        Key::ErrFileAlreadyExists => "File already exists",
        Key::ErrDiskFull => "Not enough disk space",
        Key::ErrIoGeneric => "Read or write error",
        Key::ErrAddressOutOfRange => "Address out of memory range",
        Key::ErrUnknownRegister => "Unknown register name",
        Key::ErrUndocumentedOpcode => "Undocumented opcode",
        Key::ErrInternal => "Internal application error",
        Key::ErrGenericFailed => "Operation failed",
    }
}
