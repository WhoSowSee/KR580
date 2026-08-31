# Testing

Run the same checks from the repository root:

```sh
cargo fmt --all --manifest-path /d/kr-580/Cargo.toml
cargo clippy --workspace --all-targets --manifest-path /d/kr-580/Cargo.toml -- -D warnings
cargo test --workspace --manifest-path /d/kr-580/Cargo.toml
```

The workspace MSRV is Rust 1.88.0. Verify it against the locked dependency set:

```sh
cargo +1.88.0 check --workspace --all-targets --locked
```

The root `CHANGELOG.md` and `CHANGELOG-EN.md` files feed release automation.
Their package-local `crates/ui/CHANGELOG.md` and
`crates/ui/CHANGELOG-EN.md` copies are embedded by the application. The
Russian pair and English pair must remain byte-for-byte identical. Whenever
one changelog changes, update all four, keep their release version/date
indexes aligned, and verify the publishable workspace archive:

```sh
cargo package --workspace --locked
```

For a release, every non-release commit since the previous tag must be
represented by a bullet; a mixed commit may use separate bullets for an
independent breaking change and user-facing feature. The `chore(release)`
version-bump commit is excluded.

Dependency audits use `cargo machete --with-metadata --skip-target-dir .`.
The Windows-only `winresource` build dependency is explicitly ignored by that
scanner because `crates/ui/build.rs` consumes it behind a target `cfg`.
Feature audits inspect the effective all-target graph and invert any dependency
whose defaults are expected to stay off:

```sh
cargo tree -p kr580 --target all -f "{p} features=[{f}]"
cargo tree -p kr580 --target all -e features -i roxmltree@0.21.1
cargo tree -p kr580 --target all -e features -i windows@0.62.2
```

The direct dependency declarations disable broad defaults where the app uses a
narrower codec, executor, or build-time feature set. Cargo feature
unification may still re-enable a feature when another dependency requests it;
the resolved tree, not the direct declaration alone, is the verification source.

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
  rebinding, main-window file drag/drop hover routing, supported-extension
  validation, dropped-path dirty confirmation, detachable tool-window lifecycle,
  native-dialog parent selection, installer layout helpers, install-mode
  detection, embedded/fallback installer payload selection, and launcher-to-app
  path resolution.

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

The build pipeline embeds `crates/ui/assets/icons/icon-64.png` (runtime window
icon) and, on Windows, one of the checked-in PE resources under
`crates/ui/assets/icons/*.ico`. If you replace
`crates/ui/assets/icons/icon.png`, `file-580.png`,
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

- at the default `1180×720` window size, confirm the left CPU stack and the
  mux/status column retain their original wide separation; maximize the window
  and confirm the two columns remain centred as a compact group with a 72 px gap;

- launch the `k580` binary and confirm there is no white flash on
  Windows (cloak/uncloak via DWM, see `docs/ui_app.md`);
- run `cargo build --release -p kr580` and double-click
  `target/release/k580.exe`: no console window should pop up;
- run `cargo run -p kr580 --bin kr -- <path/to/file.580>` and confirm
  the GUI loads the snapshot and the terminal prompt returns immediately;
- drag a `.580` file over the main emulator and confirm the surface darkens
  slightly, the localized `Open in emulator` label follows the cursor without
  clipping at any window edge, the surface returns to normal when the drag
  leaves, and the file opens on drop; repeat with `.krs` and confirm the
  RAM-range dialog appears;
- drag an unsupported file and confirm the emulator state remains unchanged
  while a localized pink-border format error appears; with unsaved changes,
  drop a supported file and confirm the modal names the dropped-file action,
  Cancel preserves the current state, and Open uses the same dropped path;
- open a `.krs` file from File → Open, enter a start address, and confirm its
  bytes appear at that RAM address; use Save as with `.krs` to verify the
  selected inclusive RAM range is written without a header;
