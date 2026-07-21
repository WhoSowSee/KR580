**Find an address by hex fragment**
Enter part of a four-digit address in the address field and press Ctrl+Enter. Search moves forward and wraps within the current area; Ctrl+Shift+Enter searches backward. Repeating the shortcut continues with the same fragment.

For example, FF matches 00FFh, 01FFh, and later addresses. Editing the address field starts a new search.

**Direct jump**
Enter a complete address and press Alt+Enter. Alt+Q jumps to 0000h and Alt+E jumps to FFFFh.

**Following an address operand**
Select either byte of a 16-bit address operand and press Alt+Enter. The list moves to the target. Alt+Shift+Enter restores the source cell and the previous scroll position.

**Opening a device**
Alt+Enter on an IN or OUT port byte opens the device for ports 00h–04h. It opens nothing on an opcode, ordinary 8-bit data operand, or unknown port.

Enable address, data, and port operand highlighting under Settings → General.
