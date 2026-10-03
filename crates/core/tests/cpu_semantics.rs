pub(crate) use k580_core::{
    CoreError, Cpu8080State, DecodeError, Flags, NullBus, RegisterName, decode_opcode,
};

pub(crate) fn step(cpu: &mut Cpu8080State) {
    let mut bus = NullBus::default();
    cpu.step_instruction(&mut bus).unwrap();
}

pub(crate) fn put_program(cpu: &mut Cpu8080State, bytes: &[u8]) {
    for (offset, byte) in bytes.iter().copied().enumerate() {
        cpu.memory.write(offset as u16, byte);
    }
}

#[path = "cpu_semantics/classification.rs"]
mod classification;
#[path = "cpu_semantics/control_flow.rs"]
mod control_flow;
#[path = "cpu_semantics/flags.rs"]
mod flags;
#[path = "cpu_semantics/io_interrupts.rs"]
mod io_interrupts;
#[path = "cpu_semantics/memory.rs"]
mod memory;
#[path = "cpu_semantics/timing.rs"]
mod timing;