- open native dialogs from the main window, detached Monitor/Floppy/HDD,
  Settings, Import/Export, and the installer; confirm each dialog belongs to its
  owning window and cancellation leaves state unchanged;
- export the full memory range to XLSX and confirm Field/Value/Address/Command
  columns open at readable widths and long values do not stretch the worksheet;
- run `cargo run -p kr580 --bin kr -- --help` and confirm usage prints
  to stdout;
- run `cargo run -p kr580 --bin kr -- --install` and confirm the
  graphical installer opens for developer or already-installed layouts;
- run `cargo run -p kr580 --bin k580-installer` and confirm the shared custom
  title bar drags and exposes working caption actions; the rail keeps balanced
  spacing, `Windows · x64` metadata, and a two-column `ADDR` / `DATA` / `CTRL` /
  `INT` table; joined mode, scope, and path/Browse controls retain one outer frame,
  square seams, rounded selected rails, blue path text, and non-cyan hover states;
  all checkboxes use the same compact size and unchecked controls have no fill;
  Russian copy reads `Системный`, `Портативный`, and `Путь`; System mode orders
  PATH, file associations, then desktop shortcut, while Portable hides scope and
  desktop shortcut and defaults to `%USERPROFILE%\KR580`; verify Installing,
  success, and failure states without duplicate status blocks, and confirm the
  finish action follows the report above the pinned `Done` button;
- after a System-mode smoke install on Windows, confirm `KR580.lnk` exists in
  the selected Start Menu scope, the optional desktop shortcut follows the
  checkbox, no terminal window flashes while shortcuts are created, the `.580`
  and `.krs` associations follow their checkbox, the install root contains `app/k580.exe`,
  `app/uninstaller.exe`, and `bin/kr.exe`, no installed `app/k580-installer.exe`,
  the setup file shows the setup icon, the installed `app/uninstaller.exe`
  shows the uninstall icon, and Apps & Features receives a `KR580` uninstall
  entry whose command points at `uninstaller.exe --uninstall <install root>`;
  run that uninstall entry and confirm the shared title bar, centered product
  header, border-only path, and joined `СИСТЕМА → СВЯЗИ → ФАЙЛЫ` block render
  correctly; verify real stage ordering, disabled action during cleanup, automatic
  post-exit removal scheduling, and percentage/progress movement in small
  monotonic steps rather than a direct 40% → 100% jump or a pause at 40% while
  Windows broadcasts environment changes; confirm the displayed value remains
  below the next unconfirmed milestone, the Files stage stays active, and Close
  stays disabled until the animation reaches exactly 100%, while a failure stops
  the animation and leaves the failed stage red; after a
  portable smoke install, confirm none of those OS entries are created and that
  `.580` and `.krs` are associated only when their checkbox was selected; run the portable
  `app/uninstaller` and confirm it removes the portable file associations and
  the `<install root>/bin` PATH entry when those checkboxes were selected;
- run `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build_installer.ps1`
  on Windows or `bash scripts/build_installer.sh` on Unix/macOS and confirm
  a standalone `KR580-Setup-*` artifact appears under `dist/`; for release
  packaging, also smoke-check `--target` builds and `scripts/package_installer_deb.sh` for one Linux target, confirm the
  Debian control metadata contains `libdbus-1-3` and `zenity`, and open a file
  dialog in the Debian, Snap, and Nix artifacts;
- run `cargo run -p kr580 --bin kr -- nonexistent.580` and confirm
  the GUI launches with a localized "Файл не найден" error notice;
- on Linux, run `cargo run -p kr580 --bin kr -- -r`, then confirm
  `~/.local/share/mime/packages/application-x-kr580.xml` and
  `~/.local/share/applications/kr580.desktop` were created and `.580` and `.krs`
  files open with `kr` from the file manager;
- on macOS, run `cargo run -p kr580 --bin kr -- -r`, then confirm
  `~/Applications/kr580.app` exists and `lsregister` reports it;
