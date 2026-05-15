# CPU Core (`kr580-core`)

A pure, deterministic Intel 8080 / KR580VM80 CPU model.

## Files

| File                           | Responsibility                                        |
| ------------------------------ | ----------------------------------------------------- |
| `state.rs`                     | `Cpu8080State`, `Reg8`, `RegPair`, helpers            |
| `flags.rs`                     | `Flags` + PSW packing / parity                        |
| `memory.rs`                    | Flat 64 KiB RAM with a serde-friendly representation  |
| `timing.rs`                    | `InstructionTiming` (taken / not-taken / m-cycles)    |
| `io.rs`                        | `IoBus` trait, `NullIoBus`, `RecordingIoBus`          |
| `decode.rs`                    | Condition codes, register-pair decoding, undocumented opcode list, base timing table |
| `interrupt.rs`                 | Interrupt acknowledge: `EI` deferral, halt exit, RST vectoring |
| `commands.rs`                  | `CoreCommand` / `CoreEvent` types for the actor       |
| `execute.rs`                   | Top-level dispatch: fetch / decode / dispatch / accounting |
| `execute/alu.rs`               | ADD / ADC / SUB / SBB / CMP / INR / DCR / immediate variants |
| `execute/logic.rs`             | ANA / XRA / ORA / immediate variants                  |
| `execute/data.rs`              | MOV / MVI / LXI / LDA / STA / LHLD / SHLD / LDAX / STAX / XCHG / XTHL / SPHL / DAD / INX / DCX |
| `execute/rotates.rs`           | RLC / RRC / RAL / RAR                                 |
| `execute/stack.rs`             | PUSH / POP (BC, DE, HL, PSW)                          |
| `execute/control.rs`           | JMP / Jcc / CALL / Ccc / RET / Rcc / RST / PCHL       |
| `execute/misc.rs`              | DAA / CMA / STC / CMC / HLT / EI / DI / IN / OUT      |

## Public API

```rust
let mut cpu = Cpu8080State::new();
let mut bus = NullIoBus;
cpu.step_instruction(&mut bus)?;          // one instruction
cpu.run_for_t_states(&mut bus, 1_000)?;   // exact T-state quantum
cpu.run_until_halt(&mut bus, 1_000_000)?; // bounded loop
cpu.step_tact()?;                         // single T-state, debug only
```

## Semantic notes

The implementation strictly follows the prompt:

* AC for SUB/SBB/CMP uses the 8080 subtractor model
  `AC = (a & 0xF) >= ((b & 0xF) + cy_in)`. `1 - 0` sets AC, `0 - 1` clears AC.
* AC for ADD/ADC is the carry out of bit 3.
* INR sets AC on `0xF → 0x0`; DCR sets AC iff `(result & 0xF) != 0xF`.
* ANA clears CY and sets AC = `bit3(A) | bit3(operand)` *before* the AND.
* ORA / XRA clear both CY and AC.
* Rotates touch only CY and the rotated bits.
* DAD touches only CY.
* DAA implements only the documented post-addition behavior.
* PSW packs `S Z 0 AC 0 P 1 CY`. POP PSW masks bits 3, 5, forces bit 1 = 1.
* EI defers enable to the next instruction boundary; DI is immediate.
* HLT is exited by reset or by an accepted interrupt while IE = 1.
* The undocumented opcodes
  `08 10 18 20 28 30 38 CB D9 DD ED FD` raise `DecodeError::UndocumentedOpcode`
  and *halt* the CPU.

## Files: see also

* [`flags.md`](./flags.md) — full flag rule table.
* [`opcodes.md`](./opcodes.md) — opcode coverage matrix.
* [`timing.md`](./timing.md) — T-state table (taken / not-taken).
* [`interrupts.md`](./interrupts.md) — interrupt latch + RST vectoring.
