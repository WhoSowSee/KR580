# Architecture

This workspace implements a layered KR580/Intel 8080 desktop emulator using only the `prompt/` documents as product source.

## Crates

- `k580-core`: deterministic CPU state, memory, flags, opcode decode/execute, timing, interrupts, typed command/event contracts, and the `PortBus` trait. Opcode execution is split by instruction family under `ops/`.
- `k580-devices`: `IoBus` for ports `00h..04h`, monitor, floppy, HDD, network, printer, device states, and non-blocking worker queues.
- `k580-persistence`: versioned `.580` snapshots, raw `.krs` subprograms, JSON settings, and direct `.txt`/`.xlsx` exporters/importers.
- `k580-app`: application orchestration, crossbeam command/event actor, top-level dependency wiring, and file/export commands.
- `k580-ui`: iced shell split into app state/update, runtime command/event helpers, view rendering, and a small Windows-only platform shim. It renders snapshots and sends commands. It does not own emulator state.

## Repository layout

- `crates/`: the workspace crates listed above.
- `prompt/`: the implementation source of truth.
- `docs/`: reference documentation (this directory).
- `assets/icons/`: pre-rendered icon set consumed at build and run time. The master `icon.png` lives next to the generated PNG fan-out and the multi-resolution `icon.ico`. See `docs/assets.md`.
- `scripts/`: developer helpers. `generate_icons.ps1` (Windows) and `generate_icons.sh` (Unix/macOS) regenerate `assets/icons/` from the master image.
- `target/`: cargo build artefacts (gitignored).

## Data flow

UI messages become `AppCommand` values. The app actor owns `Cpu8080State` and `IoBus`, applies commands, and publishes typed `AppEvent` values. The UI stores only display/input state and can always re-render from `AppSnapshot`.

## Invariants

- `prompt/` is the source of truth for behavior, file formats, and quality gates.
- CPU state is owned by `k580-core` and the app actor, never by UI widgets.
- Device state is owned by `k580-devices`; `IN`/`OUT` route through `PortBus`.
- Persistence reads from `Cpu8080State` or explicit export view models, never from UI labels or grids.
- `.krs` remains a raw byte slice with caller-provided base address; no secondary subprogram format is introduced.

## Runtime shape

`k580-ui` sends commands through a crossbeam channel to the emulator actor in `k580-app`. The actor applies commands synchronously against the core and bus, then emits state snapshots and typed events. Disk/printer operations are queued to Tokio-backed workers where host I/O is needed, keeping the UI thread away from blocking device work.

## Actor pacing loop

The worker thread in `k580-app::actor::run_worker` does not block on
`recv()`. Instead it uses `crossbeam_channel::select!` to wait
simultaneously on the command channel and a timer:

- **Paused (`!emulator.is_running()`)** — the timer arm is wired to
  `crossbeam_channel::never()`, so the `select!` degenerates to a plain
  command-channel `recv()`. The worker is fully idle until the UI sends
  the next command.
- **Running (`emulator.is_running()`)** — the timer arm is wired to
  `crossbeam_channel::after(emulator.step_interval())`. Whichever arm
  fires first wins: a command interrupts the wait immediately and is
  applied without skipping a beat, and a timer tick calls
  `emulator.tick()`, which advances exactly one instruction and emits
  `InstructionBoundaryReached`, `HaltStateChanged` (if the CPU halted on
  this step), `Stopped` (if the budget exhausted or an error fired), and
  a fresh `StateChanged`.

`AppCommand::Run` only flips `Emulator::running = true` and resets the
per-arming `instructions_since_run` counter. `AppCommand::Stop` clears
the flag. `AppCommand::SetStepInterval(duration)` overwrites
`Emulator::step_interval` (clamped to `MIN_STEP_INTERVAL = 1ms`); the
next `select!` iteration re-arms the timer at the new pace. Defaults:
`DEFAULT_STEP_INTERVAL = 100ms` (10 instructions/sec) and
`MAX_INSTRUCTIONS_PER_RUN = 100_000` instructions per arming before the
worker auto-pauses with `Stopped`.

This decoupling is what makes the UI animation visible: the previous
`Run` implementation called `cpu.run_until_halt(&mut bus, 100_000)`
synchronously inside the worker, which produced exactly one
`StateChanged` after the whole burst — the user only ever saw the
final state. With the selector loop the UI receives one snapshot per
instruction, so the highlighted cell, registers, and PC step through
the program live.
