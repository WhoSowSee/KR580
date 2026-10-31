# Manual testing

Use the [automated checks](testing.md) first. These scenarios cover native
presentation, real drivers and OS integration that fixtures cannot establish.
Select the sections affected by the change and record OS, language, display
scaling, build/commit and any skipped environment.

Use disposable documents and installation roots for destructive or update
scenarios. Keep pass/fail evidence separate from automated test results.

## Launch and native windows

1. Launch the GUI directly, through `kr`, and by opening a `.580`/`.krs` file.
   Check the title, application/file icons and current version. A release GUI
   should not flash a console; debug GUI runs should retain tracing output.
2. On Windows, inspect first launch/open/run for a white client-area flash or
   cold text redraw. Repeat in light/dark themes and at 100%, 125% and 200% scaling.
3. Maximize/restore, move across displays and inspect long file names and paths.
   Headers, status values and focused controls must stay within their frames.
4. Detach, pin, attach and close every device. On Windows repeat reuse several
   times; a hidden window must not reappear when another window is pinned.
   Other platforms must cleanly close/reopen their native windows.
5. Drag each detached header, including its blank top inset. Buttons must activate
   without dragging a different window. Device windows must remain non-resizable.
6. Drop supported files over the main window, including near edges and while dirty.
   The cursor hint must stay visible and within bounds; leaving cancels hover.
   Unsupported drops must preserve the document and show a localized error.

## Shared focus and selectors

1. In both languages, traverse every device toolbar forward and backward.
   Skip disabled commands; include detached Pin and Close. Verify the first
   forward/backward stop, wrapping, repeated Enter/Space and pointer-selected origin.
2. Tab shows an outline; activation retains the appropriate selection fill.
   Subsequent traversal moves the outline without changing unrelated toggles.
   Native-window input must not edit a text field in another window underneath.
3. Open language, import/export, printer and opcode selectors with long lists.
   Each wheel line should move one option, touchpad deltas should remain smooth,
   and first/last options must be reachable. Check fractional bottom boundaries.
4. Popups must overlay following controls, retain their final border and stay
   inside the window. Toggle repeatedly without moving the anchor or footer.
   Arrow movement should preview a highlight; Enter commits and Esc dismisses.
5. Check both pointer and keyboard activation after input focus changes and thumb
   dragging. A hidden input must not retain a caret or receive typed characters.
6. After Windows clipboard dependency updates, copy and paste Cyrillic, Unicode
   and a hex byte sequence between KR580 and another application. Check retries
   while the clipboard is briefly busy in another process.

## Memory, registers and execution

1. Paste `3E 41 D3 03 76` into a memory value field and inspect all five bytes
   without first deleting the old value. Malformed text and a block overflowing
   `FFFFh` must not partially write RAM or echo the entire paste into an error.
