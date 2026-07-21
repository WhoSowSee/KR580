Logical instructions use accumulator A and update S, Z, and P from the result.

• ANA/ANI – bitwise AND; clears CY and sets AC by the 8080 rule
• ORA/ORI – bitwise OR; clears AC and CY
• XRA/XRI – bitwise XOR; clears AC and CY
• CMP/CPI – compare through subtraction without changing A
• CMA – invert A without changing flags
• STC/CMC – set or complement CY
• RLC/RRC – circular rotate A
• RAL/RAR – rotate A through CY
• DAA – decimal-adjust A after BCD addition

Rotates change only CY. For CMP/CPI, Z=1 means equal and CY=1 means A is less than the operand.
