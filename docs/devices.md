# Devices and IoBus

`IoBus` routes the low byte of I/O port addresses:

| Port | Device |
|---|---|
| `00h` | Monitor |
| `01h` | Floppy storage |
| `02h` | HDD storage |
| `03h` | TCP network adapter |
| `04h` | Printer spool |

Invalid ports return `PortError::InvalidPort`; device failures become typed port
errors. Devices do not mutate CPU state behind the core API. Inspection, native
windows and setup workflows are described once in [Device windows](ui_devices.md).

## Network worker

Client mode connects to a remote endpoint; server mode waits for a client.
Reconfiguration explicitly cancels the old worker before starting another.
RX/TX and connection revisions let the actor publish changes while CPU execution
is paused without copying unchanged buffers every poll.

RX storage is bounded. A full receive buffer pauses socket reads rather than
silently dropping bytes; CPU reads or clear notify the worker that room exists.
`queue_received` is fallible and respects the same capacity. Empty `IN 03h`
returns zero without an error. TX admission uses nonblocking FIFO enqueue.

Cloned `NetworkDevice` values share the worker-abort owner. Dropping the final
owner requests cancellation, including a server waiting for its first client.
Resource release follows runtime processing of that cancellation: Drop does not
block, and destroying the runtime is not a confirmed asynchronous close.

Clear removes inspection data without replacing the endpoint, connection/error
state or worker. Changing a session endpoint does not persist startup defaults.

## Monitor command protocol

The monitor consumes these command sequences:

```text
text:     [0 ccccccc] [CP866 byte]
graphics: [1 ccccccc] [x] [y]
```

The first byte's low seven bits are colour intensity. The renderer uses
`0xFFFFFF / 127 * intensity` with low-byte-red packed-colour ordering; this is
not independent linear interpolation of the three RGB channels.

Text writes `(character, colour)` into the next cell of the 64×20 framebuffer
and wraps at its end. Graphics writes one of 256×256 coordinates; rewriting it
replaces its intensity. Layers are independent. `IN 00h` returns status
(`Ready` is zero). The in-flight phase is retained in `MonitorState`.

## Monitor inspection window

Unified mode composites graphics with byte-indexed CP866 bitmap glyphs; split
mode presents the layers separately. Glyphs use 8×12 cells, so the text raster
is wider than the graphics raster. Split text decoding wraps at panel width
without inserting line breaks at framebuffer row boundaries.

