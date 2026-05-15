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

- Monitor keeps a text string, a text-cell framebuffer with attributes, a graphics layer, cursor, current attribute, last command, and a hex/debug buffer.
- Storage devices append to visible buffers, maintain a bounded tail buffer, count queued bytes, expose last enqueue error, and can attach async file-backed workers.
- Network exposes explicit mode, connection state, RX buffer, and TX buffer. No-data reads are non-fatal.
- Printer accumulates bytes in a spool first, tracks buffered byte count and last enqueue error, and exports/prints through a separate queued action.

## Port behavior

Invalid ports return `PortError::InvalidPort`. Device-specific enqueue failures are converted into typed `PortError` variants such as `NotReady` and `Disconnected`, so the application can surface failures through events instead of panics or ad hoc strings.

## Monitor command convention

The monitor accepts the documented legacy mode bytes `0Eh` (text) and `0Fh` (graphics). It also supports a high-bit command convention used by the richer framebuffer model:

- `80h`: text mode;
- `81h`: graphics mode;
- `90h..9Fh`: set low-nibble color/intensity attribute.

Bytes below `80h` are data bytes and are routed to the active render layer.
