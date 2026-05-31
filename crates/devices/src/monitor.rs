use crate::DeviceStatus;
use serde::{Deserialize, Serialize};

pub const TEXT_COLS: u16 = 39;
pub const TEXT_ROWS: u16 = 20;
pub const TEXT_CELL_COUNT: usize = (TEXT_COLS as usize) * (TEXT_ROWS as usize);
pub const GRAPHICS_WIDTH: u16 = 256;
pub const GRAPHICS_HEIGHT: u16 = 256;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextCell {
    pub ch: u8,
    pub color: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MonitorPhase {
    #[default]
    Idle,
    AwaitingTextChar {
        color: u8,
    },
    AwaitingGraphicsX {
        color: u8,
    },
    AwaitingGraphicsY {
        color: u8,
        x: u8,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorState {
    pub text_cells: Vec<TextCell>,
    pub text_cursor: u16,
    pub pixels: Vec<(u8, u8, u8)>,
    pub phase: MonitorPhase,
    pub last_command: Option<u8>,
    pub hex_buffer: Vec<u8>,
    pub status: DeviceStatus,
}

#[derive(Clone, Debug)]
pub struct MonitorDevice {
    state: MonitorState,
}

impl Default for MonitorDevice {
    fn default() -> Self {
        Self {
            state: MonitorState {
                text_cells: vec![TextCell::default(); TEXT_CELL_COUNT],
                text_cursor: 0,
                pixels: Vec::new(),
                phase: MonitorPhase::Idle,
                last_command: None,
                hex_buffer: Vec::new(),
                status: DeviceStatus::Ready,
            },
        }
    }
}

impl MonitorDevice {
    pub fn output_byte(&mut self, value: u8) {
        self.state.hex_buffer.push(value);
        match self.state.phase {
            MonitorPhase::Idle => {
                self.state.last_command = Some(value);
                let color = value & 0x7F;
                self.state.phase = if value & 0x80 == 0 {
                    MonitorPhase::AwaitingTextChar { color }
                } else {
                    MonitorPhase::AwaitingGraphicsX { color }
                };
            }
            MonitorPhase::AwaitingTextChar { color } => {
                self.write_text_char(color, value);
                self.state.phase = MonitorPhase::Idle;
            }
            MonitorPhase::AwaitingGraphicsX { color } => {
                self.state.phase = MonitorPhase::AwaitingGraphicsY { color, x: value };
            }
            MonitorPhase::AwaitingGraphicsY { color, x } => {
                self.write_pixel(color, x, value);
                self.state.phase = MonitorPhase::Idle;
            }
        }
    }

    pub fn input_byte(&self) -> u8 {
        self.state.status.code()
    }

    pub fn state(&self) -> MonitorState {
        self.state.clone()
    }

    pub fn clear(&mut self) {
        self.state.text_cells = vec![TextCell::default(); TEXT_CELL_COUNT];
        self.state.text_cursor = 0;
        self.state.pixels.clear();
        self.state.phase = MonitorPhase::Idle;
        self.state.last_command = None;
        self.state.hex_buffer.clear();
    }

    fn write_text_char(&mut self, color: u8, ch: u8) {
        let idx = self.state.text_cursor as usize;
        if let Some(cell) = self.state.text_cells.get_mut(idx) {
            *cell = TextCell { ch, color };
        }
        let limit = TEXT_CELL_COUNT.max(1) as u16;
        self.state.text_cursor = self.state.text_cursor.wrapping_add(1) % limit;
    }

    fn write_pixel(&mut self, color: u8, x: u8, y: u8) {
        if let Some(slot) = self
            .state
            .pixels
            .iter_mut()
            .find(|(px, py, _)| *px == x && *py == y)
        {
            slot.2 = color;
        } else {
            self.state.pixels.push((x, y, color));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graphics_command_writes_pixel_at_coordinates() {
        let mut dev = MonitorDevice::default();
        // 3-byte cmd: bit7=1 + colour 0x7F, X=10, Y=20.
        dev.output_byte(0xFF);
        dev.output_byte(10);
        dev.output_byte(20);
        let s = dev.state();
        assert_eq!(s.pixels, vec![(10, 20, 0x7F)]);
        assert_eq!(s.phase, MonitorPhase::Idle);
        assert!(s.text_cells.iter().all(|c| c.ch == 0));
    }

    #[test]
    fn text_command_writes_character_with_colour() {
        let mut dev = MonitorDevice::default();
        // 2-byte cmd: bit7=0 + colour 0x40, char='A' (0x41).
        dev.output_byte(0x40);
        dev.output_byte(0x41);
        let s = dev.state();
        assert_eq!(
            s.text_cells[0],
            TextCell {
                ch: 0x41,
                color: 0x40
            }
        );
        assert_eq!(s.text_cursor, 1);
        assert!(s.pixels.is_empty());
        assert_eq!(s.phase, MonitorPhase::Idle);
    }

    #[test]
    fn graphics_overwrite_replaces_intensity_not_appends() {
        let mut dev = MonitorDevice::default();
        for _ in 0..2 {
            dev.output_byte(0x80);
            dev.output_byte(5);
            dev.output_byte(7);
        }
        dev.output_byte(0xFF);
        dev.output_byte(5);
        dev.output_byte(7);
        let s = dev.state();
        assert_eq!(s.pixels, vec![(5, 7, 0x7F)]);
    }

    #[test]
    fn text_cursor_wraps_at_end_of_screen() {
        let mut dev = MonitorDevice::default();
        for i in 0..(TEXT_CELL_COUNT + 1) {
            dev.output_byte(0x00);
            dev.output_byte((i & 0xFF) as u8);
        }
        let s = dev.state();
        assert_eq!(s.text_cursor, 1);
    }

    #[test]
    fn hex_buffer_records_every_outgoing_byte() {
        let mut dev = MonitorDevice::default();
        dev.output_byte(0x80);
        dev.output_byte(1);
        dev.output_byte(2);
        dev.output_byte(0x00);
        dev.output_byte(0x41);
        assert_eq!(dev.state().hex_buffer, vec![0x80, 1, 2, 0x00, 0x41]);
    }
}
