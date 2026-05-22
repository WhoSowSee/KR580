use crate::{AppCommand, AppError, AppEvent, AppSnapshot, Emulator};
use crossbeam_channel::{Receiver, Sender, after, never, select};
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

/// Worker loop. Two responsibilities:
///
/// 1. Receive commands from the UI and apply them synchronously.
/// 2. While `Run` is armed, fire `tick()` on every `step_interval`
///    deadline so the program advances one instruction at a time
///    and the UI sees a fresh snapshot per step.
///
/// The dual nature is why we use `select!` instead of `recv()`: a
/// blocking `recv()` would freeze the run loop until the user
/// happened to send another command, which is exactly the bug we
/// are fixing. With `select!` the timer ticks independently and the
/// command channel still wakes the worker the instant a press
/// arrives.
fn run_worker(command_rx: Receiver<AppCommand>, event_tx: Sender<AppEvent>) {
    let mut emulator = Emulator::default();
    publish(
        &event_tx,
        AppEvent::StateChanged(Box::new(emulator.snapshot())),
    );
    loop {
        // `never()` parks the timer arm when we are paused, so the
        // select degenerates to a plain `recv()` and we don't burn
        // CPU spinning on a deadline that nobody will read.
        let tick: Receiver<std::time::Instant> = if emulator.is_running() {
            after(emulator.step_interval())
        } else {
            never()
        };
        select! {
            recv(command_rx) -> command => {
                let Ok(command) = command else { break };
                let shutdown = matches!(command, AppCommand::Shutdown);
                for event in emulator.handle_command(command) {
                    publish(&event_tx, event);
                }
                if shutdown {
                    break;
                }
            }
            recv(tick) -> _ => {
                // The tick channel only fires when `is_running()`
                // was true at select-time. Forward the events
                // unconditionally — `tick()` handles its own
                // re-entrancy (halt, budget exhausted, errors) and
                // always returns a fresh snapshot.
                for event in emulator.tick() {
                    publish(&event_tx, event);
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

/// Smallest interval the worker will accept. Mirrors the floor in
/// `Emulator::apply` for `SetStepInterval` so callers that want to
/// know "fastest sensible speed" without poking at internals can
/// reach for this constant.
pub const MIN_STEP_INTERVAL: Duration = Duration::from_millis(1);
