//! Runtime wiring between the UI thread and the deterministic core actor.
//!
//! Architecture (per `prompt/01_architecture.md`):
//!
//! * The core runs on a *single dedicated worker thread*.
//! * The UI sends commands through a [`crossbeam_channel`] queue.
//! * Device workers and the network device run inside a Tokio runtime, but
//!   they never mutate core state directly: every observation goes through
//!   the bus or status snapshots.
//! * The UI receives `StateView` snapshots — never mutable references.

use crossbeam_channel::{unbounded, Receiver, Sender};
use kr580_core::{Cpu8080State, Flags, IoBus, Reg8};
use kr580_devices::DeviceBus;
use kr580_persistence::Snapshot580Serializer;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Commands sent from the UI to the core actor.
#[derive(Debug, Clone)]
pub enum UiCommand {
    /// Reset the CPU.
    ResetCpu,
    /// Reset the registers only.
    ResetRegisters,
    /// Wipe RAM.
    ResetRam,
    /// Step a single instruction.
    StepInstruction,
    /// Run continuously until halt or stop.
    Run,
    /// Stop a running core (cooperatively).
    Stop,
    /// Save a snapshot to bytes.
    SaveSnapshot,
    /// Load a snapshot from bytes.
    LoadSnapshot(Vec<u8>),
    /// Set a register.
    SetRegister(Reg8, u8),
    /// Write a single byte of RAM.
    SetMemory(u16, u8),
    /// Write a port value through the device bus.
    WritePort(u8, u8),
}

/// Events published by the core actor for the UI.
#[derive(Debug, Clone)]
pub enum UiEvent {
    /// Latest authoritative state snapshot (small subset, shared via `Arc`).
    State(Arc<StateView>),
    /// Instruction executed.
    InstructionExecuted {
        /// Program counter before fetch.
        pc: u16,
        /// T-states consumed.
        t_states: u32,
    },
    /// Snapshot saved.
    SnapshotSaved(Vec<u8>),
    /// Snapshot loaded.
    SnapshotLoaded,
    /// Halt state changed.
    HaltChanged(bool),
    /// Error from the core.
    Error(String),
}

/// Read-only view of core state used by the UI.
#[derive(Debug, Clone)]
pub struct StateView {
    /// 8-bit registers in canonical order.
    pub registers: [(Reg8, u8); 7],
    /// Flags.
    pub flags: Flags,
    /// Program counter.
    pub pc: u16,
    /// Stack pointer.
    pub sp: u16,
    /// Cycle count since reset.
    pub cycle_count: u64,
    /// Whether the CPU is halted.
    pub halted: bool,
    /// Compact RAM preview (first 256 bytes, used for instruction disassembly).
    pub ram_preview: Vec<u8>,
}

impl StateView {
    /// Build a view from a core state.
    pub fn from_core(state: &Cpu8080State) -> Self {
        Self {
            registers: [
                (Reg8::A, state.a),
                (Reg8::B, state.b),
                (Reg8::C, state.c),
                (Reg8::D, state.d),
                (Reg8::E, state.e),
                (Reg8::H, state.h),
                (Reg8::L, state.l),
            ],
            flags: state.flags,
            pc: state.pc,
            sp: state.sp,
            cycle_count: state.cycle_count,
            halted: state.halted,
            ram_preview: state.ram.as_slice()[..256].to_vec(),
        }
    }
}

/// Handles the UI keeps to talk to the core actor.
#[derive(Clone)]
pub struct RuntimeHandles {
    /// Send commands to the core.
    pub commands: Sender<UiCommand>,
    /// Receive events from the core.
    pub events: Receiver<UiEvent>,
}

/// Boot the core actor on a dedicated thread and return the channel handles.
///
/// `bus` is shared between the core actor and the device workers via Mutex;
/// the core thread is the only one that calls into the bus on the hot path.
pub fn run(bus: Arc<Mutex<DeviceBus>>) -> RuntimeHandles {
    let (cmd_tx, cmd_rx): (Sender<UiCommand>, Receiver<UiCommand>) = unbounded();
    let (evt_tx, evt_rx): (Sender<UiEvent>, Receiver<UiEvent>) = unbounded();

    thread::Builder::new()
        .name("kr580-core".to_string())
        .spawn(move || core_actor_loop(bus, cmd_rx, evt_tx))
        .expect("spawn core actor");

    RuntimeHandles {
        commands: cmd_tx,
        events: evt_rx,
    }
}

