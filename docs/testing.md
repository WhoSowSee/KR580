# Testing

Run the same checks from the repository root:

```sh
cargo fmt --all --manifest-path /d/kr-580/Cargo.toml
cargo clippy --workspace --all-targets --manifest-path /d/kr-580/Cargo.toml -- -D warnings
cargo test --workspace --manifest-path /d/kr-580/Cargo.toml
```

On Windows, the installed-driver PrintTicket roundtrip has an explicit ignored
smoke test:

```sh
cargo test -p kr580 --test native_printer_properties -- --ignored --nocapture
```

It loads a real installed printer, parses its capabilities, and reapplies its
current selected option without submitting a physical print job.

## Current coverage

- `k580-core`: opcode classification, documented-opcode smoke execution,
  modular executor families, flags, conditionals, stack, interrupts, I/O
  routing, exact `RunForTStates` accounting, and `tact_execution`
  regressions proving that partial T-state walks do not commit PC,
  memory, or device I/O before the instruction boundary.
- `kr580` internal modules: port routing, invalid-port typed errors,
  monitor framebuffer/attribute state, storage worker queueing, storage
  visible-buffer clearing, storage debug-buffer acceptance without an
  attached file, network no-data handling, Tokio TCP worker roundtrip,
  CP866 decoding and 80-column native printer line wrapping, PrintTicket
  capability parsing, delta generation, feature de-duplication, and property
  localization,
  `.580` roundtrip/determinism/header validation, raw `.krs` behavior,
  settings JSON versioning, `.txt`/`.xlsx` direct exporters/importers,
  command-mediated state mutation, floppy image attachment, printer
  clearing/raw export, and actor publication of completed printer jobs. Native
  printer discovery, capability loading, PrintTicket validation, fallback
  Properties pages, and printing are a Windows smoke-test path because they
  depend on installed OS printers and drivers. The `square_program` integration
  test synthesizes a
  temporary `square.580` snapshot, loads it, runs it to HLT through the
  `Emulator`, and asserts the monitor pixel layer contains exactly
  the 28-pixel outline of an 8×8 square (corners included, interior
  untouched, every pixel at colour `0x7F`) – a smoke check that
  `OUT 00h` round-trips through `IoBus` into `MonitorDevice` using
  the documented 3-byte graphics command (`prompt/03_peripherals.md`).
- `kr580` UI and installer: pure view helpers, printer HEX and CP866 text formatting,
  printer view-mode toggling, printer target/settings updates, memory-cell action and return shortcut
  rebinding, detachable tool-window lifecycle,
  installer layout helpers, install-mode detection, embedded/fallback
  installer payload selection, and launcher-to-app path resolution.

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

The build pipeline embeds `assets/icons/icon-64.png` (runtime window icon) and,
on Windows, one of the checked-in PE resources under `assets/icons/*.ico`.
If you replace `assets/icons/icon.png`, `file-580.png`,
`installer-setup.png`, or `installer-uninstall.png`, run the matching script
before rebuilding so the embedded artefacts stay in sync with the source
artwork:

- Windows: `powershell -File scripts/generate_icons.ps1`
- Unix/macOS: `./scripts/generate_icons.sh` (requires ImageMagick)

The Windows build script does not regenerate `icon.ico` automatically –
it only embeds it. A stale `icon.ico` will be silently shipped if you
forget to rerun the generator.

## Manual smoke checks for the UI

Some UI behavior cannot be unit-tested directly with iced 0.14, so it is
worth eyeballing after touching `crates/ui`:

- launch the `k580` binary and confirm there is no white flash on
  Windows (cloak/uncloak via DWM, see `docs/ui_app.md`);
- run `cargo build --release -p kr580` and double-click
  `target/release/k580.exe`: no console window should pop up;
- run `cargo run -p kr580 --bin kr -- <path/to/file.580>` and confirm
  the GUI loads the snapshot and the terminal prompt returns immediately;
- run `cargo run -p kr580 --bin kr -- --help` and confirm usage prints
  to stdout;
- run `cargo run -p kr580 --bin kr -- --install` and confirm the
  graphical installer opens for developer or already-installed layouts;
- run `cargo run -p kr580 --bin k580-installer` and confirm the native OS
  title bar is gone, the custom black/white title bar says only `KR580 Setup`,
  drags the window, and exposes the same SVG minimize / maximize / close glyphs
  as the emulator; confirm the setup content is one black panel with no left
  information rail, and System / Portable mode selection, folder browsing,
  Browse, Install, and the PATH checkbox render with proportional row heights
  and without drifting or overlapping text; on Windows, confirm the setup window
  has the same rounded native corners as the emulator; confirm Russian system UI
  starts with Russian installer text and English/other system UI starts with
  English installer text; confirm the Ready result card is compact instead of
  reserving installed-state height; click Install once and confirm the result
  panel immediately shows an Installing progress bar, then confirm the
  installed status expands without pushing the bottom action out of its rail and
  the finish screen centers a checked "Open installation folder" action for
  portable installs or "Launch KR580" for system installs between the installed
  status card and a pinned `Done` button; in System mode confirm the
  "Create desktop shortcut" checkbox is
  visible, and in Portable mode confirm it is hidden, the Windows scope selector
  is hidden, and the default folder is `%USERPROFILE%\KR580`; in both modes
  confirm the "Associate .580 files with KR580" checkbox is visible;
