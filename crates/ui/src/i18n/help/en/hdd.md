The HDD uses port 02h. OUT appends a byte to hdd.kpd, while IN returns the device status code.

**Storage file**
Choosing a directory creates or opens hdd.kpd inside it. Settings provide the default directory and verify write access. The window can create or delete the file with confirmation and switch between the received buffer and file contents.

Without an open file, writes return Not Ready. Debug mode writes only to the visible buffer. Clearing the buffer does not remove data from hdd.kpd. The window can be detached, pinned, and attached again.
