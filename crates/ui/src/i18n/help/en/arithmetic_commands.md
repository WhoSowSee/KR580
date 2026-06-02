Arithmetic instructions: addition, subtraction, increment, decrement. Most affect S, Z, AC, P, CY.

- ADD r / ADI data - A = A + operand
- ADC r / ACI data - A = A + operand + CY (add with carry)
- SUB r / SUI data - A = A - operand
- SBB r / SBI data - A = A - operand - CY (subtract with borrow)
- INR r - increment register or M (does not affect CY)
- DCR r - decrement register or M (does not affect CY)
- INX rp - increment 16-bit pair (no flags affected)
- DCX rp - decrement 16-bit pair (no flags affected)
- DAD rp - HL = HL + rp (16-bit add, only CY affected)
- DAA - Decimal Adjust Accumulator for BCD arithmetic

All results are immediately shown on the schematic.