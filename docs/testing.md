# Testing

Run the same checks from the repository root:

The `loaded_range_and_next_save_follow_the_bytes_read` UI regression test covers
successive loads with different lengths and replaces the file before delivering
the real backend completion events; ordinary Save must retain exactly the loaded
range. Event delivery is explicit, without sleeps or timing-dependent file changes.

```sh
cargo fmt --all --manifest-path /d/kr-580/Cargo.toml
cargo clippy --workspace --all-targets --manifest-path /d/kr-580/Cargo.toml -- -D warnings
cargo test --workspace --manifest-path /d/kr-580/Cargo.toml
```

`cargo test -p kr580 --test persistence_formats` checks desktop `.580`
save/load byte layout, independent PC/SP fixtures, and invalid file rejection.
`cargo test -p kr580 --test program_registers` checks all nine register slots
against an independent byte fixture and byte-for-byte re-saving.

`cargo test -p kr580 --lib persistence::subprogram` verifies raw-range saves.
`cargo test -p kr580 --lib persistence::atomic_save` checks the shared `.580`,
`.krs` and settings replacement mechanism: preservation of the previous file after an injected partial write failure,
temporary-file cleanup, replacement, Windows destination-lock failures, and
dangling-symbolic-link preservation on Unix.

The workspace MSRV is Rust 1.88.0. Verify it against the locked dependency set:

