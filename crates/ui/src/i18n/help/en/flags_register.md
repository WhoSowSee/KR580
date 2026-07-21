The F register stores result conditions. Together with A, it forms the PSW.

**F bits**
• S, bit 7 – sign
• Z, bit 6 – zero result
• AC, bit 4 – auxiliary carry
• P, bit 2 – even parity
• CY, bit 0 – carry or borrow
• Bit 1 is always 1; bits 3 and 5 are 0

Arithmetic, logic, and compare instructions update flags using 8080 rules. INR/DCR preserve CY, DAD changes only CY, ANA sets AC according to the 8080 rule, and ORA/XRA clear AC and CY. Rotates change only CY; CMA changes no flags.

The diagram displays active and inactive flags using colours from the selected theme.
