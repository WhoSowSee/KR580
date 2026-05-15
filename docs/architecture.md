# Architecture

This workspace implements a layered KR580/Intel 8080 desktop emulator using only the `prompt/` documents as product source.

## Crates

- `k580-core`: deterministic CPU state, memory, flags, opcode decode/execute, timing, interrupts, typed command/event contracts, and the `PortBus` trait.
- `k580-devices`: `IoBus` for ports `00h..04h`, monitor, floppy, HDD, network, printer, device states, and non-blocking worker queues.
- `k580-persistence`: versioned `.580` snapshots, raw `.krs` subprograms, JSON settings, and direct `.txt`/`.xlsx`/`.docx` exporters.
- `k580-app`: application orchestration, crossbeam command/event actor, top-level dependency wiring, and file/export commands.
- `k580-ui`: iced shell. It renders snapshots and sends commands. It does not own emulator state.

## Data flow

UI messages become `AppCommand` values. The app actor owns `Cpu8080State` and `IoBus`, applies commands, and publishes typed `AppEvent` values. The UI stores only display/input state and can always re-render from `AppSnapshot`.
