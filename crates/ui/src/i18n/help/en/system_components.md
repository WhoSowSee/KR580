An 8-bit MP-system based on the KR580VM80A consists of:

- CPU (KR580VM80A) - data processing and control
- RAM - 64 KB (addresses 0000h-FFFFh)
- ROM - not separately modeled; programs are loaded into RAM
- Clock Generator - modeled as configurable execution speed
- System Controller - generates bus control signals
- Buffer Registers and Bus Drivers - bus demultiplexing
- Address Bus (16 lines) - carries memory/port addresses
- Data Bus (8 lines) - bidirectional data transfer
- Control Bus - sync, R/W, interrupt signals
- Peripherals - connected via I/O ports (00h-FFh)

All components are visualized on the structural diagram.} // About dialogKey::AboutTitle => "About",Key::AppName => "KR580",Key::AboutDescription => "Microprocessor system emulator based on the KR580VM80 chip",Key::AboutVersion => "Version 1.0.0",Key::AboutGithubLabel => "GitHub",Key::FileNew => "New file",Key::FileOpen => "Open",Key::FileSave => "Save",Key::FileSaveAs => "Save as",Key::FileImport => "Import",Key::FileExport => "Export",Key::LegacyFormatNote => "legacy format",Key::MpRunProgram => "Run program",Key::MpRunInstruction => "Run instruction",Key::MpRunTact => "Run tact",Key::MpResetRam => "Clear RAM",Key::MpResetCpu => "Clear registers",Key::MpClearHalt => "Clear HLT flag",Key::DiscardCancel => "Cancel",Key::DiscardBody => "Unsaved changes will be lost.",Key::DiscardTitleOpen => "Open file",Key::DiscardTitleNew => "New file",Key::DiscardTitleImport => "Import",Key::DiscardTitleClose => "Close application",Key::DiscardConfirmOpen => "Open",Key::DiscardConfirmNew => "Create",Key::DiscardConfirmImport => "Import",Key::DiscardConfirmClose => "Close",Key::StatusReady => "Ready",Key::StatusNewFile => "New file",Key::StatusCpuHalted => "CPU halted",Key::StatusStopped => "Stopped",Key::StatusTact => "Tact",Key::StatusCycle => "cycle",Key::StatusOpened => "Opened",Key::StatusSavedTo => "Saved to",Key::StatusExportTo => "Exported to",Key::ErrorPrefix => "Error",Key::LegacyOpenedNotice => "Opened a legacy-format file",Key::HaltNotice => {CPU halted by the HLT instruction\nReset registers or clear the HLT flag