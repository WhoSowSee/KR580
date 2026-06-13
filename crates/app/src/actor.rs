use crate::{AppCommand, AppError, AppEvent, AppSnapshot, Emulator, RunMode};
use crossbeam_channel::{Receiver, Sender, after, never, select, tick};
use std::thread;
use std::time::Duration;

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

    /// Blocks until at least one `StateChanged` is observed (or the
    /// timeout elapses). Without this, UI handlers that read
    /// `self.snapshot` after dispatch race the channel.
    pub fn drain_until_state_change(&self, timeout: std::time::Duration) -> Vec<AppEvent> {
        let deadline = std::time::Instant::now() + timeout;
        let mut events = Vec::new();
        loop {
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
    let initial = emulator.snapshot();
    let mut published_network = initial.devices.network.clone();
    publish(&event_tx, AppEvent::StateChanged(Box::new(initial)));
    let network_poll = tick(Duration::from_millis(50));
    loop {
        // `never()` parks the timer when paused; otherwise the deadline
        // is `step_interval` (Paced) or `slice` (Burst). The slice also
        // bounds `Stop` responsiveness – a press lands within one slice.
        let tick: Receiver<std::time::Instant> = if emulator.is_running() {
            let deadline = match emulator.run_mode() {
                RunMode::Paced => emulator.step_interval(),
                RunMode::Burst { slice } => slice,
            };
            after(deadline)
        } else {
            never()
        };
        select! {
            recv(command_rx) -> command => {
                let Ok(command) = command else { break };
                let shutdown = matches!(command, AppCommand::Shutdown);
                for event in emulator.handle_command(command) {
                    if let AppEvent::StateChanged(snapshot) = &event {
                        published_network = snapshot.devices.network.clone();
                    }
                    publish(&event_tx, event);
                }
                if shutdown {
                    break;
                }
            }
            recv(tick) -> _ => {
                for event in emulator.tick() {
                    if let AppEvent::StateChanged(snapshot) = &event {
                        published_network = snapshot.devices.network.clone();
                    }
                    publish(&event_tx, event);
                }
            }
            recv(network_poll) -> _ => {
                let snapshot = emulator.snapshot();
                if snapshot.devices.network != published_network {
                    published_network = snapshot.devices.network.clone();
                    publish(&event_tx, AppEvent::StateChanged(Box::new(snapshot)));
                }
            }
        }
    }
}

fn publish(event_tx: &Sender<AppEvent>, event: AppEvent) {
    if event_tx.send(event).is_err() {
        tracing::debug!("UI event receiver dropped");
    }
}

pub const MIN_STEP_INTERVAL: Duration = Duration::from_millis(1);
