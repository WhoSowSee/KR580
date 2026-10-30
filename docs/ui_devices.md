# Device windows

These views inspect the actor's device snapshots and send explicit device
commands. Port protocols, status codes, retained-buffer limits and shutdown
belong to [Devices](devices.md).

## Shared window lifecycle

Monitor, floppy, HDD, network and printer open from View, Quick Access or their
configured shortcuts. They appear as an attached overlay or a borderless native
window. Closing one device does not close the emulator.

`ToolWindowState` owns the native ID, opening/ready state, attachment and pin
preference. Native windows are not created at application startup. Windows
creates them lazily on first detach and hides/reuses them on attach or close;
other platforms close and reopen them.

Both presentations share the same snapshot and session view state. Detached
windows are non-resizable. Storage, network and printer use their fixed panel
bounds; Monitor derives its detached size from the main window's attached
content area. Dragging the blank header moves the exact owning native window.
Header buttons retain priority over dragging.

Pinning applies only while detached. Attach/close resets the pin. Ready native
windows own their file dialogs; attached workflows fall back to the main window.
Opening a device from an I/O operand forwards its complete native-window task
while retaining the RAM selection and scroll position.

### Keyboard navigation

Each toolbar has a closed Tab/Shift+Tab ring in visual order, including
detach/attach, detached Pin and Close. Initial forward/backward traversal selects
the first/last enabled action; disabled commands are skipped.

Tab draws the keyboard outline. Enter/Space activates the action and retains a
filled logical selection; pointer activation also becomes the next traversal's
origin. A subsequent Tab moves the outline. Closing resets the selection;
attach/detach retains the corresponding toolbar action.

Routing uses the event's native window ID, so the main editor remains usable
when a device is detached. Device input is captured before underlying text
widgets. Monitor HEX and printer setup/properties own their separate focus rings.
Esc closes the innermost popup before its owning device.

Device footers localize categorical values such as status and network mode
after their labels. Paths, endpoints, printer names and user-defined text keep
their original spelling and case.

## Monitor

Unified mode composites graphics and the text bitmap on one canvas. Split mode
shows the two layers independently. The rendering rules and byte protocol are
defined in [Monitor inspection](devices.md#monitor-inspection-window), and the
portable glyph source is in [Assets](assets.md#monitor-bitmap-font).

The header can split/merge, inspect raw bytes, clear the device and save an
image. The HEX popup filters all, graphics or text command bytes without changing
the device. It shows the filtered count, scrolls independently, and has its own
Filter/Close keyboard ring. Closing Monitor also closes the popup.

Image export supports PNG, JPEG, WebP and BMP at logical monitor resolution.
Rendering, encoding and atomic writing run on the UI file worker; only a
successful replacement reports Saved. The chooser is parented to a detached
Monitor when applicable.

The startup layout comes from Settings → External Devices → Monitor layout.
The header split/merge toggle changes only the session; settings preview/cancel
and saved defaults follow [Settings](ui_settings.md#live-preview-and-saving).

## Floppy and HDD

The storage window switches between a CP866-decoded received buffer and the
backing file's preview. Preview reads are bounded and asynchronous. File stamps
avoid rereading unchanged data; external or atomic replacements refresh the
open preview. A generation prevents an old read from replacing a newer attachment.

Floppy can attach/detach an image, save its visible buffer as `.kpd`, `.img` or
`.bin`, clear the buffer and toggle buffer-only debug mode. Saving uses the
shared atomic writer and reports success after completion.

HDD selects a directory containing `hdd.kpd`, creates or deletes that file,
switches its preview and debug mode, and clears visible data. Directory selection
is session-only; Settings → External Devices stores the startup default.
Delete requires confirmation and cannot detach a newer same-path attachment
when an older delete completion arrives.

Debug mode shows a localized debug label in place of the file path. Clearing
inspection buffers does not truncate files. See
[storage workers](devices.md#storage-inspection-windows) for admission and
flush behavior.

## Network

The window shows RX data, the latest TX byte in HEX mode, counters and localized
connection state. Text mode decodes CP866 and displays the retained TX text.
Raw socket errors remain in diagnostic state rather than the visible footer.

The endpoint modal changes client/server mode, address and port for this session.
Applying it cancels the old worker before starting the new connection/listener.
Saved startup endpoints live in Settings → External Devices; launch starts in
client mode.

Clear removes inspection buffers without changing endpoint, connection, error
or worker ownership. Device polling continues while CPU execution is paused.
The worker's last-owner cancellation contract is documented in
[Network worker](devices.md#network-worker).

## Printer buffer

The printer shows uppercase HEX with offsets or CP866-decoded text. Print uses
a captured spool; Clear removes the buffer without changing its target or the
already-submitted job. Busy disables a second Print, while buffer inspection
and clear remain available.

Target precedence is session `PrinterSettings`, persisted global settings, then
the OS default. The header gear edits only the session override. Setup from
Settings edits the global configuration. Printing and clearing do not reset
either target. Cancellation restores Ready without exposing a raw OS error.

Text conversion, native job ownership and typed printer failures are defined in
[Printer spool and printing](devices.md#printer-spool-and-printing) and
[Printer API errors](devices.md#printer-api-errors).

## Printer setup and Properties

Settings select the emulator-styled or system setup dialog. The custom dialog
loads printers and driver configuration asynchronously, reserving its final
layout while pending. It exposes printer metadata, paper, source, orientation
and driver Properties. The preview follows the selected orientation.

Selectors use anchored overlays: they do not move the form or resize its native
window. Short lists fit their contents; long lists scroll and retain access to
their last option. Arrow keys move a highlight without committing; Enter selects,
outside click dismisses, and Esc closes the selector before the dialog.

Properties opens on Favorites and includes General, Paper, Graphics and Advanced
views as available. Options and parameters come from the driver's PrintTicket
capabilities. Changes remain in the setup draft until nested and top-level OK
accept them. Named profiles retain complete driver configurations.

Setup and Properties have independent Tab/Shift+Tab rings rebuilt from enabled
controls. Keyboard traversal reveals an outline; Enter or pointer activation
clears it while retaining ordinary selection styling. Moving to a button blurs
the previous input so typing cannot leak into a hidden field.

Standard paper/bin identifiers and recognized driver labels are localized in
both directions. Unknown labels in another language use localized neutral
identifiers. Printer and profile names remain unchanged.

### Detached printer dialogs

Session setup for a detached Printer uses a dedicated `720×500` native Setup
window over the unchanged `760×340` device. Properties uses its own `1040×680`
window above Setup; opening it does not resize or replace either parent.
It is laid out hidden and revealed at its final size after opening.

While Properties is open, Setup's close controls are disabled. Attempted parent
close refocuses Properties and restarts a restrained attention pulse. Closing
Properties returns focus to Setup; closing Setup or Printer cleans up descendant
windows. Blank header insets drag the corresponding child without intercepting
its buttons.

Setup started from application Settings stays in the main window's overlay stack.
Physical driver behavior and detached native presentation require the
[manual printer checks](manual_testing.md#printer-and-driver-checks).
