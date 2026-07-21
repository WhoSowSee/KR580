**1. Prepare memory**
Press Ctrl+N for a new session or Ctrl+O to open a .580 file. Select a RAM address, enter a byte in the value field, and press Enter. Press E to open the opcode list. Pasting space-separated hexadecimal pairs writes a block from the selected address.

**2. Set the initial state**
Set PC to the first instruction address. Set SP and other registers from the right panel or directly on the structural diagram when needed.

**3. Execute**
Ctrl+T runs one instruction, Ctrl+Y runs one T-state, and Ctrl+R starts or stops continuous execution. Clear HLT before continuing after a halt.

**4. Inspect the result**
Watch PC, registers, flags, the selected RAM cell, and device windows. Ctrl+Z reverses the latest manual change.
