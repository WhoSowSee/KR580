KR580 Hard Disk emulates a large-capacity permanent storage device.

**Interface**
- Accessible via Quick Access panel (HDD icon)
- Window shows accepted drive buffer bytes
- Data exchange via I/O port 02h

**Window controls**
- Choose directory — picks a folder; creates/opens `hdd.kpd` inside it (session-only)
- Show file contents — toggles between write buffer and on-disk file contents
- Debug mode — writes bytes directly to the visible buffer without a file
- Clear buffer — clears only the visible window buffer
- Delete file — removes `hdd.kpd` from disk (with confirmation dialog)
- Create file — creates `hdd.kpd` in the current or default directory

**Settings**
The default directory for the HDD file is configured in Settings → General → HDD directory.
If not set, the user's home directory is used. The directory picker includes a write-permission
check — an error notice is shown if the chosen directory is not writable.

Program interaction is via the `OUT 02h` and `IN 02h` instructions.
If the device is not ready, the byte is rejected and does not enter the buffer.
