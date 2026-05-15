# Instruction timing

`InstructionTiming` carries:

```rust
pub struct InstructionTiming {
    pub t_states_taken: u8,
    pub t_states_not_taken: u8,
    pub machine_cycle_count: Option<u8>,
}
```

For unconditional instructions both T-state fields are equal. For
conditional control-flow instructions they differ:

| Family            | taken | not taken | notes                                |
| ----------------- |:-----:|:---------:| ------------------------------------ |
| `Jcc a16`         | 10    | 10        | both branches are 3-byte fetches     |
| `Ccc a16`         | 17    | 11        | matches Intel data sheet             |
| `Rcc`             | 11    | 5         | matches Intel data sheet             |

Other reference timings are codified in `decode::opcode_timing`:

* `MOV r,r' = 5`, `MOV r,M / MOV M,r = 7`
* `ALU r/M = 4 / 7`
* `LXI rp,d16 = 10`
* `LHLD/SHLD = 16`, `LDA/STA = 13`
* `XTHL = 18`, `XCHG = 5`, `SPHL/PCHL = 5`
* `IN/OUT = 10`
* `PUSH = 11`, `POP = 10`, `RST = 11`
* `EI/DI = 4`, rotates / DAA / CMA / STC / CMC = 4

The full reference table is the prompt + the dispatch tests in
`execute/control.rs::tests::cc_taken_uses_taken_timing` and
`cc_not_taken_uses_not_taken_timing`.

## RunForTStates

`Cpu8080State::run_for_t_states(target)` is **exact** at the granularity of
a single instruction: it executes whole instructions until the consumed
T-state count reaches or exceeds `target`. It may overshoot by the cost of
the final instruction. This matches the prompt requirement that the
quantum be T-state-defined, not a wall-clock duration.