The raw stream is bounded diagnostic history; text/pixel buffers remain
authoritative. Protocol filtering and image export are described in
[the monitor UI](ui_devices.md#monitor). Font source and geometry are documented
in [Assets](assets.md#monitor-bitmap-font).

## Storage inspection windows

Floppy and HDD share `StorageDevice`: accepted bytes update inspection buffers
and enqueue append writes to the attached file. Rejected NotReady/Disconnected/
Busy writes do not update accepted counters or visible data.

`attach_file` returns a typed result; open/create failure sets error state and
cannot publish a successful attachment or saved path. Actor requests instead
prepare an owned file on the persistence worker, then use `attach_open_file`.

Reattachment reuses a writer. FIFO file-switch/detach commands flush preceding
accepted bytes before replacing the backing file. Reserved control capacity
keeps these operations separate from byte admission. A full control queue makes
detach fail explicitly.

Detach clears the file path but preserves received inspection data and counters.
Subsequent writes are NotReady unless buffer-only debug is enabled. Attaching
a file disables debug. Debug accepts unsaved inspection bytes without file I/O.

Clear removes visible/tail buffers without changing worker, file, status or
accepted-byte counter. Buffer export writes retained inspection data; it is
distinct from truncating or saving the backing file.

HDD uses `hdd.kpd` in the selected/default directory, creating/appending as
needed. The initial fallback is the home directory. Persistent directory
selection is under Settings → External Devices; window selection is session-only.
The [storage UI](ui_devices.md#floppy-and-hdd) owns previews and create/delete
confirmation.

## Admission and retained data

| Resource | Budget/behavior |
|---|---|
| Storage byte writes and network TX queue | 64 KiB; full admission returns Busy |
| Storage control reserve | 16 slots for switching/detaching files |
| Network RX | 64 KiB, with backpressure |
| Monitor raw history and attached-storage inspection | Latest 64 KiB |
| Storage tail | Latest 4096 bytes |
| Unsaved storage debug data and printer spool | 1 MiB; refuse more bytes until explicit clear |
| Printer export admission | One captured spool at a time |

Socket/file writers drain FIFO batches up to 4096 bytes. Attached storage keeps
writing every accepted byte even when older inspection history is evicted.
Unsaved user buffers refuse overflow rather than silently evicting accepted data.
Snapshots expose retained deques as ordered byte vectors.

## Printer spool and printing

`OUT 04h` accumulates bytes; a separate action exports or prints a captured
spool. The device enters Busy immediately and delivers completion through a
bounded channel, which the actor polls every 50 ms. Clear retains target
settings and does not alter a captured in-flight job.

Native target precedence is session configuration, global configuration, then
the OS default. Windows printing uses the validated complete DEVMODE, including
driver-private bytes. CP866 conversion preserves CR/LF and blank lines, expands
tabs to four spaces, marks unsupported controls with `·` and wraps long lines
at 80 columns. Native cancellation returns Ready without recording a failure.

Printer discovery returns metadata and a typed spooler status for localization.
`DocumentPropertiesW` normalizes driver configuration; `DeviceCapabilitiesW`
supplies paper/source choices. PrintTicket exposes features, constrained options
and typed parameters, and validates edits before converting them back to DEVMODE.
The provider opens, operates and closes on one blocking MTA thread.

### Native safety limits

GDI DC/font/job handles and dialog global memory use ownership guards. Dialog
buffers are checked against GlobalSize, enumeration strings stay within their
returned buffer, and DEVMODE headers/driver-extra lengths are validated before
aligned access. Capability counts above 65,536 and PrintTicket streams above
16 MiB are rejected. Early errors release conversion resources.

## Printer API errors

The five native setup/list/configuration/property functions return `PrinterError`.
`PrinterErrorKind` distinguishes unsupported platform, native API and invalid
data. Display supplies diagnostic context; `Error::source()` retains available
Win32 I/O, HRESULT, XML and encoding causes.

`PrinterDialogError` preserves common-dialog codes separately from Win32 I/O
codes, following [CommDlgExtendedError](https://learn.microsoft.com/en-us/windows/win32/api/commdlg/nf-commdlg-commdlgextendederror).
Cancelled setup returns `Ok(None)`. Partial PrintTicket provider failure may
still produce a usable property sheet with textual `provider_error` feedback.

Rust callers that require a String convert explicitly at a display boundary.
Application blocking-task adapters do that before producing UI messages; the
typed printer API retains its cause chain.

## Confirmed shutdown

`StorageDevice::close()` closes admission without awaiting accepted work.
`shutdown().await` waits for all current/retired writers and final host-file
flushes while the supplied Tokio runtime remains alive. Polling retains writer
failures so completed handles cannot hide an error. The first failure is
reported after all writers have joined.

Backend Shutdown first drains preceding persistence jobs, then joins both
storage devices and accepted printer exports/native tasks, even if one fails.
Only afterward does it finish the matching request. Native task completion is
separate from the physical printer finishing paper output.

Failure produces ErrorRaised and a failed CommandFinished while leaving the
actor available. A subsequent close can acknowledge the reported failure.
Native print cancellation is successful cancellation.

Device/runtime Drop is not a substitute for this protocol. The desktop's
[confirmed exit](ui_app.md#confirmed-application-exit) waits for it. Flush confirms
host writes, not resilience to power failure.
