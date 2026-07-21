The floppy uses port 01h. OUT writes a byte to an attached image, while IN returns the device status code.

**Image workflow**
• Open image attaches a file; Settings can provide a default path
• Detach closes the image without clearing the visible buffer
• The view switches between received bytes and file contents
• The buffer can be saved as .kpd, .img, or .bin

Without an attached image, writes return Not Ready. Debug mode accepts bytes only into the visible buffer. Clearing the buffer does not alter the file. The window can be detached, pinned, and attached again.
