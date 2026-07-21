The stack grows toward lower addresses. Set SP to usable RAM before PUSH, CALL, or interrupts.

**PUSH rp**
Decrements SP by two and stores BC, DE, HL, or PSW. For PSW, the high byte is A and the low byte is F.

**POP rp**
Restores a pair from two bytes at SP and increments SP by two. POP PSW restores A and flags while preserving the required F layout.

CALL, RET, conditional calls/returns, RST, and accepted interrupts use the same stack. The View menu can temporarily show FF00h–FFFFh for inspecting high memory.
