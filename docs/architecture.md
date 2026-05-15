# Architecture

This workspace implements a layered KR580/Intel 8080 desktop emulator using only the `prompt/` documents as product source.

## Crates

- `k580-core`: deterministic CPU state, memory, flags, opcode decode/execute, timing, interrupts, typed command/event contracts, and the `PortBus` trait. Opcode execution is split by instruction family under `ops/`.
- `k580-devices`: `IoBus` for ports `00h..04h`, monitor, floppy, HDD, network, printer, device states, and non-blocking worker queues.
- `k580-persistence`: versioned `.580` snapshots, raw `.krs` subprograms, JSON settings, and direct `.txt`/`.xlsx`/`.docx` exporters.
- `k580-app`: application orchestration, crossbeam command/event actor, top-level dependency wiring, and file/export commands.
- `k580-ui`: iced shell split into app state/update, runtime command/event helpers, and view rendering. It renders snapshots and sends commands. It does not own emulator state.

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
