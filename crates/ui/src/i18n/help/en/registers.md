Software-accessible registers:

- Accumulator (A, 8-bit) - primary arithmetic/logic register
- Flags (F, 8-bit, 5 used): S (bit 7), Z (bit 6), AC (bit 4), P (bit 2), CY (bit 0)
- GPRs: B, C, D, E, H, L (6 x 8-bit), paired as BC, DE, HL
HL is the primary memory pointer ('M' addressing)
- Stack Pointer (SP, 16-bit) - stack top address
- Program Counter (PC, 16-bit) - next instruction address

Internal (software-inaccessible) registers:
- W, Z - temporary 8-bit registers forming WZ pair
- IR - Instruction Register
- BR1, BR2 - ALU buffer registers
- Address Buffer, Data Buffer - interface registers

All registers are shown in real time on the schematic. Hover + click opens inline editing.