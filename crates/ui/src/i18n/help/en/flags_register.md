The Flags Register (F) records result attributes. Together with A, it forms the PSW (Program Status Word).

Format (8 bits, 5 used):
Bit 7 (S) - Sign = MSB of result
Bit 6 (Z) - Zero = 1 if result is 0
Bit 5 - always 0
Bit 4 (AC) - Auxiliary Carry from bit 3 to bit 4
Bit 3 - always 0
Bit 2 (P) - Parity = 1 if even number of 1-bits
Bit 1 - always 1
Bit 0 (CY) - Carry from MSB

Flag updates:
- S, Z, P - updated by most instructions
- AC - arithmetic and DAA only
- CY - arithmetic, shifts, STC, CMC

Key flags:
- CY: enables multi-byte arithmetic on 8-bit CPU
- Z: loops and branching
- S: sign-based branching

On the schematic, flags show as colored indicators: green = 1, grey = 0.