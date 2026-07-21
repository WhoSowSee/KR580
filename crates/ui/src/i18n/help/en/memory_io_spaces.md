**RAM**
All 65,536 bytes from 0000h through FFFFh are available. M in a mnemonic means the cell addressed by HL. PC marks the next instruction and SP marks the stack top.

**I/O ports**
• 00h – monitor
• 01h – floppy
• 02h – HDD
• 03h – network adapter
• 04h – printer

IN and OUT use an 8-bit port number. Accessing any other port stops with an invalid-port error.

**Stack area**
The View menu or Ctrl+Shift+C switches the RAM list to FF00h–FFFFh. Repeat the action or press Esc to return to the normal list and its previous position.
