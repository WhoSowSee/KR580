**Jumps**
• JMP a16 – unconditional jump
• JNZ/JZ, JNC/JC, JPO/JPE, JP/JM – jumps testing Z, CY, P, and S
• PCHL – load PC from HL

**Subroutines**
• CALL a16 – push the return address and jump
• RET – restore PC from the stack
• Conditional calls and returns use the same eight conditions
• RST n – one-byte call to address n×8

Conditional CALL and RET timings differ for taken and not-taken conditions. Set SP to usable RAM before calling subroutines.
