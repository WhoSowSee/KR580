The network adapter uses TCP and device port 03h.

**Modes**
Client connects to a remote host and TCP port. Server listens on a local address and port. Settings define initial client/server endpoints; changes made in the device window apply to the current session.

OUT 03h transmits a byte and shows it as the latest TX value. IN 03h pops the next RX byte or returns 00h when no data is available. RX/TX totals and connection state appear at the bottom.

Clear removes only RX/TX buffers. Changing mode stops the old connection safely. The window can be detached, pinned, and attached again.
