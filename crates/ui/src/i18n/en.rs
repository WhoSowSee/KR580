use super::{help_en, keys::Key};

pub(super) fn translate(key: Key) -> &'static str {
    match key {
        Key::MenuFile => "File",
        Key::MenuMp => "MP-System",
        Key::MenuView => "View",
        Key::MenuSettings => "Settings",
        Key::MenuHelp => "Help",

        Key::HelpShowDocs => "Show help",
        Key::HelpAbout => "About",
        Key::HelpDialogTitle => "Help",
        Key::HelpSearchPlaceholder => "Search help…",
        Key::HnAbout => "About",
        Key::HnAppearance => "Appearance",
        Key::HnArchitecture => "CPU Architecture",
        Key::HnArithmeticCommands => "Arithmetic",
        Key::HnCommandPanel => "Command Panel",
        Key::HnCommandReference => "Command Reference",
        Key::HnCommandSummary => "Appendix: Command Summary",
        Key::HnControlTransferCommands => "Control Transfer",
        Key::HnCpuArchitecture => "KR580VM80 Processor",
        Key::HnDataTransferCommands => "Data Transfer",
        Key::HnDeviceWorkflow => "Working with Devices",
        Key::HnExport => "Export Data",
        Key::HnExternalDevices => "Peripheral Devices",
        Key::HnFeatures => "Features",
        Key::HnFileFormats => "File Formats",
        Key::HnFilesExport => "Export & Files",
        Key::HnFlagsRegister => "Flags Register",
        Key::HnFloppy => "KR580 Floppy",
        Key::HnGeneralPrinciples => "General Principles",
        Key::HnGeneralSettings => "General",
        Key::HnHdd => "KR580 Hard Disk",
        Key::HnImport => "Import Subprograms",
        Key::HnInstructionSet => "Instruction Set",
        Key::HnIntroduction => "Introduction",
        Key::HnIoCommands => "I/O",
        Key::HnLogicalCommands => "Logical",
        Key::HnMainMenu => "Main Menu",
        Key::HnMainWindow => "Main Window",
        Key::HnMemoryIoSpaces => "Memory & I/O Spaces",
        Key::HnMemorySearch => "Memory Search",
        Key::HnMenuFile => "File Menu",
        Key::HnMenuHelp => "Help Menu",
        Key::HnMenuMpSystem => "MP-System Menu",
        Key::HnMonitor => "KR580 Monitor",
        Key::HnNetwork => "KR580 Network Adapter",
        Key::HnPrinter => "KR580 Printer",
        Key::HnProcessorControlCommands => "Processor Control",
        Key::HnProgramInterface => "Program Description",
        Key::HnRamEditing => "RAM Editing Panel",
        Key::HnRamTable => "RAM Table",
        Key::HnRegisterEdit => "Register Editing",
        Key::HnRegisterEditing => "Register Editing Panel",
        Key::HnRegisters => "Registers",
        Key::HnResetButtons => "Reset Buttons",
        Key::HnRunButtons => "Run Buttons",
        Key::HnSaveLoad => "Save & Load",
        Key::HnSchematic => "Structural Diagram",
        Key::HnSettings => "Settings",
        Key::HnShortcuts => "Keyboard Shortcuts",
        Key::HnStackCommands => "Stack",
        Key::HnSystemComponents => "MPS Components",
        Key::HnSystemComposition => "MP-System Composition",
        Key::HnTopicShortcuts => "Shortcuts Table",
        Key::HnWorkflow => "Working with the Program",

        Key::HcAbout
        | Key::HcFeatures
        | Key::HcSystemComponents
        | Key::HcArchitecture
        | Key::HcRegisters
        | Key::HcFlagsRegister
        | Key::HcMemoryIoSpaces
        | Key::HcDataTransferCommands
        | Key::HcLogicalCommands
        | Key::HcArithmeticCommands
        | Key::HcControlTransferCommands
        | Key::HcProcessorControlCommands
        | Key::HcIoCommands
        | Key::HcStackCommands
        | Key::HcMainWindow
        | Key::HcMenuFile
        | Key::HcMenuMpSystem
        | Key::HcMenuHelp
        | Key::HcSchematic
        | Key::HcRamTable
        | Key::HcMonitor
        | Key::HcFloppy
        | Key::HcHdd
        | Key::HcNetwork
        | Key::HcPrinter
        | Key::HcRamEditing
        | Key::HcRegisterEditing
        | Key::HcResetButtons
        | Key::HcCommandPanel
        | Key::HcRunButtons
        | Key::HcSaveLoad
        | Key::HcImport
        | Key::HcExport
        | Key::HcFileFormats
        | Key::HcGeneralSettings
        | Key::HcAppearance
        | Key::HcGeneralPrinciples
        | Key::HcMemorySearch
        | Key::HcRegisterEdit
        | Key::HcDeviceWorkflow
        | Key::HcCommandSummary
        | Key::HcShortcuts => help_en::translate(key),
        Key::AboutTitle => "About",
        Key::AppName => "KR580",
        Key::AboutDescription => "Microprocessor system emulator based on the KR580VM80 chip",
        Key::AboutVersion => "Version 1.0.0",
        Key::AboutGithubLabel => "GitHub",

        Key::FileNew => "New file",
        Key::FileOpen => "Open",
        Key::FileSave => "Save",
        Key::FileSaveAs => "Save as",
        Key::FileImport => "Import",
        Key::FileExport => "Export",
        Key::LegacyFormatNote => "legacy format",

        Key::MpRunProgram => "Run program",
        Key::MpRunInstruction => "Run instruction",
        Key::MpRunTact => "Run tact",
        Key::MpResetRam => "Clear RAM",
        Key::MpResetCpu => "Clear registers",
        Key::MpClearHalt => "Clear HLT flag",

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

        Key::SpeedTitle => "Speed",
        Key::SpeedUnit => "instr/sec",

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

        Key::HeaderStatus => "Status",
        Key::HltOn => "HLT ON",
        Key::HltOff => "HLT OFF",

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

        Key::Multiplexer => "Multiplexer",
        Key::TempStorageRegisters => "Temporary storage registers",
        Key::GeneralPurposeRegisters => "General-purpose registers",
        Key::StackPointer => "Stack pointer (SP)",
        Key::ProgramCounter => "Program counter (PC)",
        Key::IncDec => "Increment-decrement",

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

        Key::MemoryListTitle => "RAM contents",
        Key::ColumnAddress => "Address",
        Key::ColumnValue => "Value",
        Key::ColumnCommand => "Command",

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

        Key::QuickAccess => "Quick access",
        Key::DeviceMonitor => "Show monitor",
        Key::DeviceFloppy => "Show floppy buffer",
        Key::DeviceHdd => "Show HDD buffer",
        Key::DeviceNetwork => "Show network buffer",
        Key::DevicePrinter => "Show printer buffer",

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

        Key::OpcodeSearchPlaceholder => "Search: hex or mnemonic",

        Key::StatusByteHeader => "Status byte T1: what the CPU is doing on this tact.",
        Key::StatusPrefix => "Status:",

        Key::StatusNoProgramAt => "No program at address",
        Key::StatusNothingToUndo => "Nothing to undo",
        Key::StatusNothingToRedo => "Nothing to redo",
        Key::StatusEnterHexPattern => "Enter a hex pattern to search",
        Key::StatusPatternFound => "Found pattern",
        Key::StatusAtAddress => "at address",
        Key::StatusNoMatchesFor => "No addresses match",

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
