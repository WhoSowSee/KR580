//! Verify that `OUT` instructions executed by the core are routed through
//! the device bus and observed by the appropriate device.

use kr580_core::{Cpu8080State, IoBus};
use kr580_devices::DeviceBus;

#[test]
fn out_to_monitor_port_reaches_monitor_device() {
    let mut cpu = Cpu8080State::new();
    let mut bus = DeviceBus::new();
    // MVI A,0x41 ; OUT 0x00 ; HLT
    cpu.ram.write(0, 0x3E);
    cpu.ram.write(1, b'A');
    cpu.ram.write(2, 0xD3);
    cpu.ram.write(3, 0x00);
    cpu.ram.write(4, 0x76);
    cpu.run_until_halt(&mut bus, 16).unwrap();
    assert_eq!(bus.monitor.state().text[0].ch, b'A');
}

#[test]
fn out_to_printer_port_reaches_printer_device() {
    let mut cpu = Cpu8080State::new();
    let mut bus = DeviceBus::new();
    // MVI A,'X' ; OUT 0x04 ; HLT
    cpu.ram.write(0, 0x3E);
    cpu.ram.write(1, b'X');
    cpu.ram.write(2, 0xD3);
    cpu.ram.write(3, 0x04);
    cpu.ram.write(4, 0x76);
    cpu.run_until_halt(&mut bus, 16).unwrap();
    assert_eq!(bus.printer.spool_text(), "X");
}

#[test]
fn unmapped_port_in_returns_open_bus() {
    let mut bus = DeviceBus::new();
    assert_eq!(bus.read(0x55), 0xFF);
    bus.write(0x55, 0xAA); // no-op
}
