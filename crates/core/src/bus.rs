use crate::PortError;

pub trait PortBus {
    fn input(&mut self, port: u8) -> Result<u8, PortError>;
    fn output(&mut self, port: u8, value: u8) -> Result<(), PortError>;
}

#[derive(Clone, Debug)]
pub struct NullBus {
    ports: [u8; 256],
    writes: Vec<(u8, u8)>,
}

impl Default for NullBus {
    fn default() -> Self {
        Self {
            ports: [0; 256],
            writes: Vec::new(),
        }
    }
}

impl NullBus {
    pub fn set_input(&mut self, port: u8, value: u8) {
        self.ports[port as usize] = value;
    }

    pub fn writes(&self) -> &[(u8, u8)] {
        &self.writes
    }
}

impl PortBus for NullBus {
    fn input(&mut self, port: u8) -> Result<u8, PortError> {
        Ok(self.ports[port as usize])
    }

    fn output(&mut self, port: u8, value: u8) -> Result<(), PortError> {
        self.ports[port as usize] = value;
        self.writes.push((port, value));
        Ok(())
    }
}
