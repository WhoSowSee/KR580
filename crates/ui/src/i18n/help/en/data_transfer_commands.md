Data transfer instructions move data between registers, memory, and I/O ports. Most do not affect flags.

- MOV r1,r2 - register-to-register (1 byte, 5 tacts). MOV M,M does not exist.
- MVI r,data - load immediate (2 bytes, 7 tacts). Example: MVI A,42h
- LXI rp,data16 - load 16-bit immediate to pair (3 bytes, 10 tacts)
- LDA addr - load A from memory (3 bytes, 13 tacts)
- STA addr - store A to memory (3 bytes, 13 tacts)
- LHLD addr - load HL from memory (3 bytes, 16 tacts)
- SHLD addr - store HL to memory (3 bytes, 16 tacts)
- LDAX rp - load A via BC or DE (1 byte, 7 tacts). Indirect addressing.
- STAX rp - store A via BC or DE (1 byte, 7 tacts)
- XCHG - exchange HL <-> DE (1 byte, 4 tacts)
- XTHL - exchange HL with stack top (1 byte, 18 tacts)
- SPHL - load SP from HL (1 byte, 5 tacts)
- PCHL - load PC from HL (1 byte, 5 tacts). Indirect jump.