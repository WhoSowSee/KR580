# Testing

Run the same checks from the repository root:

```sh
cargo fmt --all --manifest-path /d/kr-580/Cargo.toml
cargo clippy --workspace --all-targets --manifest-path /d/kr-580/Cargo.toml -- -D warnings
cargo test --workspace --manifest-path /d/kr-580/Cargo.toml
```

## Current coverage

- `k580-core`: opcode classification, documented-opcode smoke execution, modular executor families, flags, conditionals, stack, interrupts, I/O routing, and exact `RunForTStates` accounting.
- `k580-devices`: port routing, invalid-port typed errors, monitor framebuffer/attribute state, storage worker queueing, network no-data handling, Tokio TCP worker roundtrip, and printer spool/export behavior.
- `k580-persistence`: `.580` roundtrip/determinism/header validation, raw `.krs` behavior, settings JSON versioning, and `.txt`/`.xlsx`/`.docx` direct exporters.
- `k580-app`: command-mediated state mutation and actor event publication.

External Intel 8080 binary suites are not included in this workspace. When available, add them as an additional compatibility gate instead of replacing the local semantic tests.
