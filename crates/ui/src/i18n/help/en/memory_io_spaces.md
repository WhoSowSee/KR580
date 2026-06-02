Memory address space: 64 KB (0000h-FFFFh), formed by 16-bit address bus A0-A15.

Typical layout:
- 0000h-3FFFh - ROM area (16 KB, in emulator all 64 KB is usable RAM)
- 4000h-FFFFh - RAM area (48 KB)

In the emulator, all 64 KB is RAM initialized to 00h. Programs can be loaded at any address.

I/O space: 256 ports (00h-FFh). 8-bit address duplicated on A0-A7 and A8-A15. Accessible via IN/OUT instructions.

Port assignments:
- Monitor port - character and pixel output
- Floppy/HDD ports - storage device I/O
- Network adapter ports - network communication
- Printer port - print output

Stack: RAM region addressed via SP. Grows downward (high to low addresses). Power-on: SP = 0000h (requires program initialization).