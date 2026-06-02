Emulator file formats:

- .kr580 - primary binary format. Contains:
Format version (for backward compatibility), full 64 KB RAM image, all register states (A, F, B-H, SP, PC), flag states, all device states, metadata

- .sav - legacy format from the original emulator:
Limited compatibility. Contains RAM image and register values.

- Import files - any binary file (.bin, .com, .hex):
Loaded as-is at the specified address.
Allows loading compiled programs from external assemblers.

Undo/Redo (Ctrl+Z / Ctrl+Shift+Z):
- History depth: 100 steps
- Tracked: register edits, memory edits, reset, import
- Consecutive edits of the same field are coalesced