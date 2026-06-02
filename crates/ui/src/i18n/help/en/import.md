Import Subprogram (Ctrl+I) loads file contents into the current session without resetting memory or registers.

- File dialog opens
- After selecting a file, you're prompted for the target address
- File contents are written to RAM starting at the specified address
- The rest of RAM and registers remain unchanged
- Allows composing a program from multiple parts (e.g., main program + library)
- Undoable (Ctrl+Z)