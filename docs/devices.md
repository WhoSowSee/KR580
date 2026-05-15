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

- Monitor keeps text, graphics metadata, and a hex/debug buffer.
- Storage devices append to visible buffers and can attach async file-backed workers.
- Network exposes explicit mode, connection state, RX buffer, and TX buffer. No-data reads are non-fatal.
- Printer accumulates bytes in a spool first; exporting/printing is a separate queued action.
