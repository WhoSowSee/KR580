use crate::backend::{AppCommand, AppError, AppEvent, AppSnapshot, Emulator, RunMode};
use crossbeam_channel::{Receiver, Sender, after, never, select, tick, unbounded};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

pub struct EmulatorHandle {
    command_tx: Sender<AppCommand>,
    event_rx: Receiver<AppEvent>,
    critical_rx: Receiver<AppEvent>,
    state_mailbox: Arc<Mutex<Option<Box<AppSnapshot>>>>,
    next_request_id: AtomicU64,
}

impl EmulatorHandle {
    pub fn send(&self, command: AppCommand) -> Result<(), AppError> {
        self.command_tx
            .try_send(command)
            .map_err(|error| match error {
                crossbeam_channel::TrySendError::Full(_) => {
                    crate::devices::DeviceError::Busy.into()
                }
                crossbeam_channel::TrySendError::Disconnected(_) => AppError::WorkerStopped,
            })
    }

    pub fn send_request(&self, command: AppCommand) -> Result<crate::backend::RequestId, AppError> {
        let id = crate::backend::RequestId(self.next_request_id.fetch_add(1, Ordering::Relaxed));
        self.send(AppCommand::Request {
            id,
            command: Box::new(command),
        })?;
        Ok(id)
    }

    fn take_snapshot(&self) -> Option<Box<AppSnapshot>> {
        self.state_mailbox
            .lock()
            .expect("state mailbox poisoned")
            .take()
    }

    pub fn drain_events(&self) -> Vec<AppEvent> {
        let mut events: Vec<_> = self.event_rx.try_iter().collect();
        if let Some(snapshot) = self.take_snapshot() {
            events.push(AppEvent::StateChanged(snapshot));
        }
        events.extend(self.critical_rx.try_iter());
        events
    }

    /// Drains events until a latest-state mailbox value is available.
    pub fn drain_until_state_change(&self, timeout: std::time::Duration) -> Vec<AppEvent> {
        let deadline = std::time::Instant::now() + timeout;
        let mut events = Vec::new();
        loop {
            for event in self.event_rx.try_iter().chain(self.critical_rx.try_iter()) {
                let is_state_change = matches!(event, AppEvent::StateChanged(_));
                events.push(event);
                if is_state_change {
                    return events;
                }
            }
            if let Some(snapshot) = self.take_snapshot() {
                events.push(AppEvent::StateChanged(snapshot));
                return events;
            }
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                return events;
            }
            std::thread::sleep(remaining.min(Duration::from_millis(1)));
        }
    }

    pub fn drain_until_request_finished(
        &self,
        request_id: crate::backend::RequestId,
        timeout: std::time::Duration,
    ) -> Vec<AppEvent> {
        let deadline = std::time::Instant::now() + timeout;
        let mut events = Vec::new();
        loop {
            for event in self.event_rx.try_iter().chain(self.critical_rx.try_iter()) {
                let finished = matches!(
                    &event,
                    AppEvent::CommandFinished { id, .. } if *id == request_id
                );
                events.push(event);
                if finished {
                    if let Some(snapshot) = self.take_snapshot() {
                        events.push(AppEvent::StateChanged(snapshot));
                    }
                    return events;
                }
            }
            if let Some(snapshot) = self.take_snapshot() {
                events.push(AppEvent::StateChanged(snapshot));
            }
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                return events;
            }
            match self.critical_rx.recv_timeout(remaining) {
                Ok(event) => {
                    let finished = matches!(
                        &event,
                        AppEvent::CommandFinished { id, .. } if *id == request_id
                    );
                    events.push(event);
                    if finished {
                        if let Some(snapshot) = self.take_snapshot() {
                            events.push(AppEvent::StateChanged(snapshot));
                        }
                        return events;
                    }
                }
                Err(_) => return events,
            }
        }
    }
}

pub fn spawn_emulator() -> EmulatorHandle {
    let (command_tx, command_rx) = crossbeam_channel::bounded::<AppCommand>(256);
    let (event_tx, event_rx) = crossbeam_channel::bounded::<AppEvent>(256);
    let (critical_tx, critical_rx) = crossbeam_channel::bounded::<AppEvent>(1024);
    let state_mailbox = Arc::new(Mutex::new(None));
    thread::spawn({
        let state_mailbox = state_mailbox.clone();
        move || run_worker(command_rx, event_tx, critical_tx, state_mailbox)
    });
    EmulatorHandle {
        command_tx,
        event_rx,
        critical_rx,
        state_mailbox,
        next_request_id: AtomicU64::new(1),
    }
}

