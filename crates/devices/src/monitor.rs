use crate::DeviceStatus;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MonitorMode {
    Text,
    Graphics,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorState {
    pub mode: MonitorMode,
    pub text: String,
    pub pixels: Vec<(u16, u16, u8)>,
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
                mode: MonitorMode::Text,
                text: String::new(),
                pixels: Vec::new(),
                hex_buffer: Vec::new(),
                status: DeviceStatus::Ready,
            },
        }
    }
}

impl MonitorDevice {
    pub fn output_byte(&mut self, value: u8) {
        self.state.hex_buffer.push(value);
        match value {
            0x0E => self.state.mode = MonitorMode::Text,
            0x0F => self.state.mode = MonitorMode::Graphics,
            byte if byte.is_ascii_graphic() || byte == b' ' || byte == b'\n' => {
                self.state.text.push(byte as char)
            }
            _ => {}
        }
    }

    pub fn input_byte(&self) -> u8 {
        self.state.status.code()
    }

    pub fn state(&self) -> MonitorState {
        self.state.clone()
    }
}
