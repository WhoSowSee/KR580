Saving and loading full emulator state:

- Save (Ctrl+S): quick save to current file. Prompts for path if none selected.
- Save As (Ctrl+Shift+S): always prompts for path.

- Open (Ctrl+O): auto-detects format (.kr580 or .sav). Updates schematic and RAM table.
File path shown in status bar.

- Legacy files are auto-detected by Open and written through Save As when the selected path uses that format.

- New file (Ctrl+N): clears RAM and registers. Warns if unsaved changes exist.

A confirmation dialog appears when opening/creating with unsaved changes.
