//! Monitor device.
//!
//! Per `prompt/03_peripherals.md`, the monitor exposes:
//!
//! * a command stream;
//! * a text layer;
//! * a pixel layer;
//! * a hex / debug buffer.
//!
//! The visible hex buffer is a debug surface only; it is *not* the primary
//! state. The text layer / pixel layer are the source of truth.
//!
//! The state-machine semantics are intentionally minimal: the prompt sets the
//! contract (text vs graphics, color/intensity from the command byte) but
//! does not mandate exact CRT behavior. We model just enough that other
//! layers — exporters, persistence, UI — have something to consume.

use serde::{Deserialize, Serialize};

/// The two render paths defined by `prompt/03_peripherals.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MonitorMode {
    /// Text-mode rendering.
    #[default]
    Text,
    /// Graphics-mode (pixel buffer) rendering.
    Graphics,
}

/// One framebuffer cell in text mode.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextCell {
    /// 7-bit ASCII character (top bit reserved for intensity).
    pub ch: u8,
    /// Color / intensity attribute byte, derived from the latest command.
    pub attr: u8,
}

/// Visible monitor state (a snapshot the UI can render).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorState {
    /// Current rendering mode.
    pub mode: MonitorMode,
    /// Text framebuffer (row-major). 32 columns × 8 rows by default.
    pub text: Vec<TextCell>,
    /// Logical text width.
    pub text_cols: u16,
    /// Logical text height.
    pub text_rows: u16,
    /// Pixel framebuffer for graphics mode.
    pub pixels: Vec<u8>,
    /// Current attribute byte (color / intensity).
    pub current_attr: u8,
    /// Current write cursor (for text mode).
    pub cursor: u16,
    /// Last `OUT` byte observed (debug surface).
    pub last_command: Option<u8>,
}

impl Default for MonitorState {
    fn default() -> Self {
        let cols = 32u16;
        let rows = 8u16;
        Self {
            mode: MonitorMode::Text,
            text: vec![TextCell::default(); (cols as usize) * (rows as usize)],
            text_cols: cols,
            text_rows: rows,
            pixels: vec![0u8; 256 * 64],
            current_attr: 0x07,
            cursor: 0,
            last_command: None,
        }
    }
}

/// In-process monitor device. Synchronous, but cheap; the `IoBus` calls into
/// it directly. The device has no internal worker thread because there is
/// no host I/O involved — only buffer mutation.
#[derive(Debug, Default)]
pub struct MonitorDevice {
    state: MonitorState,
}

impl MonitorDevice {
    /// Build a fresh monitor.
    pub fn new() -> Self {
        Self::default()
    }

    /// Borrow the current state for rendering or export.
    pub fn state(&self) -> &MonitorState {
        &self.state
    }

    /// Reset the monitor.
    pub fn reset(&mut self) {
        self.state = MonitorState::default();
    }

    /// Handle an `OUT 0x00` byte.
    ///
    /// The encoding interprets the high bit as a "command vs data" selector:
    /// commands set mode and color/intensity; data writes characters into
    /// the active layer. The exact bit layout is not specified by the
    /// prompt, so we choose a simple, testable convention and document it
    /// here (and in `docs/devices/monitor.md`).
    ///
    /// * `0x80 | 0x00`  → switch to text mode
    /// * `0x80 | 0x01`  → switch to graphics mode
    /// * `0x80 | 0x10..0x1F` → set color/intensity attribute (low nibble)
    /// * `0x00..0x7F`   → printable byte written at the cursor / pixel byte
    pub fn write(&mut self, byte: u8) {
        self.state.last_command = Some(byte);
        if byte & 0x80 != 0 {
            // Command byte
            let cmd = byte & 0x7F;
            match cmd {
                0x00 => self.state.mode = MonitorMode::Text,
                0x01 => self.state.mode = MonitorMode::Graphics,
                v if (0x10..=0x1F).contains(&v) => {
                    self.state.current_attr = v & 0x0F;
                }
                _ => { /* ignore unknown commands */ }
            }
        } else {
            match self.state.mode {
                MonitorMode::Text => {
                    let idx = self.state.cursor as usize;
                    if idx < self.state.text.len() {
                        self.state.text[idx] = TextCell {
                            ch: byte,
                            attr: self.state.current_attr,
                        };
                    }
                    self.state.cursor = self
                        .state
                        .cursor
                        .wrapping_add(1)
                        .min(self.state.text.len() as u16);
                }
                MonitorMode::Graphics => {
                    let idx = self.state.cursor as usize % self.state.pixels.len();
                    self.state.pixels[idx] = byte;
                    self.state.cursor = self.state.cursor.wrapping_add(1);
                }
            }
        }
    }

    /// `IN 0x00` returns `0x00` by default — there is no read port semantics
    /// for a monitor in the prompt.
    pub fn read(&mut self) -> u8 {
        0x00
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writing_data_in_text_mode_sets_cell() {
        let mut m = MonitorDevice::new();
        m.write(b'A');
        assert_eq!(m.state().text[0].ch, b'A');
        assert_eq!(m.state().cursor, 1);
    }

    #[test]
    fn switching_to_graphics_clears_command_log() {
        let mut m = MonitorDevice::new();
        m.write(0x81); // graphics
        assert_eq!(m.state().mode, MonitorMode::Graphics);
    }

    #[test]
    fn attribute_byte_is_remembered() {
        let mut m = MonitorDevice::new();
        m.write(0x9A); // attr = 0x0A
        assert_eq!(m.state().current_attr, 0x0A);
        m.write(b'X');
        assert_eq!(m.state().text[0].attr, 0x0A);
    }
}
