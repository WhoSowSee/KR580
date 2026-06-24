use std::time::{Duration, Instant};

use super::{Emulator, MAX_INSTRUCTIONS_PER_RUN};
use crate::backend::{AppEvent, RunMode};

impl Emulator {
    pub fn tick(&mut self) -> Vec<AppEvent> {
        let mut events = Vec::new();
        if !self.running {
            // Defensive: a tick after `Stop` still gets a snapshot so the
            // UI doesn't lose its recovery path.
            events.push(AppEvent::StateChanged(Box::new(self.snapshot())));
            return events;
        }
        if self.cpu.halted {
            self.running = false;
            events.push(AppEvent::HaltStateChanged(true));
            events.push(AppEvent::Stopped);
            events.push(AppEvent::StateChanged(Box::new(self.snapshot())));
            return events;
        }
        if self.instructions_since_run >= MAX_INSTRUCTIONS_PER_RUN {
            self.running = false;
            events.push(AppEvent::Stopped);
            events.push(AppEvent::StateChanged(Box::new(self.snapshot())));
            return events;
        }

        match self.run_mode {
            RunMode::Paced => self.tick_paced(&mut events),
            RunMode::Burst { slice } => self.tick_burst(slice, &mut events),
        }

        events.push(AppEvent::StateChanged(Box::new(self.snapshot())));
        events
    }

    fn tick_paced(&mut self, events: &mut Vec<AppEvent>) {
        match self.cpu.step_instruction(&mut self.bus) {
            Ok(outcome) => {
                self.instructions_since_run += 1;
                events.push(AppEvent::InstructionBoundaryReached(outcome));
                if self.cpu.halted {
                    self.running = false;
                    events.push(AppEvent::HaltStateChanged(true));
                    events.push(AppEvent::Stopped);
                }
            }
            Err(error) => {
                self.running = false;
                events.push(AppEvent::ErrorRaised(error.into()));
                events.push(AppEvent::Stopped);
            }
        }
    }

    /// Wall-time is checked every 64 instructions so `Instant::now()`
    /// doesn't dominate the hot loop.
    fn tick_burst(&mut self, slice: Duration, events: &mut Vec<AppEvent>) {
        let slice = slice.max(Duration::from_millis(1));
        let started = Instant::now();
        let mut since_check: u32 = 0;
        loop {
            if self.instructions_since_run >= MAX_INSTRUCTIONS_PER_RUN {
                self.running = false;
                events.push(AppEvent::Stopped);
                return;
            }
            match self.cpu.step_instruction(&mut self.bus) {
                Ok(_outcome) => {
                    self.instructions_since_run += 1;
                    if self.cpu.halted {
                        self.running = false;
                        events.push(AppEvent::HaltStateChanged(true));
                        events.push(AppEvent::Stopped);
                        return;
                    }
                }
                Err(error) => {
                    self.running = false;
                    events.push(AppEvent::ErrorRaised(error.into()));
                    events.push(AppEvent::Stopped);
                    return;
                }
            }
            since_check = since_check.wrapping_add(1);
            if since_check >= 64 {
                since_check = 0;
                if started.elapsed() >= slice {
                    return;
                }
            }
        }
    }
}