- open each top-menu dropdown and verify Up/Down wraps through enabled rows
  without moving the selected RAM address, paints the current row with the
  pointer-hover fill and no blue border; verify Left/Right cyclically opens the
  previous/next dropdown category, skips Settings, and does not draw a blue
  category border or move RAM;
- with one top-menu dropdown open, hover File, MP-System, View, and Help and
  confirm the dropdown switches without another click; close it and confirm
  hovering those categories alone does not open anything;
  use Tab/Shift+Tab to walk the blue-outlined category and rows through
  File → MP-System → View → Settings → Help in both directions, confirm
  Settings receives the category outline without opening a dropdown, category
  outlines include comfortable padding around their labels, disabled Clear
  Halt and separators are skipped, and Enter activates the outlined row or
  opens Settings from its category stop;
- open the in-app Settings dialog (`,`), confirm logical focus starts on the
  language control without a white outline, and use Tab/Shift+Tab to visit both
  On and Off segments for Follow PC and memory-operand highlighting; verify the
  `.580` / `.krs` associations plus Reset, Cancel, and Save show a white border without a
  fill change, and that Enter or a mouse click clears the border before
  activation; open the language dropdown and confirm its anchor gains the same
  active fill as an opened printer selector; open the Reset confirmation and
  confirm Cancel starts filled without a white border, then Tab/Shift+Tab removes
  the focus fill and draws only the white border; in the Sidebar, verify
  Tab/Shift+Tab moves the category cursor
  without changing the page until Enter; finally confirm the `.580 and .krs file
  associations` row shows `Add` when either association is missing and `Remove`
  when it is present, then click it and verify the button label flips and the
  OS association is created/removed;
- make the current file dirty and invoke Open, New, Import, Close, and HDD
  deletion confirmations; in each shared confirmation overlay verify Cancel is
  initially filled without a white border, the first Tab/Shift+Tab changes the
  indication to a white border without focus fill, and Enter or pointer input
  hides the border before activating the chosen button;
- open Export and Import and use Tab/Shift+Tab from their native text inputs and
  buttons; confirm captured keyboard events still traverse each custom wrapping
  focus ring in both directions, draw a white outline on the active tab,
  target add/delete button, field, checkbox, or footer control without replacing
  its selected fill/check,
  skip the unavailable Import target selector and disabled Import action, move
  directly from Export's Text file tab to its target selector without an
  invisible intermediate stop, and let Enter or pointer input clear the
  keyboard-only outline before activating the current control; confirm Import
  opens at a fixed size without a title bar or close action, with a large source
  drop zone, `.txt`/`.xlsx` format hint inside the zone below Choose file, and
  adjacent neutral Cancel/Import actions on the right; verify the empty drop zone
  is taller, selecting a file contracts it without changing the dialog height,
  removes the format badge and repeated drop instruction, and leaves the icon,
  shortened path, Choose file action, and format hint with a slightly larger gap
  between the path and action; verify the target label sits close to its field
  while the source-to-target gap is visibly larger, the taller empty source zone
  leaves less space above the footer, and neither state has an oversized footer
  spacer; confirm the gap between Choose file and the format hint is identical
  before and after selecting a file; drag a file over the modal
  and verify only the source zone gains the accent border, then verify leaving
  clears it and dropping a supported `.txt` or `.xlsx` file selects it and
  enables Import without changing its neutral color; drop an unsupported
  extension and confirm the modal remains open with a localized inline error;
  load an import file with enough long target names to overflow the two-row
  dropdown, confirm the dialog height stays fixed, the wheel/touchpad still
  scrolls the list without painting a scrollbar, the options keep the same
  four-pixel inner panel spacing as the language dropdown, the popup keeps a
  separate eight-pixel gap below its closed selector, both visible options are
  fully rendered, and every target stays on one clipped, middle-shortened row;
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
- while file-content mode is active, modify or atomically replace the
  attached floppy image and `hdd.kpd` from another process; both open
  windows must refresh without toggling file-content mode, while unchanged
  files must not be read again on every UI tick;
