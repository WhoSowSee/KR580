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
  framebuffer/attribute state, storage worker queueing, storage visible
  buffer clearing, storage debug-buffer acceptance without an attached
  file, network no-data handling, Tokio TCP worker roundtrip, CP866
  decoding, and asynchronous printer PDF export with spool preservation.
  The PDF regression test parses the generated file, verifies that the
  embedded font resource exists, and extracts the expected CP866-decoded
  Cyrillic text.
- `k580-persistence`: `.580` roundtrip/determinism/header validation,
  raw `.krs` behavior, settings JSON versioning, `.txt`/`.xlsx`
  direct exporters, and `.txt`/`.xlsx` importers (round-trip an
  `ExportModel` back into a `Cpu8080State`).
- `k580-app`: command-mediated state mutation, including floppy image
  attachment, floppy debug-buffer mode, floppy-buffer clearing, printer
  clearing/export, and actor publication of completed printer jobs. The
  `square_program` integration test generates a
  temporary `square.580` snapshot, loads it, runs it to HLT through the
  `Emulator`, and asserts the monitor pixel layer contains exactly
  the 28-pixel outline of an 8×8 square (corners included, interior
  untouched, every pixel at colour `0x7F`) – a smoke check that
  `OUT 00h` round-trips through `IoBus` into `MonitorDevice` using
  the documented 3-byte graphics command (`prompt/03_peripherals.md`).
- `k580-ui`: pure view helpers, printer HEX and CP866 text formatting,
  printer view-mode toggling, PDF path normalization, and detachable
  tool-window lifecycle.

External Intel 8080 binary suites are not included in this workspace.
When available, add them as an additional compatibility gate instead of
replacing the local semantic tests.

## Sample programs

- `counter_loop.580` – pre-existing demo snapshot.
- `test_program.580` – pre-existing demo snapshot.
- `square_program` synthesizes its `.580` fixture during the test. The
  encoded program walks the four edges of an 8×8 square at the origin
  of the graphics layer, emitting one 3-byte graphics command per
  pixel. Command form is `[FF][X][Y]` (`FF` = bit7=1 for graphics + max
  colour `0x7F`).
- `printer_demo_program_writes_test_line_to_port_four` loads a compact
  null-terminated 8080 loop at `0000h`, writes `TEST PRINTER\r\n` through
  `OUT 04h`, and verifies the CPU reaches `HLT` with the expected spool.

## Asset prerequisites

The build pipeline embeds `assets/icons/icon-64.png` (runtime window
icon) and, on Windows, `assets/icons/icon.ico` (PE resource). Both files
are checked in. If you replace `assets/icons/icon.png` (the master), run
the matching script before rebuilding so the embedded artefacts stay in
sync with the source artwork:

- Windows: `powershell -File scripts/generate_icons.ps1`
- Unix/macOS: `./scripts/generate_icons.sh` (requires ImageMagick)

The Windows build script does not regenerate `icon.ico` automatically –
it only embeds it. A stale `icon.ico` will be silently shipped if you
forget to rerun the generator.

Printer PDF tests also require the checked-in `assets/fonts/RobotoMono.ttf` because `k580-devices` embeds it at compile time. The font has no generated derivative that needs regeneration.

## Manual smoke checks for the UI

Some UI behavior cannot be unit-tested directly with iced 0.14, so it is
worth eyeballing after touching `crates/ui`:

- launch the `k580` binary and confirm there is no white flash on
  Windows (cloak/uncloak via DWM, see `docs/ui_app.md`);
- run `cargo build --release -p k580-ui` and double-click
  `target/release/k580.exe`: no console window should pop up;
- run `cargo run -p k580-ui --bin kr -- <path/to/file.580>` and confirm
  the GUI loads the snapshot and the terminal prompt returns immediately;
- run `cargo run -p k580-ui --bin kr -- --help` and confirm usage prints
  to stdout;
- run `cargo run -p k580-ui --bin kr -- nonexistent.580` and confirm
  the GUI launches with a localized "Файл не найден" error notice;
- on Linux, run `cargo run -p k580-ui --bin kr -- -r`, then confirm
  `~/.local/share/mime/packages/application-x-kr580.xml` and
  `~/.local/share/applications/kr580.desktop` were created and a `.580`
  file opens with `kr` from the file manager;
