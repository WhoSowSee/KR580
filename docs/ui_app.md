# App and UI

`kr580` is an iced multi-window daemon. `DesktopApp` renders the latest
`AppSnapshot` and sends commands to the backend actor; widgets do not execute
instructions or serialize program files.

Related guides:

- [Architecture](architecture.md): actor, worker and completion contracts.
- [Execution and schematic](ui_execution.md): speed, HLT, follow-PC and CPU readouts.
- [Device windows](ui_devices.md): attached/detached devices and printer dialogs.
- [Settings](ui_settings.md): live preview, focus and save/cancel semantics.
- [Persistence](persistence.md): file formats and atomic writes.
- [Manual testing](manual_testing.md): native presentation and interaction checks.

## State ownership

`app/state.rs` composes the following states around the backend handle and
snapshot. Their definitions live under `app/state/`.

| State | Responsibility |
|---|---|
| `DocumentState` | File identity, dirty baseline, undo, discard and file-drop state |
| `MemoryEditorState` | RAM viewport, inputs, opcode selection, search and operand return |
| `RegisterEditorState` | Selected register and inline editing target |
| `InteractionState` | Modifiers, focused inputs, replacement and pointer bookkeeping |
| `ExecutionState` | Run state, speed and HLT/follow-PC feedback |
| `ExportDialogState`, `ImportDialogState` | Format, target, selection and pending file work |
| `PreferencesState` | Confirmed settings, settings draft, language, theme and shortcuts |
| `PrinterSetupState` | Setup/properties drafts, asynchronous driver work and native child windows |
| `DevicePanels` | Inspection views, previews and tool-window presentation |
| `ShellState` | Main window, menus, notices, Help, updates and shutdown |
| `RequestState` | Pending backend requests, UI file worker and deferred effects |

CPU and device mutations belong to the actor. Local drafts and navigation may
change immediately; saved paths, dirty baselines and success notices follow the
matching successful completion.

## Event handling

`app/messages/` groups application messages by concern; `app/update.rs` routes
them to the relevant handlers. Completion messages are handled before modal
capture, so closing a dialog cannot strand accepted work.

The UI receives backend events through two paths:

- Full snapshots use a latest-state mailbox and replace intermediate snapshots.
- Command completions, errors and execution boundaries retain their typed events.

`CommandFinished` is correlated by `RequestId`. A snapshot alone is not proof
that a particular edit or save completed. Older actor revisions and superseded
document completions cannot overwrite a newer document, draft or selection.
`WorkerStopped` clears unresolved request/history slots and reports the failure.

CPU edits and undo reserve their place in the bounded timeline on submission.
The actor returns the actual before/after change; confirmation fills the slot.
Failed admission or execution removes or restores the appropriate pending entry.
Text edits made while a CPU reply is pending retain their chronological position.

