# Monitor device

The prompt sets the contract for the monitor (text vs graphics layer,
attribute byte from a command stream, hex/debug surface as a *secondary*
view) but does not mandate a specific CRT layout. The implementation
documents and tests its own decoded byte layout.

## Byte interpretation

`OUT 0x00` writes a single byte. The high bit selects command vs data:

| Byte form          | Meaning                                                   |
| ------------------ | --------------------------------------------------------- |
| `0b1000_0000`      | switch to **text mode**                                   |
| `0b1000_0001`      | switch to **graphics mode**                               |
| `0b1001_xxxx`      | set color/intensity attribute = `xxxx`                    |
| `0b0xxx_xxxx`      | data byte: written to text framebuffer or pixel buffer    |
| other `0b1xxx_xxxx`| ignored (forward-compatible)                              |

The cursor advances on each data byte. In text mode the framebuffer is
32 columns × 8 rows by default. The pixel buffer is 256 × 64 bytes by
default. These dimensions are arbitrary choices that satisfy the prompt's
requirement to expose both layers.

## `IN 0x00`

Returns `0x00`. The prompt does not define a read protocol for the monitor.

## State snapshot

```rust
pub struct MonitorState {
    pub mode: MonitorMode,
    pub text: Vec<TextCell>,
    pub text_cols: u16,
    pub text_rows: u16,
    pub pixels: Vec<u8>,
    pub current_attr: u8,
    pub cursor: u16,
    pub last_command: Option<u8>,
}
```

`last_command` is the debug surface; the UI uses it to display the most
recently observed byte without scraping the framebuffer.
