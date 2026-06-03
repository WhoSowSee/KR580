KR580 Floppy Drive emulates a floppy disk read/write device.

- Accessible via Quick Access panel (floppy icon)
- Window shows accepted drive buffer bytes
- Image button attaches a floppy image file to port 01h
- Save button writes the current visible buffer through separate .kpd, .img, and .bin choices; .kpd is selected by default
- Detach button disconnects the image file without clearing the visible buffer
- Binary view switches the window body to the image file contents
- Debug mode writes bytes directly to the visible buffer without an image file
- Clear resets only the visible window buffer
- Program interacts via dedicated I/O ports
- Not-ready writes are rejected and do not enter the buffer
- Last selected image path is stored in settings
