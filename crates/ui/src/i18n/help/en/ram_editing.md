**Selecting an address**
Click a RAM row or enter four hexadecimal digits in the address field. Arrow keys move the selection, PageUp/PageDown scroll, and Alt+Q / Alt+E jump to 0000h / FFFFh.

**Writing bytes**
Enter two hexadecimal digits in the value field and press Enter or the apply button. The editor advances after a write. Pasting space-separated hexadecimal pairs writes a block from the selected cell; a block extending beyond FFFFh is rejected.

**Opcodes**
Press E or “…” to open the 244 documented encodings. Search by hexadecimal code or mnemonic and press Enter to write the selected opcode.

**Operands and stack**
When operand highlighting is enabled, addresses, data, and port numbers use distinct states. Alt+Enter acts on the selected cell; Alt+Shift+Enter returns to the source operand. Ctrl+Shift+C shows or hides FF00h–FFFFh.
