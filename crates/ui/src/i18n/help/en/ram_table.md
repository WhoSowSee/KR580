The RAM contents table is in the right panel. Three columns:

1. Address - cell address in hex (0000h-FFFFh)
2. Value - current cell value (00h-FFh)
3. Command - disassembly into assembly mnemonic

Color highlights:
- Blue - selected cell (cursor)
- Green - PC (program counter) - moves during execution
- Brown - SP (stack pointer)
- Yellow - all cells with address >= SP (stack area)

Select a row (mouse or arrows) to edit that cell. The '...' button opens the opcode picker with all 244 instructions. Search by hex code or mnemonic, move the highlighted result with arrow keys or Tab/Shift+Tab, then press Enter to write that opcode to the selected cell.