```sh
cargo +1.88.0 check --workspace --all-targets --locked
cargo +1.88.0 clippy --workspace --all-targets --locked -- -D warnings
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

For a release, every non-release commit since the previous tag must have
exactly one bullet in each language. Keep all changes from a mixed commit
in that single bullet. The `chore(release)` version-bump commit is excluded.
Avoid dash punctuation in the bullet text; retain the Markdown list markers
and the version/date heading format required by the embedded reader.

Dependency audits use `cargo machete --with-metadata --skip-target-dir .`.
The Windows-only `winresource` and `embed-resource` build dependencies are
explicitly ignored by that scanner because `crates/ui/build.rs` consumes them
behind a target `cfg`.
Linux metadata changes must validate the canonical files under
`crates/ui/assets/linux`; runtime and package outputs are rendered from those
same inputs and must not add independent copies.

On Windows, build `cargo build -p kr580 --bin kr --bin kr580` and inspect
`(Get-Item target/debug/kr.exe).VersionInfo` and the corresponding `kr580.exe`
property. Both `FileDescription` and `ProductName` must be `KR` for the launcher
and `KR580` for the GUI. Repeat with `--release` and `target/release` when
checking release artifacts. The build-script output must contain one
`cargo:rustc-link-arg-bin` resource per binary, without a shared
`cargo:rustc-link-arg` resource. The setup/uninstaller stages must retain their
`KR580 Setup` / `KR580 Uninstaller` descriptions and role-specific icons.

Native metadata smoke checks use disposable roots and never change the current
user's desktop or association databases:

```sh
bash scripts/verify_linux_metadata.sh
bash scripts/verify_release_artifact_names.sh
bash scripts/verify_macos_dmg.sh --dmg <image> --architecture <arm64|x86_64> --version <version>
```

The Linux script requires `desktop-file-utils` and `shared-mime-info`. The
macOS script requires the built-in `hdiutil`, `plutil`, `iconutil`, and `lipo`; it attaches
the image read-only and always detaches it. Release CI runs the Linux ownership
tests and metadata script natively, while each macOS image build invokes the
DMG verifier before upload.
Both DMG staging and mounted-image checks put the binary before `-verify_arch`:
`lipo "$binary" -verify_arch "$architecture"`. All arguments after the flag
are architecture names. Validate both `arm64` and `x86_64` on macOS.
The DMG check expands both ICNS files with `iconutil`. The release-name test
covers a mixed ZIP/DEB/DMG/Snap set, repeated execution, and mismatched versions.
Linux association lifecycle tests launch the real CLI in isolated child
environments with deterministic desktop-tool fixtures. They cover foreign
ownership, changed external defaults, repeated registration, errors before
and after default changes, and rollback of new/existing metadata:

```sh
cargo test --locked -p kr580 --test linux_associations
cargo test --locked -p kr580 --lib file_assoc::linux::tests::desktop_entry_consumer_preserves_executable_and_file_arguments -- --ignored --exact
```

The second command requires `gio` (`libglib2.0-bin` on Ubuntu) and launches a
temporary script through the rendered Desktop Entry to check paths containing
spaces, dollars, backticks and percent signs. Windows registry regression tests
use a private temporary HKCU subtree as their root and never modify the real
`Software/Classes` association tree. macOS bundle tests verify foreign-bundle
preservation, legacy launcher upgrades, and owned portable bundle creation
using temporary directories.
The association button test sends mouse events through the same iced widget
tree across registered/pending changes. CLI lifecycle tests cover Linux file
creation, ownership and rollback; duplicate unit tests of those operations are
not maintained separately. Unix registration-status queries were removed with
their unused tests. The Windows open-command assertion runs inside the actual
isolated registry roundtrip.
Run Clippy natively on Linux/macOS as well as Windows; `--all-targets` alone
does not enable another operating system's `cfg` branches.
`Workspace quality` runs on branch pushes, pull requests and manual dispatch.
Its native Windows, Linux and macOS rows check formatting, strict all-target
workspace Clippy and the complete workspace suite with the checked-in lockfile.
The toolchain action records the resolved compiler version. Release packaging
keeps its separate tag/manual workflow and its existing target matrix.
CPU conditional return/call regressions build short programs in memory from
`prompt/`; they need no external fixture directory. Export/import UI tests set
their language explicitly. Attach tests expect Windows to retain a hidden
window ID and Unix to close it. Parallel Windows association tests reserve
different private registry roots with a process-local counter.
Hosted macOS and release packaging remain native CI checks; a Windows or WSL
run alone does not verify them.
Installer script changes must capture all three effective commands for debug
and release, with and without a target, including `KR580_CARGO=cross`.
Each build contains exactly one `--locked`; verify the icon role and embedded
payload directory as well as the final artifact name. A deliberately stale
temporary lockfile must fail locked metadata validation. Command capture does
not replace hosted cross/container, Snap or native macOS packaging checks.
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

`k580-core`'s `metadata_execution` regression counts allocations on the executing
test thread and verifies a 20,000-instruction INR/JMP loop has zero allocations,
the expected register result and exact T-state total.

Backend persistence regressions check ordered writes through one FIFO worker;
completion reconciliation tests inject results directly to keep import/document
generation races deterministic without oversized files or sleeps.

- `k580-core`: opcode classification, documented-opcode smoke execution,
  modular executor families, flags, conditionals, stack, interrupts, I/O
  routing, exact `RunForTStates` accounting, and `tact_execution`
  regressions proving that partial T-state walks do not commit PC,
  memory, or device I/O before the instruction boundary.
- `kr580` internal modules: port routing, invalid-port typed errors,
  monitor framebuffer/attribute state, complete CP866 byte-to-glyph coverage,
  split-view text sequencing, storage worker queueing, storage
  visible-buffer clearing, storage debug-buffer acceptance without an
  attached file, network no-data handling, Tokio TCP worker roundtrip,
  CP866 decoding and 80-column native printer line wrapping, PrintTicket
  capability parsing, delta generation, feature de-duplication, and property
  localization,
  `.580` roundtrip/determinism/fixed-layout validation, raw `.krs` behavior,
  settings JSON versioning, `.txt`/`.xlsx` direct exporters/importers,
  request-correlated command completion, coalesced actor snapshots,
  command-mediated state mutation, floppy image attachment and worker error
  propagation, non-blocking external image refresh, printer
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
  validation, dropped-path dirty confirmation, full-width detached-window
  drag routing, fixed user-resize settings, detachable tool-window lifecycle,
  native-dialog parent selection, installer layout helpers, install-mode
  detection, embedded/fallback installer payload selection, and launcher-to-app
  path resolution.

`view/widgets/compact_scrollbar/tests.rs` uses headless `tiny-skia` with real iced
`Stack`, button and `Scrollable` fixtures. It checks geometry, grab/click
protection, rail recentering and continued dragging, native line/pixel wheel
behavior for direct and `responsive` layers, hover-cursor restoration and
covering-overlay isolation, without an OS window or system clipboard.

`runtime/memory/editor/tests.rs` checks minimal scrolling and wrapping for
arrows and Tab, search/cell-change scroll resets, and Enter selection.

The opcode picker view test applies native scroll and focus operations to the
real picker, then sends a drag whose press and movement share a frame's final
cursor position. It verifies dragging and wheel scrolling after search focus,
both at the top and after keyboard scrolling.

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
- Unix/macOS: `./scripts/generate_icons.sh` (requires ImageMagick and either
  Apple `iconutil` or Python 3 for ICNS encoding)

The Windows build script does not regenerate `icon.ico` automatically –
it only embeds it. A stale `icon.ico` will be silently shipped if you
forget to rerun the generator.
When a Windows host cross-checks Linux or macOS, the build script skips PE
resource compilation for the non-Windows target.

## Manual smoke checks for the UI

- In each external device (Monitor, Floppy, HDD, Network, Printer), use Tab and
  Shift+Tab to traverse the toolbar in each direction, including wrapping at both
  ends. Check the blue outline, first/last entry, and Enter/Space activation. After activation,
  the selected button must retain the same background fill as the default button
  in file confirmation dialogs, including after repeated Enter/Space presses.
  Tab/Shift+Tab must remove that focus fill and outline the next target. Repeat after
  detaching; Attach and Pin must be included, and Tab in the main window must
  still use the main editor's navigation. Click a toolbar button and check that
  the next Tab continues after it. In Monitor's HEX popup, traversal must stay
  on Filter and Close. Check that disabled HDD Create/Delete and busy-printer
  Print actions are skipped. Repeat navigation while the emulator is running.

The two tests in `app/device_keyboard/tests.rs` cover repeated activation,
selection after pointer actions, HEX wrapping, window/modal ownership, and
disabled HDD/printer commands. They drive the runtime event handler and actual
actions. `cargo test -p kr580 --bin kr580 device_` also runs the shared capture
test, which checks that these keys cannot edit a focused input underneath the
device while Ctrl+Tab remains outside the device ring.

Some UI behavior cannot be unit-tested directly with iced 0.14, so it is
worth eyeballing after touching `crates/ui`:

- at the default `1180×720` window size, confirm the left CPU stack and the
  mux/status column retain their original wide separation; maximize the window
  and confirm the two columns remain centred as a compact group with a 72 px gap;

- launch the `kr580` binary and confirm there is no white flash on
  Windows (cloak/uncloak via DWM, see `docs/ui_app.md`);
- run `cargo build --release -p kr580` and double-click
  `target/release/kr580.exe`: no console window should pop up;
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
- open a `.krs` file and confirm Cancel has a light fill and its normal border,
  matching the confirmation dialogs; a white focus border appears only after Tab/Shift+Tab;
  Enter must cancel. Reopen and traverse Tab/Shift+Tab in both directions:
  Cancel, Open, and Start address must each show a light border. The address
  field must accept typing only while focused; moving to a button must remove
  its caret. Repeat after clicking inside the address field, including without
  editing it, and in Save as with both Start and End address fields;
- in the `.krs` address fields, enter `1a2f` and confirm `1A2F`; try a fifth
  digit, `G`, Cyrillic letters, punctuation, and an overlong paste, and confirm
  the previous value remains intact. Verify Backspace can clear the field and
  both Start and End use the same validation;
- open a `.krs` file from File → Open, enter a start address, and confirm its
  bytes appear at that RAM address; use Save as with `.krs` to verify the
  selected inclusive RAM range is written without a header;
- open native dialogs from the main window, detached Monitor/Floppy/HDD,
  Settings, Import/Export, and the installer; confirm each dialog belongs to its
  owning window and cancellation leaves state unchanged;
- on Windows, detach Monitor, Floppy, HDD, Network, and Printer; confirm their
  title bands still move the windows, borders and corners cannot resize them,
  and dragging them into top, side, or corner Snap zones does not maximize or
  tile them;
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
  and `.krs` associations follow their checkbox, the install root contains `app/kr580.exe`,
  `app/uninstaller.exe`, and `bin/kr.exe`, no installed `app/k580-installer.exe`,
  and upgrading a manifest-owned legacy root removes `app/k580.exe` plus any
  association owned by that exact executable, while the same path in a folder
  without `install.json` is preserved;
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
  on Windows or `bash scripts/build_installer.sh` on Linux/Unix and confirm
  a standalone `KR580-Setup-*` artifact appears under `dist/`; on macOS run
  `bash scripts/build_macos_dmg.sh` and confirm its self-validation mounts a DMG
  containing the executable `KR580.app`, both ICNS resources, the canonical
  bundle identifier, and an `/Applications` link; for release
  packaging, also smoke-check `--target` builds and `scripts/package_installer_deb.sh` for one Linux target, confirm the
  Debian control metadata contains `desktop-file-utils`, `libdbus-1-3`,
  `shared-mime-info`, `xdg-utils`, and `zenity`, and open a file dialog in the
  Debian, Snap, and Nix artifacts;
- inspect the built Snap and confirm `bin/kr580` and `meta/gui/kr580.desktop`
  exist, the app command is `bin/kr580`, and neither `k580-installer` nor
  `k580-uninstaller` is present; launch it, persist a setting across refresh,
  open `.580`/`.krs` through the app, and verify the association action is
  absent under confinement;
- run `cargo run -p kr580 --bin kr -- nonexistent.580` and confirm
  the GUI launches with a localized "Файл не найден" error notice;
- on Linux, run `cargo run -p kr580 --bin kr -- -r`, then confirm
  `$XDG_DATA_HOME/mime/packages/application-x-kr580.xml` and
  `$XDG_DATA_HOME/applications/kr580-file-handler.desktop` were created (using
  `$HOME/.local/share` when `XDG_DATA_HOME` is unset), the
  handler does not appear as a second application-menu entry, and `.580` and
  `.krs` files open with `kr` from the file manager; confirm
  `xdg-mime query default application/x-kr580` returns
  `kr580-file-handler.desktop`;
  register twice, remove the association and confirm the preceding default is
  restored; choose another default externally and confirm removal preserves it;
- on macOS, run `cargo run -p kr580 --bin kr -- -r`, then confirm
  `~/Applications/KR580.app` uses bundle identifier `dev.kr580.emulator`,
  executable name `kr580`, the generated version from the canonical plist,
  separate snapshot/subprogram UTIs, valid application/document ICNS files,
  and successful registration through `LSRegisterURL`; double-click one file
  of each type in Finder and confirm the existing app receives the document,
  including the load-address dialog for `.krs`; change one setting and confirm
  it is written under `~/Library/Application Support/KR580`, not inside the app;
  in a Portable installation verify `<install root>/KR580.app` is created and
  settings remain under `<install root>/data`; with a foreign
  `~/Applications/KR580.app`, System setup must stop before copying its payload;
  `kr -u` must report unsupported instead of claiming success;
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
  On and Off segments for Follow PC, memory-operand highlighting, and Show file
  name; open a `.580` or `.krs` file and confirm that enabling the option centres
  only its base name in the top title bar, disabling it clears the title, and a
  long name keeps its extension after middle shortening; confirm a soft lower
  fade and small down chevron reveal that General has more content without
  drawing a scrollbar; verify the smaller wheel step and smooth touchpad pixels,
  then scroll to the `.580` / `.krs` association row. The lower hint must vanish
  at the bottom, the mirrored upper hint must vanish at the top, and both must be
  visible between them without jumps. At 100% and 125% display scaling, verify
  the right dialog frame stays continuous beneath both fades. Repeat in Russian
  and English; in Appearance, continued wheel-down at the bottom must remain stationary. Verify
  the association row plus Reset, Cancel, and Save show a white
  border without a fill change, and that Enter or a mouse click clears the border before
  activation; open the language dropdown and confirm its anchor gains the same
  active fill as an opened printer selector; open the Reset confirmation and
  confirm Cancel starts filled without a white border, then Tab/Shift+Tab removes
  the focus fill and draws only the white border; in the Sidebar, verify
  Tab/Shift+Tab moves the category cursor
  without changing the page until Enter; finally confirm the `.580 and .krs file
  associations` row shows `Add` when either association is missing and `Remove`
  when it is present, then click it and verify the button label flips and the
  OS association is created/removed;
- in Settings → External Devices, confirm Monitor layout follows the printer
  settings, Unified is the default, and Tab/Shift+Tab visits both layout choices
  before the network settings. Preview Split and Cancel to restore the previous monitor
  view; Save and restart to retain Split, then Reset to restore Unified. A
  temporary split/merge toggle inside the monitor must not change the saved
  default when an unrelated preference is saved;
- make the current file dirty and invoke Open, New, Import, Close, and HDD
  deletion confirmations; in each shared confirmation overlay verify Cancel is
  initially filled without a white border, the first Tab/Shift+Tab changes the
  indication to a white border without focus fill, and Enter or pointer input
  hides the border before activating the chosen button;
- with the attached Monitor open, open and cancel Export; reopen Monitor, then
  open and cancel Import; confirm each modal replaces the Monitor panel and
  cancellation returns to the main app instead of restoring it; while Export
  is open invoke Import, then invoke Export from Import, and confirm each
  command replaces the current modal;
- open Export and Import and use Tab/Shift+Tab from their native text inputs and
  buttons; confirm captured keyboard events still traverse each custom wrapping
  focus ring in both directions, draw a white outline on the active tab,
  target add/delete button, field, checkbox, or footer control without replacing
  its selected fill/check,
  skip the unavailable Import target selector and disabled Import action, move
  directly from Export's Text file tab to its target selector without an
  invisible intermediate stop, confirm the opened Export target panel keeps a
  six-logical-pixel gap below its anchor like the language and printer selectors,
  and let Enter or pointer input clear the
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
  fully rendered and vertically centred in their highlight, and every target
  stays on one clipped, middle-shortened row; toggle the selector repeatedly
  and verify neither the footer geometry nor the anchor highlight changes;
- in the memory cell editor, confirm `Enter`, `Ctrl+Enter`, `Alt+Enter`,
  and `Tab`/`Shift+Tab` follow the table in `docs/ui_app.md`; assign different
  shortcuts to pattern search and cell replacement and confirm each remains
  limited to its own editor context; assign a printable chord such as `Alt+F`
  and confirm it does not append `F` to the active field; select an
  `IN`/`OUT` port operand below the first visible page, open its device with
  `Alt+Enter`, and confirm the selected address and scroll position do not
  reset to `0000`;
- paste `3E 41 D3 03 76` into a memory value field and confirm the five
  consecutive cells update immediately without first deleting the
  existing two-digit value; malformed or overflowing input must not
  write a partial sequence and must show a short localized status
  without repeating the pasted text;
- in the memory list, select a row without entering its editor and confirm Tab
  walks down through addresses while Shift+Tab walks back up without activating
  edit mode; then focus the inline editor and confirm the same traversal leaves
  each destination empty with its stored byte shown as the placeholder;
- in the inline memory list, confirm the scrollbar thumb is compact, does not
  jump when grabbed off-centre, appears when hovering either the thumb or an empty
  part of its rail, moves by only a few addresses for a minimal drag, catches the
  pointer within 12 px, stays under it for a fast drag, reaches both track ends
  without stutter, and leaves wheel/touchpad sensitivity unchanged;
- in the RAM list, opcode picker, and Monitor byte stream, grab the unpainted
  left portion of the 8 px target and confirm dragging starts without a jump;
  click the empty rail,
  confirm the thumb centres on the pointer and follows a continued drag without
  selecting underlying content, then click just outside the rail and confirm the
  scrollbar does not capture it and RAM/opcode row actions still work;
- in all three lists, scroll with the wheel or touchpad directly over the painted
  thumb and its unpainted grab area, confirm sensitivity matches scrolling over
  the content, and confirm hovering, dragging, and a covering popup still work;
- in the opcode picker, type part of an opcode or mnemonic, confirm
  ArrowDown/Tab and ArrowUp/Shift+Tab move the highlighted filtered row
  with wrapping and keep it fully visible without moving already visible rows;
  after Tab scrolls the list, immediately quick-drag the thumb and use the
  wheel, and confirm both keep working;
  scroll down manually, change or clear the search, and confirm the first result
  is shown at the top even after a short or empty result list; open the picker
  for another RAM cell and confirm it starts at the top; confirm Enter writes
  the highlighted opcode to the selected memory cell;
- in the opcode picker, confirm the thumb has the same 28 px length as the RAM
  viewer's thumb, reveals on rail hover, keeps its grab point, and reaches both
  ends of the full and filtered lists; confirm wheel/touchpad scrolling still
  works over the list and rail, filtering to a few or no matches hides the
  thumb, and clearing the filter or reopening the picker keeps the thumb in
  sync with the visible options;
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
  its final text scale immediately without stretching the Setup window; drag the
  Printer, Setup, and Properties from the blank inset above their header controls
  and confirm the exact grabbed window moves, then confirm every header button
  still activates without starting a drag; while
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