- switch between Russian and English and inspect the Floppy, HDD, Network, and
  Printer footers; every localized status or mode value after a colon must begin
  with a lowercase letter (`Статус: готов`, `Status: refused`, `Mode: client`),
  while paths, endpoints, and printer names must preserve their original case;
- on Windows, open Settings → External Devices, choose a printer with the
  custom Printer row setup modal, confirm its status/driver/port details,
  paper sizes, paper sources, and orientation are populated; with the
  application in Russian confirm the Status row is Russian for whatever the
  spooler reports, including states past the common ones - pause the printer,
  open its cover, or unload paper to see `Приостановлен`, `Открыта крышка`,
  `Нет бумаги` rather than English; confirm the modal
  appears at its final size before the asynchronous printer details arrive,
  the Name and Comment text have balanced outer margins, the compact dialog
  does not clip long printer, paper, or source values, the orientation content
  has balanced top and bottom spacing, and the close
  glyph uses the standard framed `34x34` modal button, section labels interrupt
  the top-left border, no
  header/footer separators are drawn, and the paper preview rotates when
  landscape is selected; open Properties,
  check that the Paper tab's Size, Source, and Orientation rows have no shared
  frame, and their selectors and first radio align with the driver fields below;
  check this for multiple printers and both app languages, including a driver
  with no additional Paper features; keep the Profiles and Preview frames intact;
  visit Favorites, General, Paper, Graphics, and Advanced, and confirm feature,
  option, and parameter labels follow the selected app language without exposing
  raw QName prefixes or `PageDevmodeSnapshot`; with Windows and the printer
  driver using Russian, switch the application to English and confirm those
  rows contain no Cyrillic, including altitude correction, print quality,
  duplex mode, and automatic paper-source selection; also inspect the paper
  and source selectors in both Setup and the Properties Paper tab and confirm
  standard bins such as `Автовыбор` / `Лоток 1` render as `Auto select` /
  `Tray 1` identically in both dialogs; with an English driver and the
  application in Russian, confirm `Automatically Select`, `Tray 1`, and
  `DL envelope` render as `Автовыбор`, `Лоток 1`, and `Конверт DL`, while
  unknown foreign names use `Подача <id>` or `Бумага <id>`; both directions
  share one table in `view/printer_setup/driver_locale.rs`; focused direction
  and fallback regressions live in `view/printer_setup/labels/tests.rs`;
  change a driver option, close it,
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
  groups have equal height; on Advanced, confirm every parameter input starts on
  the same left edge as the selectors while its Apply button only reduces the
  input width; save and reload a named profile,
  restart the emulator, and confirm
  the printer footer uses that global target and configuration for every file;
  switch the Printer setup window row to System, reopen setup, confirm the OS
  dialog appears, then switch back to the emulator window; with a long printer
  name, confirm the clear icon stays fixed at the right edge of the row; clear
  the row and confirm it returns to the OS default target;
- detach the Printer device window, open Print Setup from its header, and confirm
  setup plus nested Properties render over the detached printer instead of the
  main emulator; verify each native dialog hugs its panel with no empty backdrop,
  stays centred over the Printer, and uses a separate `1040×680` Properties
  window above the unchanged `720×500` Setup window; neither dialog may resize
  the Printer from its original `760×340` bounds, and Properties must appear at
  its final text scale immediately without stretching the Setup window; while
  Properties is open, confirm Setup's close glyph, Cancel, and OK buttons stay
  muted with unchanged borders; try each control and a native Setup close request,
  confirm Properties receives focus plus a clearly visible but restrained 520 ms
  surface-and-border pulse that rises and fades once, then close Properties and
  confirm the parent controls become active;
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