- after a System-mode smoke install on Windows, confirm `KR580.lnk` exists in
  the selected Start Menu scope, the optional desktop shortcut follows the
  checkbox, no terminal window flashes while shortcuts are created, the `.580`
  association follows its checkbox, the install root contains `app/k580.exe`,
  `app/uninstaller.exe`, and `bin/kr.exe`, no installed `app/k580-installer.exe`,
  the setup file shows the setup icon, the installed `app/uninstaller.exe`
  shows the uninstall icon, and Apps & Features receives a `KR580` uninstall
  entry whose command points at `uninstaller.exe --uninstall <install root>`;
  run that uninstall entry and confirm it opens the black/white uninstaller GUI,
  immediately shows a progress bar while cleanup runs, switches to a completed
  state with `Close` on English/other systems or `Закрыть` on Russian systems,
  and removes the install folder only after that button is pressed; after a
  portable smoke install, confirm none of those OS entries are created and that
  `.580` is associated only when its checkbox was selected; run the portable
  `app/uninstaller` and confirm it removes the portable `.580` association and
  the `<install root>/bin` PATH entry when those checkboxes were selected;
- run `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build_installer.ps1`
  on Windows or `bash scripts/build_installer.sh` on Unix/macOS and confirm
  a standalone `KR580-Setup-*` artifact appears under `dist/`; for release
  packaging, also smoke-check `--target` builds and `scripts/package_installer_deb.sh` for one Linux target;
- run `cargo run -p kr580 --bin kr -- nonexistent.580` and confirm
  the GUI launches with a localized "Файл не найден" error notice;
- on Linux, run `cargo run -p kr580 --bin kr -- -r`, then confirm
  `~/.local/share/mime/packages/application-x-kr580.xml` and
  `~/.local/share/applications/kr580.desktop` were created and a `.580`
  file opens with `kr` from the file manager;
- on macOS, run `cargo run -p kr580 --bin kr -- -r`, then confirm
  `~/Applications/kr580.app` exists and `lsregister` reports it;
- open each top-menu dropdown and verify Up/Down wraps through enabled rows
  without moving the selected RAM address, paints the current row with the
  pointer-hover fill and no blue border; verify Left/Right cyclically opens the
  previous/next dropdown category, skips Settings, and does not draw a blue
  category border or move RAM;
  use Tab/Shift+Tab to walk the blue-outlined category and rows through
  File → MP-System → View → Settings → Help in both directions, confirm
  Settings receives the category outline without opening a dropdown, category
  outlines include comfortable padding around their labels, disabled Clear
  Halt and separators are skipped, and Enter activates the outlined row or
  opens Settings from its category stop;
- open the in-app Settings dialog (`,`), confirm logical focus starts on the
  language control without a white outline, and use Tab/Shift+Tab to visit both
  On and Off segments for Follow PC and memory-operand highlighting; verify the
  `.580` association plus Reset, Cancel, and Save show a white border without a
  fill change, and that Enter or a mouse click clears the border before
  activation; open the language dropdown and confirm its anchor gains the same
  active fill as an opened printer selector; open the Reset confirmation and
  confirm Cancel starts filled without a white border, then Tab/Shift+Tab removes
  the focus fill and draws only the white border; in the Sidebar, verify
  Tab/Shift+Tab moves the category cursor
  without changing the page until Enter; finally confirm the `.580 file
  association` row shows `Add` when the association is missing and `Remove`
  when it is present, then click it and verify the button label flips and the
  OS association is created/removed;
- make the current file dirty and invoke Open, New, Import, Close, and HDD
  deletion confirmations; in each shared confirmation overlay verify Cancel is
  initially filled without a white border, the first Tab/Shift+Tab changes the
  indication to a white border without focus fill, and Enter or pointer input
  hides the border before activating the chosen button;
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
- in the inline memory list, confirm the scrollbar thumb is compact, does not
  jump when grabbed off-centre, appears when hovering either the thumb or an empty
  part of its rail, moves by only a few addresses for a minimal drag, catches the
  pointer within 12 px, stays under it for a fast drag, reaches both track ends
  without stutter, and leaves wheel/touchpad sensitivity unchanged;
- in the opcode picker, type part of an opcode or mnemonic, confirm
  ArrowDown/Tab and ArrowUp/Shift+Tab move the highlighted filtered row
  with wrapping, and Enter writes the highlighted opcode to the selected
  memory cell;
- switch to the Russian layout and confirm the same physical shortcuts
  still resolve: `У` opens the opcode picker, `Ctrl+Ы` saves, `Ctrl+У`
  exports, `Ctrl+Ь` opens the monitor, and `Ctrl+А` opens the floppy
  buffer;
