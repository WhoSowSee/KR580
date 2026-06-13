KR580 Network Adapter exchanges bytes over TCP/IP through device port `03h`.

- The receive buffer is shown as hexadecimal rows of 16 bytes, while the right panel shows only the last transmitted value
- The footer shows mode, address, TCP port, connection state, and byte counters
- The globe button opens a compact mode, address, and port dialog
- Client mode connects to the configured endpoint; server mode listens on the configured local endpoint
- Applying settings safely stops the previous connection and starts the new one for the current session only
- Startup client and server endpoints are saved under Settings → General; launch always begins in client mode
- Raw socket errors are hidden; the footer shows a short status such as `Refused`, `Timed out`, or `Error`
- The clear button removes only RX/TX contents without changing settings, status, or connection state; empty buffers are a no-op
- The window can be detached, dragged by its top band, and pinned above other windows

`OUT 03h` replaces the last transmitted value and sends the byte to the connected peer. `IN 03h` returns the next RX byte or `00h` when no data is available.
