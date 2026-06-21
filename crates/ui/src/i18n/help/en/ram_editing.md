RAM Cell and Value panel (right side, below the memory table):

- Address field (4 hex digits, 0000-FFFF) with +/- buttons
- Value field (2 hex digits, 00-FF) with +/- buttons
- The value field previews in the selected RAM row immediately, but writes memory only on Enter
- Ctrl+V with space-separated HEX pairs writes a sequence from the selected row even outside edit mode
- Apply button
- '...' button - opcode picker dropdown (all 244 instructions)

Enter in address field jumps to that cell and moves focus to value field.

Opcode picker:
- Search by hex code (e.g., '3E') or mnemonic ('MVI A')
- Arrow keys and Tab/Shift+Tab move the highlighted filtered command
- Enter writes the highlighted opcode to the current cell
- Columns: code, mnemonic, operand, length, kind, addressing
