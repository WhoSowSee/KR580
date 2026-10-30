**Execution**
• Ctrl+R – start or stop continuous execution
• Ctrl+T – execute one instruction
• Ctrl+Y – execute one T-state of the current instruction

Choose paced execution at 5, 20, or 120 instructions per second, or Max. Max runs instruction batches with a 16 ms time budget; the displayed 1000 is a nominal tier label, not a throughput limit.

With Follow PC enabled, the RAM list shows the current execution position. At higher speeds, intermediate states may be coalesced.

**Reset**
• Ctrl+Shift+R – fill RAM with zeroes
• Ctrl+Shift+G – reset registers, flags, PC, SP, interrupts, HLT, and timing
• Ctrl+Shift+H – clear only HLT

After HLT, run and step actions remain blocked until an accepted interrupt, processor reset, or Clear HLT. Manual RAM and processor resets can be undone with Ctrl+Z.
