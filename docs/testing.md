# Testing

Run the same checks from the repository root:

```sh
cargo fmt --all --manifest-path /d/kr-580/Cargo.toml
cargo clippy --workspace --all-targets --manifest-path /d/kr-580/Cargo.toml -- -D warnings
cargo test --workspace --manifest-path /d/kr-580/Cargo.toml
```

## Current coverage

- `k580-core`: opcode classification, documented-opcode smoke execution,
  modular executor families, flags, conditionals, stack, interrupts, I/O
  routing, and exact `RunForTStates` accounting.
- `k580-devices`: port routing, invalid-port typed errors, monitor
  framebuffer/attribute state, storage worker queueing, network no-data
  handling, Tokio TCP worker roundtrip, and printer spool/export
  behavior.
- `k580-persistence`: `.580` roundtrip/determinism/header validation,
  raw `.krs` behavior, settings JSON versioning, and `.txt`/`.xlsx`/
  `.docx` direct exporters.
- `k580-app`: command-mediated state mutation and actor event
  publication.

External Intel 8080 binary suites are not included in this workspace.
When available, add them as an additional compatibility gate instead of
replacing the local semantic tests.

## Asset prerequisites

The build pipeline embeds `assets/icons/icon-64.png` (runtime window
icon) and, on Windows, `assets/icons/icon.ico` (PE resource). Both files
are checked in. If you replace `assets/icons/icon.png` (the master), run
the matching script before rebuilding so the embedded artefacts stay in
sync with the source artwork:

- Windows: `powershell -File scripts/generate_icons.ps1`
- Unix/macOS: `./scripts/generate_icons.sh` (requires ImageMagick)

The Windows build script does not regenerate `icon.ico` automatically —
it only embeds it. A stale `icon.ico` will be silently shipped if you
forget to rerun the generator.

## Manual smoke checks for the UI

Some UI behavior cannot be unit-tested directly with iced 0.14, so it is
worth eyeballing after touching `crates/ui`:

- launch the `k580` binary and confirm there is no white flash on
  Windows (cloak/uncloak via DWM, see `docs/ui_app.md`);
- run `cargo build --release -p k580-ui` and double-click
  `target/release/k580.exe`: no console window should pop up;
- in the memory cell editor, confirm `Enter`, `Ctrl+Enter`, `Alt+Enter`,
  and `Tab`/`Shift+Tab` follow the table in `docs/ui_app.md`;
- in the inline memory list, confirm Tab walks down through addresses
  and Shift+Tab walks back up;
- focus the address spinner with the mouse and Tab through the panel:
  hover and focus rings should match the standalone byte-value field.