- on macOS, run `cargo run -p k580-ui --bin kr -- -r`, then confirm
  `~/Applications/kr580.app` exists and `lsregister` reports it;
- open the in-app Settings dialog (`,`), go to General, and confirm the
  `.580 file association` row shows `Add` when the association is
  missing and `Remove` when it is present; click it and verify the
  button label flips and the OS association is created/removed;
- in the memory cell editor, confirm `Enter`, `Ctrl+Enter`, `Alt+Enter`,
  and `Tab`/`Shift+Tab` follow the table in `docs/ui_app.md`;
- paste `3E 41 D3 03 76` into a memory value field and confirm the five
  consecutive cells update immediately without first deleting the
  existing two-digit value; malformed or overflowing input must not
  write a partial sequence and must show a short localized status
  without repeating the pasted text;
- in the inline memory list, confirm Tab walks down through addresses
  and Shift+Tab walks back up, with each destination empty and its stored
  byte shown as the placeholder;
- in the opcode picker, type part of an opcode or mnemonic, confirm
  ArrowDown/Tab and ArrowUp/Shift+Tab move the highlighted filtered row
  with wrapping, and Enter writes the highlighted opcode to the selected
  memory cell;
- switch to the Russian layout and confirm the same physical shortcuts
  still resolve: `У` opens the opcode picker, `Ctrl+Ы` saves, `Ctrl+У`
  exports, `Ctrl+Ь` opens the monitor, and `Ctrl+А` opens the floppy
  buffer;
- hover the execution buttons and Quick Access chips and confirm
  shortcuts render as muted same-line tooltip text (`Ctrl+R`, `Ctrl+T`,
  `Ctrl+Y`, `Ctrl+M`, `Ctrl+F`) where the action actually has one, and
  tooltips near window edges keep visible breathing room instead of
  snapping flush to the border without moving farther away from the
  hovered button;
- on the schematic, enter inline editing for «Буферный регистр 1» and
  «Буферный регистр 2» and confirm the hex value stays vertically stable
  instead of jumping upward; double-click must clear the editor while
  retaining the current value as its placeholder; while replacement is
  active, Left/Right must carry the empty editor across `A/B/C`, and all
  four arrows must carry it through the multiplexer grid; Up/Down in the
  inline RAM editor must do the same for adjacent memory cells; entering
  replacement again on an already empty field must keep its visible
  `00`, `0000`, or `A` placeholder without materializing it after Esc or
  repeated Tab/Shift+Tab focus cycles;
- click the status-strip `HLT` indicator on and off and confirm the
  selected RAM row does not move; then execute a `76` byte and confirm
  the highlight stays on that HLT row without briefly flashing the next
  address; after manually clearing HLT, reset registers and confirm the
  selected RAM row still returns to PC `0000`;
- focus the address spinner with the mouse and Tab through the panel:
  hover and focus rings should match the standalone byte-value field.
- clear the address or register-name field and type a valid value in its
  paired value field; the empty field must become `0000` or `A`
  respectively, while invalid value input must leave it empty;
- click the Дисковод quick-access chip, confirm the buffer modal opens,
  Esc and backdrop-click close it, the open-image button attaches an
  existing `.kpd`/`.img`/`.bin` file, the save button writes the visible
  buffer to `.kpd`/`.img`/`.bin` through three separate export filters
  with `.kpd` selected first, the detach-image button clears the file
  path while leaving the visible buffer text intact, the binary button
  switches the body to the image file contents, the debug button toggles
  between `bug-off` and active blue `bug`, the empty buffer state has no
  cursor glyph, and the clear button empties the visible buffer without
  changing the device footer state.
- send bytes to port `04h`, open the Принтер quick-access chip, and confirm
  the buffer renders as uppercase HEX with four-digit offsets and 16 bytes
  per line; toggle the `type` button and confirm CP866 text appears without
  changing the byte count, then toggle back to HEX; print to PDF, verify the
  UI returns from `Busy` to `Ready` and the CP866 text is readable; clear the
  buffer and confirm the last PDF path remains in the footer; detach, pin,
  attach, and close the window.
