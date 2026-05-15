# Limitations and assumptions

This is a deliberate, scoped first iteration. Items below are either
out-of-scope by the prompt or explicitly deferred. Each entry is a place
for a future PR to extend the implementation without rewriting it.

## Not implemented

* **`.xlsx` and `.docx` exporters.** The `ExportView` source-of-truth and
  the `TxtExporter` are in place. Adding spreadsheet / document writers is
  additive: pick a crate, implement the trait, reuse the view. The prompt
  requires that they exist eventually but the first iteration delivers
  `.txt` only and the typed contract.
* **External 8080 reference suites.** The CPU is structured for it
  (`run_until_halt`, deterministic IO bus, `RecordingIoBus`), but the
  binaries themselves (`CPUDIAG.BIN`, `8080EXM.COM`, …) are not vendored.
* **`step_tact` opcode-accurate decomposition.** `step_tact` advances the
  cycle counter and a phase counter; it does not yet model per-opcode
  T-state pipelines. Devices observe IO at instruction boundaries, so
  this is purely a debug nicety.
* **Multi-byte interrupt acknowledge sequences.** Per the prompt, only
  single-byte vectors (`RST n`) are honoured. Other vector bytes are
  consumed and charged 4 T-states without further effect.

## Documented behavioural choices

* **Monitor byte protocol.** The prompt sets the contract (text vs
  graphics, command byte selects mode and color/intensity, debug surface
  separate from authoritative state) but leaves the exact byte layout
  open. The chosen layout is documented in
  [`docs/devices/monitor.md`](./devices/monitor.md) and tested.
* **`DAA` after subtraction is undefined.** The implementation handles
  the documented post-addition path only, as the prompt explicitly
  forbids treating post-subtraction `DAA` as a compatibility target.

## Standard 8080 choices used in ambiguous spots

When the prompt referred to "standard 8080 / KR580 semantics" without
restating exact bit-level rules, the implementation matched the Intel
8080 data sheet:

* AC for ADD/ADC = carry-out of bit 3.
* AC for SUB/SBB = `(a & 0xF) >= (b & 0xF) + cy_in`.
* INR sets AC on `0xF → 0x0`. DCR clears AC iff `(result & 0xF) != 0xF`.
* `CALL` always pushes the return address; `Ccc` only when the condition
  holds; both consume their immediate before branching.
* `RST n` pushes the address of the *byte after* the opcode (`PC` after
  fetch) per the data sheet.
* `XTHL` swaps HL with the *top* of the stack (low byte at `[SP]`, high
  byte at `[SP+1]`).

Each choice has at least one regression test.

## CI environment

The repository does not yet ship a CI configuration. The local commands
that should pass on every change are:

```text
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
