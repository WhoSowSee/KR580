# Testing

The repository ships **76** tests at the time of writing, all passing on
`cargo test --workspace`.

## Where tests live

| Layer           | Location                                                | Count |
| --------------- | ------------------------------------------------------- | ----- |
| Core            | `crates/kr580-core/src/**/tests.rs` (inline `#[cfg(test)] mod tests`) | 47 |
| Persistence     | `crates/kr580-persistence/src/**/tests.rs`              | 12    |
| Devices         | `crates/kr580-devices/src/**/tests.rs`                  | 9     |
| UI              | `crates/kr580-ui/src/runtime.rs`                        | 1     |
| Integration     | `crates/kr580-integration-tests/tests/*.rs`             | 7     |

Run everything:

```text
cargo test --workspace
```

Run only one crate:

```text
cargo test -p kr580-core
```

## Coverage highlights

* **Every documented opcode** is exercised by
  `tests/opcode_coverage.rs::every_documented_opcode_executes`.
* **Every undocumented slot** is asserted to raise a decode error and halt
  the CPU by `tests/opcode_coverage.rs::every_undocumented_slot_raises_decode_error`.
* **Snapshot round-trip** is verified end-to-end against a tiny program
  that runs to halt, then restores deterministically.
* **Device routing** verifies that `OUT` instructions executed by the core
  reach the right device through the bus.
* **Async devices** have `#[tokio::test]` round-trips for the storage and
  network workers.
* **Interrupts** test `EI` deferral, masked-IRQ pending behaviour, RST
  vectoring on accept, and HLT exit.
* **Flag invariance** is tested: rotates / STC / CMC / CMA, INR/DCR not
  touching CY, DAD touching only CY.
* **AC corner cases** required by the prompt (`1 - 0` sets AC, `0 - 1`
  clears AC under SUB) are explicit tests.

## External KCP tests

The prompt suggests running external 8080 reference suites
(`CPUDIAG.BIN`, `8080EXM.COM`, `TST8080.COM`, `8080PRE.COM`,
`CPUTEST.COM`). The current build does not vendor those binaries — they
are not redistributable in this repository. The architecture supports
running them via `Memory64K::load_at` plus `run_until_halt`; see
[`limitations.md`](./limitations.md).

## CI signals

`cargo fmt --check`, `cargo clippy -- -D warnings`, and
`cargo test --workspace` all return zero on a clean checkout.
