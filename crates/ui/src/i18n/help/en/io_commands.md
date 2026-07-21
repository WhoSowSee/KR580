**IN d8** reads a byte from a port into A. **OUT d8** sends A to a port.

**Port map**
• 00h – monitor: text and graphics commands
• 01h – floppy: write to the attached image; IN returns status
• 02h – HDD: write to hdd.kpd; IN returns status
• 03h – network: OUT transmits a byte; IN pops RX or returns 00h
• 04h – printer: OUT appends a byte to the print spool

The 8080 presents the port number on both halves of the address bus, while the emulator selects the device from the low byte.
