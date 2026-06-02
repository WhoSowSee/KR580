The Command Panel (left side of schematic) shows execution state:

- Machine cycle number (M1-M5) - current instruction step
- Tact number within cycle (T1-T5)
- Phase - same as instruction tact, zero-based
- Total tact counter - since session start
- Instruction tact - tact within current instruction (T1, T2, ...)

Micro-operation indicators show current tact actions: memory read, write, ALU operation, etc.

The instruction decoder displays the opcode, mnemonic, type, and addressing mode.