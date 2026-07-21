**8-bit registers**
• A – accumulator used by arithmetic and logical operations
• B, C, D, E, H, L – general-purpose registers
• BC, DE, and HL form 16-bit pairs; M means the RAM cell addressed by HL

**16-bit registers**
• PC – address of the next instruction
• SP – address of the stack top; the stack grows toward lower addresses

**Internal values**
W and Z expose the processor's temporary pair. They appear in the diagram and exports but are not addressed directly by program instructions.

Processor reset clears the registers, PC, and SP. The flags register retains its required bit 1 layout.