- open Settings → Shortcuts, confirm `Memory cell action` / `Действие с ячейкой ОЗУ` shows `Alt+Enter` and `Return to memory operand cell` / `Вернуться к ячейке операнда ОЗУ` shows `Shift+Alt+Enter`, click the current shortcut for Monitor,
  press `Ctrl+Shift+Alt+M`, save, and confirm that chord opens the monitor
  while the Quick Access tooltip and View menu row show `Ctrl+Shift+Alt+M`;
- reopen Settings → Shortcuts, press `Reset shortcuts`, save, and confirm the
  Monitor shortcut returns to `Ctrl+M`, the memory cell action returns to
  `Alt+Enter`, and the memory return action returns to `Shift+Alt+Enter`;
- hover the execution buttons and Quick Access chips and confirm
  shortcuts render as muted same-line tooltip text (`Ctrl+R`, `Ctrl+T`,
  `Ctrl+Y`, `Ctrl+M`, `Ctrl+F`) where the action actually has one, and
  tooltips near window edges keep visible breathing room instead of
  snapping flush to the border without moving farther away from the
  hovered button;
- hover the address buffer, instruction register, decoder, multiplexer rows,
  cycle rows, control-signal lamps, and status register; confirm their tooltip
  body text uses the same readable size as button tooltip labels while shortcut
  suffixes remain smaller;
- on the schematic, enter inline editing for «Буферный регистр 1» and
  «Буферный регистр 2» and confirm the hex value stays vertically stable
  instead of jumping upward; double-click must clear the editor while
  retaining the current value as its placeholder; while replacement is
  active, Left/Right must carry the empty editor across `A/B/C`, and all
  four arrows must carry it through the multiplexer grid; with either a selected
  register or its inline editor active, Tab/Shift+Tab must walk one wrapping ring
  through `A/B/C` and multiplexer `B/C/D/E/H/L`; Up/Down on `A/B/C` must do
  nothing, while Left/Right remains confined to that trio; selected `A/B/C`
  blocks must use the standard selection-blue token without an alpha override,
  matching RAM and mux cells; Up/Down in the
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
- on Windows, open Settings → External Devices, choose a printer with the
  custom Printer row setup modal, confirm its status/driver/port details,
  paper sizes, paper sources, and orientation are populated; confirm the modal
  appears at its final size before the asynchronous printer details arrive,
  the Name and Comment text have balanced outer margins, the compact dialog
  does not clip long printer, paper, or source values, the orientation content
  has balanced top and bottom spacing, and the close
  glyph uses the standard framed `34x34` modal button, section labels interrupt
  the top-left border, no
  header/footer separators are drawn, and the paper preview rotates when
  landscape is selected; open Properties,
  visit Favorites, General, Paper, Graphics, and Advanced, and confirm feature,
  option, and parameter labels follow the selected app language without exposing
  raw QName prefixes or `PageDevmodeSnapshot`; change a driver option, close it,
  and confirm the emulator remains responsive and refreshes the top-level
  controls; confirm dropdown panels keep a gap below their anchors, retain
  the bottom border under the final option in both setup windows, and close
  after clicking elsewhere inside the same modal; confirm an opened property
  selector overlays the following rows instead of moving them;
  use `Tab` and `Shift+Tab` to traverse the enabled top-level controls and the
  complete Properties ring in both directions, including tabs, active feature
  controls, parameter fields, profiles, and footer actions; confirm the blue
  outline appears only after keyboard traversal and disappears on Enter or a
  mouse click while the activated selector/tab/radio keeps only its normal
  active fill, bottom indicator, or selected dot; then use
  `ArrowUp`/`ArrowDown` in each kind of open selector and
  confirm the highlight moves without committing until `Enter`; confirm `Esc`
  closes the selector before the modal; confirm Properties opens on Favorites
  without a focus outline, a mouse-selected tab shows only its bottom indicator,
  and keyboard traversal then enables the control focus outline; confirm the property
  lists remain scrollable without a visible scrollbar, the compact paper preview
  fits without a side-panel scroll, and the top-level Paper and Orientation
  groups have equal height; save and reload a named profile,
  restart the emulator, and confirm
  the printer footer uses that global target and configuration for every file;
  switch the Printer setup window row to System, reopen setup, confirm the OS
  dialog appears, then switch back to the emulator window; with a long printer
  name, confirm the clear icon stays fixed at the right edge of the row; clear
  the row and confirm it returns to the OS default target;
- send bytes to port `04h`, open the Принтер quick-access chip, and confirm
  the buffer renders as uppercase HEX with four-digit offsets and 16 bytes
  per line; toggle the `type` button and confirm CP866 text appears without
  changing the byte count, then toggle back to HEX; click the settings gear,
  select a different printer and paper/orientation, and confirm the footer shows
  its name without changing `settings.json`; confirm the header contains one
  Print action and no separate PDF action; print and
  verify the UI returns from `Busy` to `Ready` and the selected printer
  receives the CP866-decoded text; cancel a native printer or output-file prompt
  and confirm the UI also returns to `Ready`, shows no raw Win32 error, and keeps
  all three footer fields within the window; clear the buffer and confirm the active
  printer target remains unchanged; detach, pin, attach, and close the window.
