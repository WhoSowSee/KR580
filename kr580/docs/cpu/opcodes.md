# Opcode coverage

Every documented 8080 opcode has a defined size, flag behaviour, PC effect,
cycle metadata, and test coverage. The full reference matrix lives in the
prompt at `prompt/opcode_dispatch.md` — the file is the source of truth for
which slots are documented vs. undocumented.

## Documented slots

The dispatcher in `kr580-core/src/execute.rs` covers these families:

| Family            | Slots                                              | Module                |
| ----------------- | -------------------------------------------------- | --------------------- |
| Data transfer     | `MOV`, `MVI`, `LXI`, `LDA`, `STA`, `LDAX`, `STAX`, `LHLD`, `SHLD`, `XCHG`, `XTHL`, `SPHL` | `execute/data.rs`     |
| Increment / decr. | `INR r/M`, `DCR r/M`, `INX`, `DCX`                 | `execute/alu.rs` + `execute/data.rs` |
| Arithmetic        | `ADD`, `ADC`, `SUB`, `SBB`, `CMP` (reg/M/immediate)| `execute/alu.rs`      |
| Logic             | `ANA`, `XRA`, `ORA` (reg/M/immediate)              | `execute/logic.rs`    |
| Rotates / flags   | `RLC`, `RRC`, `RAL`, `RAR`, `CMA`, `STC`, `CMC`, `DAA` | `execute/rotates.rs`, `execute/misc.rs` |
| Stack             | `PUSH`, `POP` (BC, DE, HL, PSW)                    | `execute/stack.rs`    |
| Control           | `JMP`, `Jcc`, `CALL`, `Ccc`, `RET`, `Rcc`, `RST n`, `PCHL` | `execute/control.rs` |
| 16-bit add        | `DAD rp`                                           | `execute/data.rs`     |
| Misc / IO / IRQ   | `HLT`, `EI`, `DI`, `IN`, `OUT`                     | `execute/misc.rs`     |

## Undocumented slots

`08 10 18 20 28 30 38 CB D9 DD ED FD` raise
`DecodeError::UndocumentedOpcode(op)` and set `halted = true`. They are
**not** NOP / JMP / CALL / RET aliases.

`every_undocumented_slot_raises_decode_error` (in
`crates/kr580-integration-tests/tests/opcode_coverage.rs`) verifies this
against the entire prompt list.

## Coverage tests

* `crates/kr580-integration-tests/tests/opcode_coverage.rs`
  * `every_documented_opcode_executes` — runs each documented opcode under a
    fresh state and asserts no decode error.
  * `every_undocumented_slot_raises_decode_error` — asserts the list above
    halts with the right error.
* Family-specific semantic tests live in the unit-test modules of each
  `execute/*.rs` file.
