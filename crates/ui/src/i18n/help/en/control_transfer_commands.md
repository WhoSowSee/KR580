Control transfer instructions modify PC for branching, loops, and subroutines.

- JMP addr - unconditional jump (3 bytes, 10 tacts)
- Jcond addr - conditional jump (3 bytes, 10 tacts). 8 conditions: JNZ, JZ, JNC, JC, JPO, JPE, JP, JM
- CALL addr - subroutine call (3 bytes, 17 tacts): SP-=2, [SP]=PC, PC=addr
- Ccond addr - conditional call (3 bytes, 11/17 tacts). Same 8 conditions.
- RET - return (1 byte, 10 tacts): PC=[SP], SP+=2
- Rcond - conditional return (1 byte, 5/11 tacts). 8 conditions.
- RST n - restart (1 byte, 11 tacts): short call to n*8 (00h, 08h, ..., 38h)
- PCHL - PC = HL (1 byte, 5 tacts). Indirect jump.

In the emulator, PC is shown on the schematic and highlights the current instruction in the RAM table.