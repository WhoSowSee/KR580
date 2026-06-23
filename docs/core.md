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
  The accepted vector also updates `last_fetched_opcode`, so the
  instruction-register mirror is identical after `StepInstruction` and
  after an 11T interrupt-ack tact walk.

`tact.rs` owns the debug T-state walk. It prepares the active opcode,
branch choice, and total T-state count without committing architectural
effects; registers, memory, PC, and device I/O are committed by the same
instruction-boundary executor on the final T-state. This keeps
`StepTact` faithful to the original emulator's one-tact control flow
while preserving the stable `StepInstruction` / `Run` execution path.

## Execution API

- `step_instruction(bus)` executes one instruction boundary or accepts one pending `RST n` interrupt vector.
- `step_tact(bus)` advances exactly one T-state in the debug tact model, keeps `cycle_count` exact, and commits the instruction only on its boundary.
- `run_for_t_states(bus, n)` calls `step_tact` exactly `n` times, so it never overshoots the requested T-state quantum.
- `run_until_halt(bus, max_instructions)` executes instruction boundaries until `HLT` or the explicit safety cap.

## Executor layout

`execute.rs` now owns instruction-boundary orchestration only. Family-specific execution lives in:

- `ops/data.rs` for MOV/MVI/register-pair/memory transfer instructions;
- `ops/alu.rs` for arithmetic, logic, INR/DCR, and immediate ALU instructions;
- `ops/control.rs` for jumps, calls, returns, `RST`, and `PCHL`;
- `ops/stack.rs` for PUSH/POP and PSW stack handling;
- `ops/misc.rs` for NOP, rotates, DAA, flag toggles, EI/DI, and IN/OUT.

## Machine-cycle layout

`machine_cycle/` ships the schoolbook M-cycle / T-phase tables that the UI uses to mirror the reference KR-580 panel. Split into focused submodules to stay under the 400-line per-file budget:

- `machine_cycle/mod.rs` – public types: `MachineCycleLengths`, `MachineCycleLayout`, `MachineCyclePosition`, `MachineCycleKind`, `MachineCycleKinds`, plus `position_for` and the `status_byte()` / `label_ru()` helpers on `MachineCycleKind`.
- `machine_cycle/tables.rs` – opcode → M-cycle layout (`layout_for`) and opcode → M-cycle types (`kinds_for` / `kind_at`). For conditional instructions both `taken` and `not_taken` branches are covered; HLT layout is `[4]` (only the visible M1, school convention) while `decode.rs` keeps the datasheet 7T total.
- `machine_cycle/tests.rs` – invariants pinning the tables to `decode.rs` timing for all 244 documented opcodes and to the Intel 8080A datasheet status-byte raster.
- `tests/tact_execution.rs` – regression tests proving that partial
  tact walks do not advance PC or fire I/O early, while a complete tact
  walk reaches the same CPU/device result as `step_instruction`,
  including interrupt-acknowledge vectors.

## Tested opcode areas

The semantic test suite now covers:

- full opcode classification for all 256 byte values;
- smoke execution for every documented opcode from a controlled CPU state;
- ADD/ADC/SUB/SBB/CMP/INR/DCR/ANA/ORA/XRA flag edge cases;
- `DAA`, PSW reserved-bit normalization, stack roundtrips, rotate/carry operations;
- `SHLD`, `LHLD`, `XCHG`, `XTHL`, `CALL`, `RET`, conditional branch/call timing;
- `IN`/`OUT` bus routing and EI/DI/HLT interrupt acceptance behavior.

External Intel 8080 binary suites are still a recommended next gate when the binaries are available.
