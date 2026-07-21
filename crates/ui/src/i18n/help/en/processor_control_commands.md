**NOP (00h)**
Changes no processor state except PC and the timing counter.

**HLT (76h)**
Stops execution. Continue after an accepted interrupt, processor reset, or the manual Clear HLT action. Run and step actions show a halt notice while HLT remains active.

**DI (F3h)**
Disables interrupts immediately and cancels a pending enable.

**EI (FBh)**
Enables interrupts after the next instruction boundary. A request received while interrupts are disabled remains pending until accepted.