pub fn initial_snapshot() -> AppSnapshot {
    Emulator::default().snapshot()
}

fn run_worker(
    command_rx: Receiver<AppCommand>,
    event_tx: Sender<AppEvent>,
    critical_tx: Sender<AppEvent>,
    state_mailbox: Arc<Mutex<Option<Box<AppSnapshot>>>>,
) {
    let mut emulator = Emulator::default();
    let (io_tx, io_rx) = unbounded::<crate::backend::emulator::io::IoCompletion>();
    let io_worker = match crate::backend::emulator::io::IoWorker::new(io_tx) {
        Ok(worker) => worker,
        Err(error) => {
            publish(
                &event_tx,
                &critical_tx,
                &state_mailbox,
                AppEvent::ErrorRaised(error),
            );
            return;
        }
    };
    let initial = emulator.snapshot();
    let mut published_network = emulator.bus().network.revision();
    publish(
        &event_tx,
        &critical_tx,
        &state_mailbox,
        AppEvent::StateChanged(Box::new(initial)),
    );
    let device_poll = tick(Duration::from_millis(50));
    let mut next_run_at: Option<Instant> = None;
    loop {
        let run_tick: Receiver<Instant> = if let Some(deadline) = next_run_at {
            after(deadline.saturating_duration_since(Instant::now()))
        } else {
            never()
        };
        let network_revision = emulator.bus().network.revision();
        select! {
            recv(command_rx) -> command => {
                let Ok(command) = command else { break };
                let shutdown = matches!(&command, AppCommand::Shutdown);
                if let AppCommand::Request { id, command } = command {
                    if let Some(events) = emulator.start_io_request(id, &command, &io_worker) {
                        for event in events {
                            publish(&event_tx, &critical_tx, &state_mailbox, event);
                        }
                        next_run_at = schedule_run_tick(&emulator);
                        continue;
                    }
                    for event in emulator.handle_command(AppCommand::Request { id, command }) {
                        if matches!(event, AppEvent::StateChanged(_)) {
                            published_network = network_revision;
                        }
                        publish(&event_tx, &critical_tx, &state_mailbox, event);
                    }
                } else {
                    for event in emulator.handle_command(command) {
                        if matches!(event, AppEvent::StateChanged(_)) {
                            published_network = network_revision;
                        }
                        publish(&event_tx, &critical_tx, &state_mailbox, event);
                    }
                }
                next_run_at = schedule_run_tick(&emulator);
                if shutdown {
                    break;
                }
            }
            recv(run_tick) -> _ => {
                for event in emulator.tick() {
                    if matches!(event, AppEvent::StateChanged(_)) {
                        published_network = network_revision;
                    }
                    publish(&event_tx, &critical_tx, &state_mailbox, event);
                }
                next_run_at = schedule_run_tick(&emulator);
            }
            recv(device_poll) -> _ => {
                if let Some(snapshot) = emulator.poll_devices(&mut published_network) {
                    publish(
                        &event_tx,
                        &critical_tx,
                        &state_mailbox,
                        AppEvent::StateChanged(Box::new(snapshot)),
                    );
                }
            }
            recv(io_rx) -> completion => {
                let Ok(completion) = completion else { break };
                for event in emulator.finish_io(completion) {
                    if matches!(event, AppEvent::StateChanged(_)) {
                        published_network = network_revision;
                    }
                    publish(&event_tx, &critical_tx, &state_mailbox, event);
                }
                next_run_at = schedule_run_tick(&emulator);
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

fn publish(
    event_tx: &Sender<AppEvent>,
    critical_tx: &Sender<AppEvent>,
    state_mailbox: &Arc<Mutex<Option<Box<AppSnapshot>>>>,
    event: AppEvent,
) {
    if let AppEvent::StateChanged(snapshot) = event {
        *state_mailbox.lock().expect("state mailbox poisoned") = Some(snapshot);
        return;
    }
    let critical = matches!(
        event,
        AppEvent::ErrorRaised(_)
            | AppEvent::Stopped
            | AppEvent::HaltStateChanged(_)
            | AppEvent::CommandFinished { .. }
    );
    let result = if critical {
        critical_tx.send(event).map_err(|_| ())
    } else {
        event_tx.try_send(event).map_err(|_| ())
    };
    if result.is_err() {
        tracing::debug!("UI event receiver dropped or event queue is full");
    }
}

pub const MIN_STEP_INTERVAL: Duration = Duration::from_millis(1);
