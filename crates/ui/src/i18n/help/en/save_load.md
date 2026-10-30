A .580 file stores 64 KB of RAM, registers A, B, C, D, E, H, L, W, Z, and PC and SP. Its size is 65,549 bytes.

Flags, HLT, interrupt state, and timing are absent and take their initial CPU values on load. Device buffers and preferences are not stored in .580 files.

Ctrl+O opens .580 or .krs. For .krs, provide the load address; when saving,
provide the start and end addresses of the RAM range. Ctrl+S saves to the
current path or asks for one on first save. Ctrl+Shift+S always saves under a
new name.

Ctrl+N starts an empty session by zeroing RAM and resetting the processor. Unsaved changes require confirmation before open, new, or close.

Use TXT/XLSX import and export to exchange selected registers, flags, and RAM ranges.
