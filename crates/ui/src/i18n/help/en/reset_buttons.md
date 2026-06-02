Reset button group (right panel, bottom row):

- Reset RAM (red, Ctrl+Shift+R) - fills all 64 KB with 00h. Undoable (Ctrl+Z).
- Reset Registers (purple, Ctrl+Shift+G) - clears A, F(02h), B-H, SP, PC to zero. Undoable.

Both also accessible via MP-System menu.

Clear HLT flag (Ctrl+Shift+H) - removes execution block after HLT without affecting memory or registers.