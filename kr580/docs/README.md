# KR580 Modern Desktop Emulator — Documentation

This `docs/` tree mirrors the runtime architecture and is meant to be read
alongside the source. Each subfolder maps to one workspace crate.

## Map

- [`architecture.md`](./architecture.md) — workspace structure, runtime model, threading.
- [`cpu/`](./cpu/) — pure CPU core: state, decoding, execution, flags, interrupts, timing.
- [`persistence/`](./persistence/) — `.580` snapshot format, `.krs` subprograms, settings, exporters.
- [`devices/`](./devices/) — IoBus routing and the asynchronous peripheral devices.
- [`ui/`](./ui/) — iced view layer and runtime wiring.
- [`testing.md`](./testing.md) — test layout and how to run them.
- [`limitations.md`](./limitations.md) — current limitations and assumptions made under the prompt.

## Source of Truth

Everything in this implementation traces back to the prompt files in
`prompt/`. The implementation does *not* depend on `spec/` or any
reverse-engineering material.

## Quick Start

```text
cargo build --workspace
cargo test  --workspace
cargo run   -p kr580-app
```

The binary launches the iced UI with the deterministic core actor running on
its own thread and a Tokio runtime hosting the device workers.
