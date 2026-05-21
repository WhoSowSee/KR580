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

    /// Drains all currently buffered events, blocking until at least one
    /// `AppEvent::StateChanged` is observed (or the timeout elapses, or
    /// the worker has shut down). Used by UI handlers that *must* see
    /// the post-command snapshot before they decide what to do next —
    /// e.g. the step-instruction button reads the new PC to follow it
    /// in the memory list. Without this the snapshot lag turns "click
    /// once" into "click twice", because the first dispatch would race
    /// the worker thread.
    pub fn drain_until_state_change(&self, timeout: std::time::Duration) -> Vec<AppEvent> {
        let deadline = std::time::Instant::now() + timeout;
        let mut events = Vec::new();
        loop {
            // Pull anything already queued first; the StateChanged we
            // are after may already be sitting in the channel.
            for event in self.event_rx.try_iter() {
                let is_state_change = matches!(event, AppEvent::StateChanged(_));
                events.push(event);
                if is_state_change {
                    return events;
                }
            }
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                return events;
            }
            match self.event_rx.recv_timeout(remaining) {
                Ok(event) => {
                    let is_state_change = matches!(event, AppEvent::StateChanged(_));
                    events.push(event);
                    if is_state_change {
                        return events;
                    }
                }
                Err(_) => return events,
            }
        }
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