fn core_actor_loop(
    bus: Arc<Mutex<DeviceBus>>,
    cmd_rx: Receiver<UiCommand>,
    evt_tx: Sender<UiEvent>,
) {
    let mut cpu = Cpu8080State::new();
    let mut running = false;

    loop {
        // Drain commands. If we are not running, block until one arrives.
        let cmd = if running {
            cmd_rx.try_recv().ok()
        } else {
            match cmd_rx.recv() {
                Ok(c) => Some(c),
                Err(_) => return, // sender dropped → exit
            }
        };

        if let Some(cmd) = cmd {
            match cmd {
                UiCommand::ResetCpu => cpu.reset_cpu(),
                UiCommand::ResetRegisters => cpu.reset_registers(),
                UiCommand::ResetRam => cpu.ram.clear(),
                UiCommand::StepInstruction => {
                    let pc_before = cpu.pc;
                    let mut bus_guard = bus.lock().unwrap();
                    match cpu.step_instruction(&mut *bus_guard) {
                        Ok(t) => {
                            let _ = evt_tx.send(UiEvent::InstructionExecuted {
                                pc: pc_before,
                                t_states: t,
                            });
                        }
                        Err(e) => {
                            let _ = evt_tx.send(UiEvent::Error(e.to_string()));
                        }
                    }
                }
                UiCommand::Run => running = true,
                UiCommand::Stop => running = false,
                UiCommand::SaveSnapshot => {
                    let bytes = Snapshot580Serializer::save(&cpu);
                    let _ = evt_tx.send(UiEvent::SnapshotSaved(bytes));
                }
                UiCommand::LoadSnapshot(bytes) => match Snapshot580Serializer::load(&bytes) {
                    Ok(state) => {
                        cpu = state;
                        let _ = evt_tx.send(UiEvent::SnapshotLoaded);
                    }
                    Err(e) => {
                        let _ = evt_tx.send(UiEvent::Error(e.to_string()));
                    }
                },
                UiCommand::SetRegister(r, v) => cpu.set_reg8(r, v),
                UiCommand::SetMemory(addr, v) => cpu.ram.write(addr, v),
                UiCommand::WritePort(port, v) => {
                    let mut bus_guard = bus.lock().unwrap();
                    bus_guard.write(port, v);
                }
            }
            let _ = evt_tx.send(UiEvent::State(Arc::new(StateView::from_core(&cpu))));
        }

        if running && !cpu.halted {
            let mut bus_guard = bus.lock().unwrap();
            // Run a small batch so we stay responsive to commands.
            for _ in 0..1024 {
                if cpu.halted {
                    break;
                }
                let pc_before = cpu.pc;
                match cpu.step_instruction(&mut *bus_guard) {
                    Ok(t) => {
                        if t == 0 {
                            break;
                        }
                        let _ = evt_tx.send(UiEvent::InstructionExecuted {
                            pc: pc_before,
                            t_states: t,
                        });
                    }
                    Err(e) => {
                        running = false;
                        let _ = evt_tx.send(UiEvent::Error(e.to_string()));
                        break;
                    }
                }
            }
            drop(bus_guard);
            let _ = evt_tx.send(UiEvent::State(Arc::new(StateView::from_core(&cpu))));
            if cpu.halted {
                running = false;
                let _ = evt_tx.send(UiEvent::HaltChanged(true));
            }
        }

        if !running {
            // Avoid busy spin while idle.
            thread::sleep(Duration::from_millis(1));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_instruction_publishes_state() {
        let bus = Arc::new(Mutex::new(DeviceBus::new()));
        let h = run(bus.clone());
        // Write a NOP, then step.
        h.commands.send(UiCommand::SetMemory(0, 0x00)).unwrap();
        h.commands.send(UiCommand::StepInstruction).unwrap();
        let _ = h.events.recv_timeout(Duration::from_secs(1)).unwrap(); // state from SetMemory
        let mut saw_executed = false;
        let mut saw_state = false;
        for _ in 0..4 {
            match h.events.recv_timeout(Duration::from_secs(1)) {
                Ok(UiEvent::InstructionExecuted { .. }) => saw_executed = true,
                Ok(UiEvent::State(_)) => saw_state = true,
                Ok(_) => {}
                Err(_) => break,
            }
        }
        assert!(saw_executed);
        assert!(saw_state);
    }
}