2. Exercise the context-specific controls in
   [Memory editing](ui_app.md#memory-and-register-editing). Pattern search must
   retain its fragment and wrap; replacement must retain the original placeholder.
   Empty replacement and Esc must preserve/recover the actual byte.
3. Rebind pattern search and replacement separately, including a printable chord
   such as Alt+F. Each action must stay in its context without inserting F into
   the focused input. Repeat letter shortcuts with a Russian keyboard layout.
4. Follow either byte of a 16-bit address operand, return to it, and verify the
   original scroll position. Open an IN/OUT device from an off-screen-page
   operand; selection and scroll must not reset to `0000h`.
5. Toggle stack view repeatedly and return with its action or Esc. Preserve the
   original viewport and restrict searches to the selected memory area.
6. Edit registers from the right panel and schematic copies. Verify shared drafts,
   Enter/Shift+Enter traversal, edge behavior, Esc, grid arrows and replacement.
   Switching to RAM must release the register target and vice versa.
7. Step an instruction and its individual T-states, including conditional calls,
   port output and HLT. Readouts must agree with
   [Execution and schematic](ui_execution.md#timing-panels); a partial step must
   not fire the port action early.
8. Run a loop at all tiers, change speed while running, then Stop. Check High
   paced animation, Max's coalesced updates and final follow-PC after HLT/error.
   Do not judge CPU throughput from UI frame count or Max's nominal 1000 label.
9. After HLT, attempt run/step, dismiss or wait out the notice, and verify controls
   stay blocked. Clear HLT without moving PC/RAM selection; reset CPU separately.
   Undo/redo must preserve the relevant state and dirty baseline.

## Documents, modals and exit

1. Load/save `.580` with nonzero registers and SP; verify they return. Check the
   exact format against [Persistence](persistence.md#580), not an assumed full
   runtime snapshot. Load `.krs` near `FFFFh`, then save the actual installed range.
2. Edit a register and RAM outside a `.krs` save range. Saving that range must
   leave the session dirty; cancelling Open/New/Import/Close must preserve it.
3. Export a range containing zero bytes to TXT and XLSX, import onto nonzero RAM
   and check both the selected range and untouched outside cells. Exercise reserved
   and duplicate TXT names, multiple sheets/sections and one-cell ranges.
4. Open/cancel Import and Export over an attached device, and replace one modal
   with the other. Cancellation returns to the main app, not an older panel.
5. In Import, test empty/selected/error states, file choosing and native drops.
   Unsupported extensions stay in the modal. Target loading disables confirmation;
   changing files or closing before completion must not revive an old result.
6. Traverse Import/Export from real inputs in both directions. Skip unavailable
   targets/actions; retain the same dialog height when selecting a file or
   opening a target popup. Long targets must remain readable without a visible rail.
7. In discard and subprogram confirmations, Cancel starts selected. Keyboard
   traversal changes its indicator to an outline; Enter, Esc and backdrop click
   must have their documented effects without reaching controls underneath.
8. Close after submitting storage bytes, a printer export and a file save.
   Accepted writes must finish. Exercise a controlled output failure: close must
   report failure and leave recovery/inspection available. Repeat updater restart.

## Settings

1. Search by category, label, individual theme and shortcut action in RU/EN.
   Parent matches retain the complete control; child matches narrow it.
2. Traverse Search, Sidebar, Content and Footer with Ctrl+Tab and local Tab rings.
   Sidebar arrows must not select until Enter. An open language selector previews
   without applying until confirmation. Focus must not leak into the schematic.
3. Preview language/theme/speed, Cancel and inspect restoration. Save keeps the
   dialog open. Edit again while storage is pending; confirmation must retain
   later drafts and give Cancel the last successfully submitted baseline.
4. Simulate a settings write failure. Preserve the prior baseline and suppress
   stale success notices. Save/Reset/Cancel should be disabled during storage
   without moving the footer; other draft controls remain editable.
5. Check Show file name, Follow PC and operand highlighting in both directions.
   A long file name retains its extension; a fresh document has no file title.
6. Preview Monitor layout, Cancel to the pre-open view, then Save/restart.
   Saving an unrelated setting must not persist a session split/merge toggle.
7. Scroll General/Appearance at 100% and 125% scaling, including fractional bottom
   overflow. Fades appear only where content remains and never cover the frame.
   Search hides the hints; changing search must not destabilize scrolling.
8. Open Reset confirmation, cancel, then confirm. Repeat shortcut-only reset.
   Verify notices can be replaced/dismissed smoothly and failed writes stay errors.

## Help and menus

1. Switch RU/EN and inspect corresponding Help categories/articles, including
   the snapshot and speed descriptions. Search by title and article text.
2. Open a result, verify its branch is expanded and search cleared. Rapidly edit
   or delete a query; stale results must not replace the latest language/query.
3. Select words/lines with pointer gestures, select all and copy. Articles remain
   read-only, inline bold markers are absent from copied text, and short overflow
   ranges scroll without jumping or showing an insertion caret.
4. Navigate top menus by arrows and Tab. Hover switches an open dropdown only;
   Settings is a category stop without a dropdown, and disabled items are skipped.
5. Open About/Changelog, change the selected release and return to all releases.
   Verify both embedded language histories and long scrolling.

## Monitor and storage checks

1. Send text and graphics commands to Monitor. Unified/split views must preserve
   both layers, CP866 Cyrillic/box glyphs and the modelled intensity palette.
   Long split text wraps at panel width without inserted framebuffer row breaks.
2. Filter the HEX stream, inspect its count, close/reopen, and use its Filter/Close
   keyboard ring. Clear the monitor and verify all visible device buffers reset.
3. Save each supported image format. Check logical resolution and both layers,
   ownership of detached file dialogs, cancellation and write failure.
4. Attach/clear/detach floppy and HDD files. Buffer-only debug must preserve file
   data; normal writes must reach the attached file. Saving the visible buffer
   must wait for actual replacement before reporting success.
5. Modify and atomically replace backing files externally while previews are
   visible. Refresh changed data without rereading unchanged files on every tick.
   Reattach during a pending preview/delete and verify stale completions are ignored.
6. Create/delete `hdd.kpd` with confirmation. Verify startup defaults are under
   External Devices and window-only directory changes do not overwrite them.

## Network checks

1. Connect two instances over loopback, exchange bytes in both directions, and
   inspect HEX/text, counters and EOF/error state while CPU execution is paused.
2. Reconfigure client/server mode and endpoints; the old connection/listener
   must be cancelled before the replacement starts. Clear must preserve endpoint,
   connection and worker. Closing inspection alone is not worker shutdown.
3. Check nonfatal empty reads, long RX buffers and resumed draining after reads.
   Buffer inspection remains bounded and ordered.

## Printer and driver checks

These require an installed Windows printer driver. Use a known harmless spool
for any physical/PDF print job and record which driver was tested.

1. Open setup from Settings and from Printer. Check asynchronous metadata,
   paper/source lists, orientation and preview without initial resize/clipping.
   Global setup persists defaults; the header gear changes only the session.
2. Exercise custom and system setup modes. In Properties visit available tabs,
   edit options/parameters, accept/cancel and save/reload a profile. Restart to
   check the persisted global configuration.
3. With Russian and English driver/app combinations, inspect statuses, bins, paper
   names and vendor features. Recognized labels translate both ways; unknown
   foreign labels use neutral IDs. Proper printer/profile names stay unchanged.
4. Check all setup/property focus rings, selector highlights, enabled controls
   and parameter input alignment. Moving to a button must remove the old caret.
5. From detached Printer open Setup and Properties. Confirm separate native
   surfaces at the bounds in [Device windows](ui_devices.md#detached-printer-dialogs),
   stable initial scale, correct header dragging and no resizing of the owner.
6. Attempt closing Setup while Properties is open. Properties receives focus
   and an attention pulse; its own close returns focus to Setup. Closing the
   owner cleans up descendants.
7. Print CP866 text with CR/LF, tabs, blank lines and long lines. Verify Busy to
   Ready/error, the selected driver configuration and preserved spool.
   Cancelling native/file prompts must return Ready without raw OS error text.
8. Clear, detach, pin and reopen; retain the chosen target. Long names must not
   push footer values or the settings clear action outside the window.

## Updates

1. Check in both languages. Spinner/progress stays inside Help; Up to date
   expires after three seconds and clears on menu close. Available/download/ready
   states survive menu closure.
2. In a disposable older installation, download an official matching update.
   Cancel unsaved-change confirmation, then test explicit restart and ordinary
   exit separately. Only restart should relaunch and reopen the saved document.
3. Verify user files/settings and installation mode survive replacement. Exercise
   a controlled replacement failure and inspect rollback/recovery information.
   Distinguish a real GUI update from local HTTP/process fixtures.
4. Repeat for native Windows/Linux installer layouts and macOS bundles. Package
   manager/source builds must offer the release page rather than replace their
   managed executable. See [Updates](updates.md).

## Installation and platform integration

1. Build native distributions using [Installer](installer.md#build-distribution-packages).
   Inspect setup/uninstaller icons and installed `app/` versus `bin/` layout.
2. Test System and Portable with PATH/association/shortcut choices, cancellation
   and a failing stage. Preserve unrelated files, settings and foreign registry
   or desktop entries. Upgrade a manifest-owned legacy root separately from an
   unmanaged folder with colliding binary names.
3. On Windows check current-user/all-users destinations, Start Menu/Desktop,
   Apps & Features command, no shortcut-creation console flash and association
   Add/Remove updates while Settings is open.
4. Run uninstall: real stages must advance in order, Close stays disabled until
   success/progress completion, and errors remain visible. Post-exit deletion
   must remove only the managed installation.
5. On Linux check Desktop Entry/MIME caches, paths containing spaces/percent/shell
   characters and ownership-aware default restoration. A later user default
   choice must survive unregister; missing tools must report failure.
6. On macOS verify drag-installed and owned wrapper bundles, Finder `.580`/`.krs`
   activation, both ICNS icons and Application Support settings. `kr -u` must
   report unsupported; choose replacement defaults through Finder.
7. In Snap verify writable settings across refresh, in-app file opening, hidden
   association action and the package-owned desktop entry. In Nix/RPM/Arch verify
   wrapper environment, writable XDG settings and package-manager ownership.
8. Use the disposable-container [native package smoke checks](linux_packages.md#verification-and-release-wiring)
   for Fedora/Arch installation/removal, GUI startup, caches and retained user data.
