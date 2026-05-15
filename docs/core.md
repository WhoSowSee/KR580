# Core CPU

`Cpu8080State` owns registers, flags, `PC`, `SP`, 64 KiB RAM, interrupt state, halt state, total T-state count, and optional tact phase. The UI never mutates these fields directly.

Implemented behavior follows standard Intel 8080/KR580 semantics from `prompt/`:

- all documented opcodes decode and execute;
- undocumented slots from `opcode_dispatch.md` return `DecodeError::UndocumentedOpcode`;
- PSW materialization forces bit 1 to `1` and bits 3/5 to `0`;
- prompt-specific subtract auxiliary-carry behavior is tested (`1-0 => AC=1`, `0-1 => AC=0`);
- conditional branches use normal 8080 meanings;
- `EI` enables interrupts after the following instruction boundary;
- accepted interrupt vectors are modeled as single-byte `RST n` opcodes.

`tact` stepping keeps exact T-state accounting. Architectural instruction effects are committed by the instruction executor; devices are not called at sub-instruction T-state granularity, matching the prompt rule that device effects are instruction-boundary level.
