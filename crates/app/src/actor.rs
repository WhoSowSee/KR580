use crate::{AppCommand, AppError, AppEvent, AppSnapshot, Emulator};
use crossbeam_channel::{Receiver, Sender};
use std::thread;

pub struct EmulatorHandle {
    command_tx: Sender<AppCommand>,
    event_rx: Receiver<AppEvent>,
}

impl EmulatorHandle {
    pub fn send(&self, command: AppCommand) -> Result<(), AppError> {
        self.command_tx
            .send(command)
            .map_err(|_| AppError::WorkerStopped)
    }

    pub fn drain_events(&self) -> Vec<AppEvent> {
        self.event_rx.try_iter().collect()
    }

    pub fn command_sender(&self) -> Sender<AppCommand> {
        self.command_tx.clone()
    }

    pub fn event_receiver(&self) -> Receiver<AppEvent> {
        self.event_rx.clone()
    }
}

pub fn spawn_emulator() -> EmulatorHandle {
    let (command_tx, command_rx) = crossbeam_channel::unbounded::<AppCommand>();
    let (event_tx, event_rx) = crossbeam_channel::unbounded::<AppEvent>();
    thread::spawn(move || run_worker(command_rx, event_tx));
    EmulatorHandle {
        command_tx,
        event_rx,
    }
}

pub fn initial_snapshot() -> AppSnapshot {
    Emulator::default().snapshot()
}

fn run_worker(command_rx: Receiver<AppCommand>, event_tx: Sender<AppEvent>) {
    let mut emulator = Emulator::default();
    publish(
        &event_tx,
        AppEvent::StateChanged(Box::new(emulator.snapshot())),
    );
    while let Ok(command) = command_rx.recv() {
        let shutdown = matches!(command, AppCommand::Shutdown);
        for event in emulator.handle_command(command) {
            publish(&event_tx, event);
        }
        if shutdown {
            break;
        }
    }
}

fn publish(event_tx: &Sender<AppEvent>, event: AppEvent) {
    if event_tx.send(event).is_err() {
        tracing::debug!("UI event receiver dropped");
    }
}
