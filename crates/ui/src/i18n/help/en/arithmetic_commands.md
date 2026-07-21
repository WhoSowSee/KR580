**8-bit operations**
• ADD/ADI – A = A + operand
• ADC/ACI – A = A + operand + CY
• SUB/SUI – A = A − operand
• SBB/SBI – A = A − operand − CY
• INR/DCR – change a register or M by one without changing CY

**16-bit operations**
• INX/DCX – change BC, DE, HL, or SP by one without changing flags
• DAD – HL = HL + pair; changes only CY

ADD, ADC, SUB, and SBB update S, Z, AC, P, and CY. Flag values follow KR580VM80A / Intel 8080 rules, including auxiliary carry during subtraction.
