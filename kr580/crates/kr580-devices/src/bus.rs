//! `IoBus` implementation that routes ports `00h..04h` to actual devices.
//!
//! This bus is the synchronous boundary the CPU executor calls into. All
//! heavy work happens inside the device workers; the bus methods are O(1)
//! dispatch.

use crate::monitor::MonitorDevice;
use crate::network::NetworkDevice;
use crate::printer::PrinterDevice;
use crate::storage::StorageDevice;
use kr580_core::IoBus;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::Mutex;

/// Aggregate device status for UI display / export.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeviceStatus {
    /// Counter of bytes routed to the monitor port.
    pub monitor_bytes: u64,
    /// Counter of bytes routed to the floppy port.
    pub floppy_bytes: u64,
    /// Counter of bytes routed to the HDD port.
    pub hdd_bytes: u64,
    /// Counter of bytes routed to the network port.
    pub network_bytes: u64,
    /// Counter of bytes routed to the printer port.
    pub printer_bytes: u64,
}

/// Routing IoBus.
///
/// Each peripheral is held behind an `Option<...>` so a partially-configured
/// system (no network, no storage) is still valid. Devices use interior
/// `Arc<Mutex<...>>` where mutability is required — the IoBus implements
/// the [`IoBus`] trait via `&mut self`, so external composition decides
/// whether to wrap us in an `Arc<Mutex<...>>` or hand the bus directly to
/// the core thread.
pub struct DeviceBus {
    /// Monitor (port 0x00).
    pub monitor: MonitorDevice,
    /// Floppy storage (port 0x01).
    pub floppy: Option<StorageDevice>,
    /// HDD storage (port 0x02).
    pub hdd: Option<StorageDevice>,
    /// Network adapter (port 0x03).
    pub network: Option<NetworkDevice>,
    /// Printer (port 0x04).
    pub printer: PrinterDevice,
    /// Counter snapshot (atomic-ish via Mutex; not on the hot path).
    pub status: Arc<Mutex<DeviceStatus>>,
}

impl DeviceBus {
    /// Build a bus with only the monitor + printer attached. Storage and
    /// network can be added later via [`Self::attach_storage`] /
    /// [`Self::attach_network`].
    pub fn new() -> Self {
        Self {
            monitor: MonitorDevice::new(),
            floppy: None,
            hdd: None,
            network: None,
            printer: PrinterDevice::new(),
            status: Arc::new(Mutex::new(DeviceStatus::default())),
        }
    }

    /// Attach a storage device. Returns the previously-attached device, if any.
    pub fn attach_storage(&mut self, dev: StorageDevice) -> Option<StorageDevice> {
        match dev.kind() {
            crate::storage::StorageKind::Floppy => self.floppy.replace(dev),
            crate::storage::StorageKind::Hdd => self.hdd.replace(dev),
        }
    }

    /// Attach a network device.
    pub fn attach_network(&mut self, dev: NetworkDevice) -> Option<NetworkDevice> {
        self.network.replace(dev)
    }
}

impl Default for DeviceBus {
    fn default() -> Self {
        Self::new()
    }
}

impl IoBus for DeviceBus {
    fn read(&mut self, port: u8) -> u8 {
        match port {
            0x00 => self.monitor.read(),
            0x01 => self.floppy.as_ref().map(|d| d.read()).unwrap_or(0xFF),
            0x02 => self.hdd.as_ref().map(|d| d.read()).unwrap_or(0xFF),
            0x03 => self.network.as_ref().map(|d| d.read()).unwrap_or(0xFF),
            0x04 => self.printer.read(),
            _ => 0xFF, // open bus for unmapped ports
        }
    }

    fn write(&mut self, port: u8, value: u8) {
        match port {
            0x00 => {
                self.monitor.write(value);
                self.status.lock().unwrap().monitor_bytes += 1;
            }
            0x01 => {
                if let Some(d) = &self.floppy {
                    let _ = d.write(value);
                    self.status.lock().unwrap().floppy_bytes += 1;
                }
            }
            0x02 => {
                if let Some(d) = &self.hdd {
                    let _ = d.write(value);
                    self.status.lock().unwrap().hdd_bytes += 1;
                }
            }
            0x03 => {
                if let Some(d) = &self.network {
                    let _ = d.write(value);
                    self.status.lock().unwrap().network_bytes += 1;
                }
            }
            0x04 => {
                self.printer.write(value);
                self.status.lock().unwrap().printer_bytes += 1;
            }
            _ => { /* unmapped */ }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kr580_core::IoBus;

    #[test]
    fn writes_to_monitor_increment_counter() {
        let mut bus = DeviceBus::new();
        bus.write(0x00, b'H');
        bus.write(0x00, 0x80); // mode command
        let s = bus.status.lock().unwrap();
        assert_eq!(s.monitor_bytes, 2);
    }

    #[test]
    fn unmapped_port_is_open_bus() {
        let mut bus = DeviceBus::new();
        assert_eq!(bus.read(0x10), 0xFF);
        bus.write(0x10, 0x55); // no-op
    }

    #[test]
    fn printer_buffers_bytes() {
        let mut bus = DeviceBus::new();
        bus.write(0x04, b'A');
        bus.write(0x04, b'B');
        assert_eq!(bus.printer.spool_text(), "AB");
    }
}
