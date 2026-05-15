use crate::{
    MonitorDevice, MonitorState, NetworkDevice, NetworkState, PrinterDevice, PrinterState,
    StorageDevice, StorageState,
};
use k580_core::{PortBus, PortError};

#[derive(Debug)]
pub struct IoBus {
    pub monitor: MonitorDevice,
    pub floppy: StorageDevice,
    pub hdd: StorageDevice,
    pub network: NetworkDevice,
    pub printer: PrinterDevice,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceSnapshot {
    pub monitor: MonitorState,
    pub floppy: StorageState,
    pub hdd: StorageState,
    pub network: NetworkState,
    pub printer: PrinterState,
}

impl Default for IoBus {
    fn default() -> Self {
        Self {
            monitor: MonitorDevice::default(),
            floppy: StorageDevice::new("floppy"),
            hdd: StorageDevice::new("hdd"),
            network: NetworkDevice::default(),
            printer: PrinterDevice::default(),
        }
    }
}

impl IoBus {
    pub const MONITOR_PORT: u8 = 0x00;
    pub const FLOPPY_PORT: u8 = 0x01;
    pub const HDD_PORT: u8 = 0x02;
    pub const NETWORK_PORT: u8 = 0x03;
    pub const PRINTER_PORT: u8 = 0x04;

    pub fn snapshot(&self) -> DeviceSnapshot {
        DeviceSnapshot {
            monitor: self.monitor.state(),
            floppy: self.floppy.state(),
            hdd: self.hdd.state(),
            network: self.network.state(),
            printer: self.printer.state(),
        }
    }
}

impl PortBus for IoBus {
    fn input(&mut self, port: u8) -> Result<u8, PortError> {
        match port {
            Self::MONITOR_PORT => Ok(self.monitor.input_byte()),
            Self::FLOPPY_PORT => Ok(self.floppy.input_byte()),
            Self::HDD_PORT => Ok(self.hdd.input_byte()),
            Self::NETWORK_PORT => Ok(self.network.input_byte()),
            Self::PRINTER_PORT => Ok(self.printer.input_byte()),
            invalid => Err(PortError::InvalidPort(invalid)),
        }
    }

    fn output(&mut self, port: u8, value: u8) -> Result<(), PortError> {
        match port {
            Self::MONITOR_PORT => {
                self.monitor.output_byte(value);
                Ok(())
            }
            Self::FLOPPY_PORT => self.floppy.write_byte(value).map_err(Into::into),
            Self::HDD_PORT => self.hdd.write_byte(value).map_err(Into::into),
            Self::NETWORK_PORT => self.network.output_byte(value).map_err(Into::into),
            Self::PRINTER_PORT => {
                self.printer.output_byte(value);
                Ok(())
            }
            invalid => Err(PortError::InvalidPort(invalid)),
        }
    }
}
