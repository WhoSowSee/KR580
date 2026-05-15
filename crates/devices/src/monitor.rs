use crate::DeviceStatus;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MonitorMode {
    Text,
    Graphics,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextCell {
    pub ch: u8,
    pub attr: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorState {
    pub mode: MonitorMode,
    pub text: String,
    pub pixels: Vec<(u16, u16, u8)>,
    pub text_cells: Vec<TextCell>,
    pub text_cols: u16,
    pub text_rows: u16,
    pub current_attr: u8,
    pub cursor: u16,
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
        let text_cols = 32;
        let text_rows = 8;
        Self {
            state: MonitorState {
                mode: MonitorMode::Text,
                text: String::new(),
                pixels: Vec::new(),
                text_cells: vec![TextCell::default(); text_cols as usize * text_rows as usize],
                text_cols,
                text_rows,
                current_attr: 0x07,
                cursor: 0,
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
        self.state.last_command = Some(value);
        if value == 0x0E {
            self.state.mode = MonitorMode::Text;
            return;
        }
        if value == 0x0F {
            self.state.mode = MonitorMode::Graphics;
            return;
        }
        if value & 0x80 != 0 {
            match value & 0x7F {
                0x00 => self.state.mode = MonitorMode::Text,
                0x01 => self.state.mode = MonitorMode::Graphics,
                attr @ 0x10..=0x1F => self.state.current_attr = attr & 0x0F,
                _ => {}
            }
            return;
        }
        match self.state.mode {
            MonitorMode::Text => self.write_text_byte(value),
            MonitorMode::Graphics => self.write_graphics_byte(value),
        }
    }

    pub fn input_byte(&self) -> u8 {
        self.state.status.code()
    }

    pub fn state(&self) -> MonitorState {
        self.state.clone()
    }

    fn write_text_byte(&mut self, value: u8) {
        if value.is_ascii_graphic() || value == b' ' || value == b'\n' {
            self.state.text.push(value as char);
        }
        let idx = self.state.cursor as usize;
        if let Some(cell) = self.state.text_cells.get_mut(idx) {
            *cell = TextCell {
                ch: value,
                attr: self.state.current_attr,
            };
        }
        self.advance_cursor();
    }

    fn write_graphics_byte(&mut self, value: u8) {
        let width = self.state.text_cols.max(1);
        let idx = self.state.cursor;
        self.state.pixels.push((idx % width, idx / width, value));
        self.advance_cursor();
    }

    fn advance_cursor(&mut self) {
        let limit = (self.state.text_cols as u32 * self.state.text_rows as u32).max(1) as u16;
        self.state.cursor = self.state.cursor.wrapping_add(1) % limit;
    }
}
