Emulator file formats:

**.580** - primary binary format. Contains the format version, full 64 KB RAM image, all registers (A, F, B-H, SP, PC), flags, device states, and metadata.

**Legacy format** (.580, different internal format) - compatibility path for files from the original emulator. Available from the File menu as "legacy format".

**Import formats** - TXT and XLSX files created by Export. XLSX imports the selected worksheet. TXT imports the whole file or a selected named section.

**Undo/Redo** - Ctrl+Z / Ctrl+Shift+Z, 256 history entries. Register edits, memory edits, and resets are tracked. Successful import clears the undo history.
