use crate::backend::{AppCommand, AppError, AppEvent, AppSnapshot, Emulator, RunMode};
use crossbeam_channel::{Receiver, Sender, after, never, select, tick};
use std::thread;
use std::time::{Duration, Instant};

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
    let mut published_printer = initial.devices.printer.clone();
    publish(&event_tx, AppEvent::StateChanged(Box::new(initial)));
    let device_poll = tick(Duration::from_millis(50));
    let mut next_run_at: Option<Instant> = None;
    loop {
        let run_tick: Receiver<Instant> = if let Some(deadline) = next_run_at {
            after(deadline.saturating_duration_since(Instant::now()))
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
                        published_printer = snapshot.devices.printer.clone();
                    }
                    publish(&event_tx, event);
                }
                next_run_at = schedule_run_tick(&emulator);
                if shutdown {
                    break;
                }
            }
            recv(run_tick) -> _ => {
                for event in emulator.tick() {
                    if let AppEvent::StateChanged(snapshot) = &event {
                        published_network = snapshot.devices.network.clone();
                        published_printer = snapshot.devices.printer.clone();
                    }
                    publish(&event_tx, event);
                }
                next_run_at = schedule_run_tick(&emulator);
            }
            recv(device_poll) -> _ => {
                emulator.bus_mut().printer.poll();
                let snapshot = emulator.snapshot();
                if snapshot.devices.network != published_network
                    || snapshot.devices.printer != published_printer
                {
                    published_network = snapshot.devices.network.clone();
                    published_printer = snapshot.devices.printer.clone();
                    publish(&event_tx, AppEvent::StateChanged(Box::new(snapshot)));
                }
            }
        }
    }
}

fn schedule_run_tick(emulator: &Emulator) -> Option<Instant> {
    emulator.is_running().then(|| {
        let deadline = match emulator.run_mode() {
            RunMode::Paced => emulator.step_interval(),
            RunMode::Burst { slice } => slice,
        };
        Instant::now() + deadline
    })
}

fn publish(event_tx: &Sender<AppEvent>, event: AppEvent) {
    if event_tx.send(event).is_err() {
        tracing::debug!("UI event receiver dropped");
    }
}

pub const MIN_STEP_INTERVAL: Duration = Duration::from_millis(1);
