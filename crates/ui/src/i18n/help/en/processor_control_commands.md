Processor control instructions modify CPU operating mode.

- NOP (00h) - no operation (1 byte, 4 tacts). Useful for delays and placeholders.
- HLT (76h) - halt CPU (1 byte, 7 tacts). CPU stops until interrupt or reset. In the emulator, a notification appears, Run is blocked until registers/HLT flag are reset, and the RAM row stays on the 76h byte. Manually toggling the HLT indicator changes only the halt flag and does not move the selected memory cell.
- DI (F3h) - disable interrupts (1 byte, 4 tacts). Clears INTE immediately.
- EI (FBh) - enable interrupts (1 byte, 4 tacts). Sets INTE after next instruction.

Note: interrupts are not implemented in the current emulator version. DI/EI execute for compatibility but have no visible effect.
