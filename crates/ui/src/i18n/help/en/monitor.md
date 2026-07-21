The monitor uses port 00h and accepts variable-length commands.

**Text command – 2 bytes**
The first byte has bit 7=0 and intensity 0–127. The second byte is a CP866 character. It is written to the next position of the 64×20 text layer; the position wraps after the last cell.

**Graphics command – 3 bytes**
The first byte has bit 7=1 and intensity 0–127. The next bytes are X and Y in a 256×256 plane. Writing the same coordinate replaces the pixel intensity.

Text and graphics layers are independent. The window can combine or split them, filter the raw stream, clear the screen, and save PNG. It can be detached, moved, pinned above other windows, and attached again.