`runtime/file_work/` owns lazy FIFO work for UI filesystem operations, settings
read-modify-write, previews and image encoding. Each completion carries the
request context needed to reject stale results. Settings event handlers read the
confirmed cache rather than opening the settings file on every input event.
See [worker ownership](architecture.md#workers-and-completions).

## Main window and menus

The left schematic shows CPU state; the right side holds RAM, memory/register
editors and execution/reset actions. The latest action or path appears in the
schematic header. HLT has its own indicator and execution notice.

File, MP-System, View and Help own dropdowns; Settings opens its dialog directly.
Hover switches between already-open dropdown categories but does not open a
closed menu. Arrow keys navigate enabled rows and adjacent dropdown categories.
Tab/Shift+Tab also includes Settings in the category ring and skips separators
and disabled actions. Enter activates the selected action.

The optional title-bar file name uses the base name of the current `.580` or
`.krs` file, shortened in the middle to retain the extension. It remains part of
the draggable title surface. Unsaved new documents leave that title empty.

Menu rows and tooltips display current configured shortcuts. Letter shortcuts
use physical key positions, preserving their meaning across keyboard layouts.
Focused text inputs retain native editing chords such as Ctrl+A.
Default bindings are listed in the [in-app shortcut reference](../crates/ui/src/i18n/help/en/shortcuts.md).

## Memory and register editing

Memory inputs accept bounded hexadecimal values; invalid edits preserve the
previous input. A space-separated byte sequence writes one validated RAM block.
Malformed input or a block past `FFFFh` is rejected without a partial write.

Rows can be selected without opening an editor. Replacement mode clears the
input and keeps the current byte as a placeholder; accepting an empty replacement
preserves that byte. Esc first discards an unsaved inline value.

Operand highlighting distinguishes opcodes, addresses, immediate data and I/O
ports. Operand navigation uses the same classification and works even when
colouring is disabled. Black-and-white themes use a contrast-ordered grayscale
palette without changing navigation.

### Memory cell editor

| Input | Action |
|---|---|
| Enter in address | Jump to the typed address and retain the original search fragment |
| Enter in value | Write the byte at the selected address |
| Pattern-search action, default Ctrl+Enter | Find the next address containing the retained hex fragment; Shift searches backward |
| Alt+Enter in address | Jump to the entered address |
| Alt+Enter in value | Apply the value and reveal the entered address after confirmation |
| Up/Down in address | Move one address |
| Up/Down in value | Adjust the draft byte within `00`–`FF`; Enter is still required to write it |
| Tab/Shift+Tab | Traverse the two inputs, clearing the destination for replacement while retaining its placeholder |
| Alt+Q / Alt+E | Go to `0000h` / `FFFFh` in the normal RAM view |

Editing the address field starts a new pattern search. Searches wrap inside the
current memory view. The cached fragment survives jumping from `FF` to `00FF`,
so repeated search continues using `FF`.

### RAM list and opcode picker

| Input | Action |
|---|---|
| Cell-replacement action, default Ctrl+Enter | Enter replacement mode without changing RAM or selection |
| Enter in inline editor | Apply the value; an empty replacement preserves the byte |
| Tab/Shift+Tab, Up/Down | Move selection, preserving whether editing/replacement mode is active |
| PageUp/PageDown | Move selection by 16 addresses |
| Alt+Enter on either address operand byte | Follow its little-endian 16-bit target |
| Alt+Enter on an IN/OUT port operand | Open the matching device, retaining the RAM selection and scroll |
| Alt+Shift+Enter | Return to the operand that initiated the last address jump |
| E or the opcode button | Open opcode/mnemonic search |

Address and port actions require an unfocused inline editor. An ordinary data
cell does not provide an operand target. Port actions do not replace the saved
address-return slot.

The opcode picker searches documented encodings by byte or mnemonic. Down/Tab
and Up/Shift+Tab wrap through matches; Enter writes the selected byte and Esc
closes without writing. Opening for another cell or changing the query selects
the first match. The popup stays within the RAM viewport and scrolls the
highlight into view. Wheel lines move by one row; touchpad deltas remain smooth.

Stack view restricts the list to `FF00h`–`FFFFh`. Turning it off restores the
saved viewport; repeated enable does not overwrite that return position.

### Register editor

The right editor and schematic copies share the selected register's draft.
Typing changes the draft; Enter submits the actor edit. The value field targets
A when its register-name field is empty and the value is valid.

Tab/Shift+Tab traverse the schematic A/B/C row and B/C/D/E/H/L grid. Unmodified
arrows remain inside the active visual group; Ctrl+Arrow (Command+Arrow on macOS)
provides equivalent grid navigation. Other captured modifiers retain native
text-editing behavior.

Enter/Shift+Enter commits and advances within the same group; at an edge it
closes the inline editor and retains selection. Esc discards the draft and also
retains selection. Selecting RAM clears the active register target and vice versa.

## Document flows

Native `rfd` dialogs are parented to the owning ready native window. Detached
monitor, floppy and HDD actions use their device window; application workflows
use the main window. Cancelling a picker leaves the current document unchanged.

The main window accepts `.580` and `.krs` drops case-insensitively. Hover shows a
passive scrim and cursor hint; unsupported files produce a localized error
without discarding the current document. A `.krs` drop opens the address dialog.
Device windows do not accept program drops.

Snapshot/subprogram open, save and import/export run through correlated backend
requests. A full-document replacement blocks conflicting CPU commits until it
finishes. Imports and subprogram loads apply to the current actor state, so
unrelated edits made during the read survive. Generation checks discard obsolete
results, including their errors.

### Unsaved-changes modal

Open, New, Import, dropped-file open, close and updater restart pass through the
dirty gate. The pending action owns its exact path and confirmation copy.
Cancel, Esc and backdrop click preserve the document. Confirming Open or Import
starts selection without marking the document clean; picker cancellation leaves
the next discard warning intact.

CPU save completions carry the captured state actually written. A `.krs` save
updates only its inclusive RAM range in the saved baseline. Register changes and
RAM outside that range remain dirty. Failed saves never advance the baseline.
Subprogram load records its actual installed end address from the completion,
without a metadata estimate or synchronous dispatch timeout.

Confirmation starts on Cancel with a filled logical selection. Tab/Shift+Tab
switches to a keyboard-only outline and wraps between enabled actions. Enter or
pointer activation clears that outline before performing the action. Modal
capture blocks underlying editing while allowing ticks, completions and window
bookkeeping to continue.

### Import, export and subprogram dialogs

Import owns the native chooser and a `.txt`/`.xlsx` drop zone. Target enumeration
is asynchronous; confirmation stays disabled until a supported file is ready.
XLSX selects a sheet and multi-section TXT selects a block. Plain TXT imports
its single model. A changed selection or closed dialog invalidates earlier work.

Export owns per-format targets with paired names/settings. Each target selects
an inclusive RAM range, registers and flags; XLSX also selects columns. The target popup
floats above the form; opening it does not resize the dialog. Target naming and
round-trip guarantees are defined in [Persistence](persistence.md#exports).

Import/export replace any open attached device panel and replace each other.
Cancelling returns to the main app. Tab/Shift+Tab traverses enabled controls in
a closed ring; Enter or pointer input hides the keyboard outline. Popup wheel
movement follows rows and preserves access to both ends.

Subprogram Open asks for a start address; Save asks for an inclusive range.
Address fields accept at most four hex digits and normalize lowercase to
uppercase. Empty fields show `0000` as a placeholder. Focus traversal moves the
actual caret between inputs and buttons, and initially selects Cancel.

## Help, About and changelog

Help opens through Ctrl+H, F1 or its menu action. Russian and English share the
same task-oriented tree, with articles included from `i18n/help/{ru,en}/`.
Search covers titles and article bodies. Results contain clickable breadcrumbs
and short previews; selecting one clears search and expands its parent branch.
Search results are applied only if generation, language and query still match.

The article pane permits selection and copying but rejects edits and caret
movement. Supported inline bold markers are rendered and omitted from copied
text. Wheel and touchpad scrolling use the containing pane's pixel offset.
Esc, backdrop click and Close dismiss Help.

About displays the package version and repository link. Its Changelog action
opens the embedded release history, with release selection and an all-releases
view. Packaging copies and release headings follow
[repository maintenance](maintenance.md#release-checks).

## Confirmed application exit

`ShellState.shutdown` progresses through `Idle`, `Draining`, `Pending` and `Ready`.
After discard confirmation, close/restart stops execution and rejects new work.
The shell keeps processing completions while accepted UI jobs, backend requests
and file-association work finish. It then sends the correlated backend `Shutdown`.

Only its successful receipt permits native main-window close and `iced::exit()`.
File-work or final device errors cancel exit and leave the app available for
inspection, export and another close attempt. An already-closed main window still
enters this protocol. See [device shutdown](devices.md#confirmed-shutdown).

## Platform presentation

Windows starts the main window hidden, cloaks it with DWM on opening, and
uncloaks after two redraw notifications. A covered startup layer primes UI and
monospace glyphs during those frames. Other platforms use the same theme
background with no-op DWM helpers.

During a Windows OS file drag, native pointer polling supplies DPI-correct
coordinates because normal cursor events may stop. The drop hint is withheld
until a valid position is available and stays within the window.

Release `kr580.exe` uses the GUI subsystem; debug GUI builds retain a console
for tracing. The `kr` launcher remains a console binary for help, version and
integration output, then launches the GUI. The GUI accepts an optional `.580`
or `.krs` path. Installed layouts, association ownership and setup commands are
defined in [Installer](installer.md); current CLI switches are listed by `kr --help`.

UI labels use the platform UI font and live readouts use monospace. Monitor
pixels use the separate byte-indexed [bitmap font](assets.md#monitor-bitmap-font).
Window icons and Windows per-binary resources follow the
[asset pipeline](assets.md#where-the-assets-are-consumed).
