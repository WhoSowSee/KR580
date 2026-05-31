# Devices and IoBus

`IoBus` routes the low byte of I/O port addresses:

| Port | Device |
|---:|---|
| `00h` | Monitor |
| `01h` | Floppy storage |
| `02h` | HDD storage |
| `03h` | Network adapter |
| `04h` | Printer |

Device operations return typed status or errors. They do not mutate CPU state behind the core API.

- Monitor keeps a 39×20 text-cell framebuffer (`ch` + 7-bit `color`), a sparse 256×256 graphics layer (`Vec<(x, y, intensity)>`), a phase tracker for the in-flight 2/3-byte command, last command byte, and the raw byte stream (`hex_buffer`).
- Storage devices append to visible buffers, maintain a bounded tail buffer, count queued bytes, expose last enqueue error, and can attach async file-backed workers.
- Network exposes explicit mode, connection state, RX buffer, TX buffer, byte counters, last error, and an optional Tokio-backed TCP worker. No-data reads are non-fatal.
- Printer accumulates bytes in a spool first, tracks buffered byte count and last enqueue error, and exports/prints through a separate queued action.

## Port behavior

Invalid ports return `PortError::InvalidPort`. Device-specific enqueue failures are converted into typed `PortError` variants such as `NotReady` and `Disconnected`, so the application can surface failures through events instead of panics or ad hoc strings.

## Network worker

`NetworkDevice::start_worker` spawns a Tokio task for client or server mode. The worker connects or binds explicitly from settings, splits the socket into read/write halves, queues received bytes into the device RX queue, drains outgoing bytes from a channel, and updates visible status/counters. The old manual `queue_received` test hook remains available for deterministic unit tests.

## Monitor command protocol

The monitor on port `00h` consumes 2- or 3-byte commands. The first byte's bit 7 selects the destination layer; bits 0..6 carry a 7-bit colour intensity (`rgb = 0xFFFFFF / 127 * intensity`). This matches the original KP580 emulator's documented protocol (`KP580_Help.chm` → `Prog_Wrk_Peref.htm`).

```
text command (2 bytes):     [0 ccccccc] [char_oem]
graphics command (3 bytes): [1 ccccccc] [x] [y]
```

- Text commands write `(ch, color)` into the next text cell (cursor wraps at `39 * 20`).
- Graphics commands write `(x, y, intensity)` into the pixel layer; rewriting the same coordinate replaces the previous intensity rather than appending.
- The two layers coexist independently — a graphics command never touches the text buffer and vice versa.
- `IN 00h` returns the device status byte (`Ready` → 0).

`MonitorPhase` (`Idle | AwaitingTextChar | AwaitingGraphicsX | AwaitingGraphicsY`) is part of `MonitorState` so the inspection window can show whether the device is mid-command.

## Monitor inspection window

`MonitorState`, `MonitorPhase`, `TextCell` and the geometry constants (`TEXT_COLS`, `TEXT_ROWS`, `GRAPHICS_WIDTH`, `GRAPHICS_HEIGHT`) are re-exported from `k580-app`. The Монитор chip in the bottom «Быстрый доступ» strip dispatches `Message::OpenMonitor`; the resulting modal renders pure views over the live `AppSnapshot.devices.monitor`.

The window has two visual modes, toggled from the header button:

- **unified** (default) — one 256×256 canvas with the pixel layer and the rasterised text layer composited on top, mirroring the original KP580 emulator's single-display behaviour. The text glyphs come from a bundled 5×7 ASCII font (`view::monitor_font`).
- **split** — separate graphics and text blocks, each 1:1 with its source buffer. Useful when debugging a program that mixes layers and you need to see exactly which command wrote what.

Both modes share the meta strip (phase, text cursor, pixel count, last command) and the raw byte stream (`hex_buffer`). The window never writes back to the device — it is strictly a debug surface, matching `prompt/03_peripherals.md`'s rule that the hex buffer is a debug surface, not the primary state. See `docs/ui_app.md` for the rendering details.
