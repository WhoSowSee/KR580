Logical instructions perform bitwise operations on A and an operand.

- ANA r / ANI data - bitwise AND (CY=0, AC=1). Example: ANI 0Fh masks lower nibble.
- ORA r / ORI data - bitwise OR (CY=0, AC=0)
- XRA r / XRI data - bitwise XOR (CY=0, AC=0). XRA A clears A and CY (fast, 1 byte, 4 tacts).
- CMP r / CPI data - compare: A - operand (flags only). Z=1 if equal, CY=1 if A < operand.
- CMA - complement A (flags unchanged)
- STC - set CY=1. CMC - complement CY.
- RLC / RRC - rotate A left/right. Bit 7->CY and 7->0 (RLC); 0->CY and 0->7 (RRC). Only CY affected.
- RAL / RAR - rotate A through CY. 9-bit ring shift: A (8 bits) + CY (1 bit).
- DAA - decimal adjust A after addition (BCD correction).