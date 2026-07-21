Transfer instructions normally leave flags unchanged.

**Registers and immediate data**
• MOV dst,src – transfer among A, B, C, D, E, H, L, and M
• MVI dst,d8 – load an 8-bit value
• LXI rp,d16 – load BC, DE, HL, or SP

**Memory**
• LDA/STA a16 – read or write A at a direct address
• LHLD/SHLD a16 – read or write HL at two consecutive addresses
• LDAX/STAX B|D – read or write A through BC or DE

**Exchange**
• XCHG – exchange HL and DE
• XTHL – exchange HL with two bytes at the stack top
• SPHL – copy HL into SP
