Stack instructions save and restore 16-bit values via the stack. Stack grows downward.

- PUSH rp - save pair (1 byte, 11 tacts): SP-=1, [SP]=high; SP-=1, [SP]=low. rp in {BC, DE, HL, PSW}
- POP rp - restore pair (1 byte, 10 tacts): low=[SP], SP+=1; high=[SP], SP+=1

Notes:
- PUSH PSW saves A and F to stack
- POP PSW restores A and F (can change all flags)
- Stack depth is limited only by RAM size
- In the emulator, the stack area is highlighted yellow in the RAM table (cells with address >= SP)