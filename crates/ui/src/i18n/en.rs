mod devices;
mod settings;

use super::{help_en, keys::Key};
pub(super) fn translate(key: Key) -> &'static str {
    if let Some(value) = settings::translate(key) {
        return value;
    }
    if let Some(value) = devices::translate(key) {
        return value;
    }

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
        Key::HnArithmeticCommands => "Arithmetic",
        Key::HnCommandSummary => "Command Summary",
        Key::HnControlTransferCommands => "Control Transfer",
        Key::HnCpuArchitecture => "KR580VM80 Processor",
        Key::HnDataTransferCommands => "Data Transfer",
        Key::HnExport => "Export",
        Key::HnExternalDevices => "Peripheral Devices",
        Key::HnFeatures => "Features",
        Key::HnFilesExport => "Files, Import & Export",
        Key::HnFlagsRegister => "Flags Register",
        Key::HnFloppy => "KR580 Floppy",
        Key::HnGeneralPrinciples => "Quick Start",
        Key::HnGeneralSettings => "All Settings",
        Key::HnHdd => "KR580 Hard Disk",
        Key::HnImport => "Import",
        Key::HnInstructionSet => "Instruction Set",
        Key::HnIntroduction => "Introduction",
        Key::HnIoCommands => "I/O",
        Key::HnLogicalCommands => "Logical",
        Key::HnMainMenu => "Main Menu",
        Key::HnMainWindow => "Main Window",
        Key::HnMemoryIoSpaces => "Memory & I/O Spaces",
        Key::HnMemorySearch => "Memory Navigation",
        Key::HnMenuFile => "File Menu",
        Key::HnMenuHelp => "Help Menu",
        Key::HnMenuMpSystem => "MP-System Menu",
        Key::HnMenuView => "View Menu",
        Key::HnMonitor => "KR580 Monitor",
        Key::HnNetwork => "KR580 Network Adapter",
        Key::HnPrinter => "KR580 Printer",
        Key::HnProcessorControlCommands => "Processor Control",
        Key::HnProgramInterface => "Interface & Execution",
        Key::HnRamEditing => "Viewing & Editing RAM",
        Key::HnRegisterEditing => "Register Editing",
        Key::HnRegisters => "Registers",
        Key::HnRunButtons => "Execution & Reset",
        Key::HnSaveLoad => ".580 Files",
        Key::HnSettings => "Settings",
        Key::HnStackCommands => "Stack",
        Key::HnTopicShortcuts => "Keyboard Shortcuts",

        Key::HcAbout
        | Key::HcFeatures
        | Key::HcGeneralPrinciples
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
        | Key::HcRamEditing
        | Key::HcRegisterEditing
        | Key::HcRunButtons
        | Key::HcMemorySearch
        | Key::HcMenuFile
        | Key::HcMenuMpSystem
        | Key::HcMenuView
        | Key::HcMenuHelp
        | Key::HcSaveLoad
        | Key::HcImport
        | Key::HcExport
        | Key::HcMonitor
        | Key::HcFloppy
        | Key::HcHdd
        | Key::HcNetwork
        | Key::HcPrinter
        | Key::HcGeneralSettings
        | Key::HcAppearance
        | Key::HcCommandSummary
        | Key::HcShortcuts => help_en::translate(key),
        Key::AboutTitle => "About",
        Key::AppName => "KR580",
        Key::AboutDescription => "Microprocessor system emulator based on the KR580VM80 chip",
        Key::AboutVersion => "Version",
        Key::AboutGithubLabel => "GitHub",
        Key::FileNew => "New file",
        Key::FileOpen => "Open",
        Key::FileSave => "Save",
        Key::FileSaveAs => "Save as",
        Key::FileImport => "Import",
        Key::FileExport => "Export",
        Key::ExportFormatXlsx => "MS Excel",
        Key::ExportFormatText => "Text file",
        Key::ExportPageLabel => "On page",
        Key::ExportPageDefault => "Subprogram 1",
        Key::ExportPageNameBase => "Subprogram",
        Key::ExportSectionLabel => "In section",
        Key::ExportSectionDefault => "Section 1",
        Key::ExportSectionNameBase => "Section",
        Key::ExportAddPageTooltip => "Add page",
        Key::ExportAddSectionTooltip => "Add section",
        Key::ExportDeletePageTooltip => "Delete page",
        Key::ExportDeleteSectionTooltip => "Delete section",
        Key::ExportMemoryGroup => "RAM contents",
        Key::ExportRegistersGroup => "Register values",
        Key::ExportFlagsGroup => "Flag values",
        Key::ExportRangeFrom => "Cells from",
        Key::ExportRangeTo => "to",
        Key::ExportColumnAddress => "Include \"RAM cell #\" column",
        Key::ExportColumnValue => "Include \"RAM cell value\" column",
        Key::ExportColumnCommand => "Include \"Command\" column",
        Key::ExportColumnComment => "Add empty comments column",
        Key::ExportRegisterAccumulator => "Accumulator",
        Key::ExportRegisterStackPointer => "Stack pointer",
        Key::ExportRegisterProgramCounter => "Program counter",
        Key::ExportRegisterCycles => "Tact counter",
        Key::ImportSourceGroup => "Import source",
        Key::ImportFileLabel => "File",
        Key::ImportNoFile => "No file selected",
        Key::ImportNoTargets => "The file has no separate sheets or sections",
        Key::ImportSheetLabel => "On sheet",
        Key::ImportSectionLabel => "In section",
        Key::ImportBrowseTooltip => "Choose file",
        Key::ImportChooseFileRequired => "Choose a file to import",
        Key::MpRunProgram => "Run program",
        Key::MpRunInstruction => "Run instruction",
        Key::MpRunTact => "Run tact",
        Key::MpResetRam => "Clear RAM",
        Key::MpResetCpu => "Clear registers",
        Key::MpClearHalt => "Clear HLT flag",
        Key::DiscardCancel => "Cancel",
        Key::DiscardBody => "Unsaved changes will be lost.",
        Key::DiscardBodyDeleteHdd => "All data will be lost.",
        Key::DiscardTitleOpen => "Open file",
        Key::DiscardTitleNew => "New file",
        Key::DiscardTitleImport => "Import",
        Key::DiscardTitleClose => "Close application",
        Key::DiscardTitleDeleteHdd => "Delete HDD file?",
        Key::DiscardConfirmOpen => "Open",
        Key::DiscardConfirmNew => "Create",
        Key::DiscardConfirmImport => "Import",
        Key::DiscardConfirmClose => "Close",
        Key::DiscardConfirmDeleteHdd => "Delete",
        Key::StatusReady => "Ready",
        Key::StatusNewFile => "New file",
        Key::StatusCpuHalted => "CPU halted",
        Key::StatusStopped => "Stopped",
        Key::StatusTact => "Tact",
        Key::StatusCycle => "cycle",
        Key::StatusOpened => "Opened",
        Key::StatusSavedTo => "Saved to",
        Key::StatusExportTo => "Exported to",
        Key::StatusImportFrom => "Imported from",
        Key::ErrorPrefix => "Error",
        Key::HaltNotice => {
            "CPU halted by the HLT instruction\nReset registers or clear the HLT flag"
        }

        Key::SpeedTitle => "Speed",
        Key::SpeedUnit => "instr/sec",
        Key::HeaderStatus => "Status",
        Key::HltOn => "HLT ON",
        Key::HltOff => "HLT OFF",
        Key::RegistersAndOperands => "Registers and operands",
        Key::Accumulator => "Accumulator",
        Key::BufferRegister1 => "Buffer register 1",
        Key::BufferRegister2 => "Buffer register 2",
        Key::AddressBuffer => "Address buffer",
        Key::AddressBufferTooltip => "Last 16-bit address driven by the CPU onto the address bus.",
        Key::InstructionRegister => "Instruction register",
        Key::InstructionRegisterTooltip => {
            "Opcode byte of the instruction currently being executed."
        }
        Key::InstructionDecoder => "Decoder",
        Key::InstructionDecoderTooltip => {
            "Human-readable mnemonic decoded from the instruction register."
        }
        Key::ControlSignals => "Control signals",
        Key::CurrentCommand => "Current command",
        Key::DataBuffer => "Data buffer",
        Key::DataBufferTooltip => {
            "Last byte that appeared on the CPU's 8-bit data bus. Updated on every memory or I/O read."
        }
        Key::FlagsRegister => "Flags register",
        Key::FlagsRegisterTooltip => {
            "Compact PSW flags: S (sign), Z (zero), AC (auxiliary carry), P (parity), C (carry). Bits 1 and 3 are always 0 on the 8080."
        }
        Key::StatusRegister => "Status register",
        Key::PswTooltip => {
            "Program status word: accumulator A concatenated with the PSW flag byte. Upper byte is A, lower byte is the flags."
        }
        Key::StackPointerTooltip => {
            "Stack pointer. Points to the top of the program stack in memory; PUSH decreases it, POP increases it."
        }
        Key::ProgramCounterTooltip => {
            "Program counter. Holds the address of the next instruction byte to be fetched."
        }
        Key::IncDecTooltip => {
            "Number of bytes the CPU will add to the program counter after the current instruction finishes."
        }
        Key::Multiplexer => "Multiplexer",
        Key::TempStorageRegisters => "Temporary storage registers",
        Key::GeneralPurposeRegisters => "General-purpose registers",
        Key::StackPointer => "Stack pointer (SP)",
        Key::ProgramCounter => "Program counter (PC)",
        Key::IncDec => "Increment-decrement",

        Key::LampF2 => {
            "Clock phase 2\nThe second half of the internal clock cycle; many control actions are strobed on this edge."
        }
        Key::LampF1 => {
            "Clock phase 1\nThe first half of the internal clock cycle; the CPU starts a new bus state here."
        }
        Key::LampSync => {
            "Synchronisation\nAsserted at the start of a machine cycle to identify the bus transaction type."
        }
        Key::LampReady => {
            "CPU ready\nWhen active, the CPU can finish the bus cycle; when low, it inserts wait states."
        }
        Key::LampWait => {
            "Wait state\nLit when the CPU is stretching a bus cycle because READY is not yet active."
        }
        Key::LampHold => {
            "Hold request\nA peripheral asks the CPU to release the system bus for DMA."
        }
        Key::LampInt => {
            "Interrupt request\nA peripheral is asking for attention; honoured only when INTE is set."
        }
        Key::LampInte => {
            "Interrupts enabled\nWhen set, the CPU will accept a maskable interrupt request (INT)."
        }
        Key::LampDbin => {
            "Data bus input\nThe CPU is reading data from memory or an I/O port onto the data bus."
        }
        Key::LampWr => {
            "Write strobe\nThe CPU is writing data from the accumulator or data bus to memory or I/O."
        }
        Key::LampHlda => {
            "Hold acknowledge\nThe CPU has released the bus and granted it to the requesting DMA controller."
        }
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
        Key::StatusInvalidMemoryBytes => "Invalid bytes: use space-separated hex pairs",
        Key::StatusMemoryBytesOutOfRange => "The byte sequence does not fit in memory",
        Key::StatusPatternFound => "Found pattern",
        Key::StatusAtAddress => "at address",
        Key::StatusNoMatchesFor => "No addresses match",
        Key::ErrNotA580File => "Not a .580 file – only .580 extension is supported",
        Key::ErrFileEmpty => "File is empty",
        Key::ErrWrong580Size => "Not a valid .580 file (must be exactly 65549 bytes)",
        Key::ErrLegacyTrailerCorrupt => "File trailer is corrupted – this is not a valid .580 file",
        Key::ErrSettingsNewerVersion => {
            "Settings file was saved by a newer version – please update the application"
        }
        Key::ErrSettingsCorrupt => "Settings file is corrupted",
        Key::ErrCannotReadFileFormat => "Failed to read the file – check the format",
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
        Key::ErrFloppyImageNotAttached => "Floppy image file is not attached",
        Key::ErrInternal => "Internal application error",
        Key::ErrGenericFailed => "Operation failed",
        _ => unreachable!("missing english translation for {key:?}"),
    }
}
