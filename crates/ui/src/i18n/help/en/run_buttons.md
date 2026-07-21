**Execution**
• Ctrl+R – start or stop continuous execution
• Ctrl+T – execute one instruction
• Ctrl+Y – execute one T-state of the current instruction

Choose 5, 20, 120, or 1000 instructions per second on the left. When Follow PC is enabled, the RAM list follows the next instruction at execution boundaries.

**Reset**
• Ctrl+Shift+R – fill RAM with zeroes
• Ctrl+Shift+G – reset registers, flags, PC, SP, interrupts, HLT, and timing
• Ctrl+Shift+H – clear only HLT

After HLT, run and step actions remain blocked until an accepted interrupt, processor reset, or Clear HLT. Manual RAM and processor resets can be undone with Ctrl+Z.
