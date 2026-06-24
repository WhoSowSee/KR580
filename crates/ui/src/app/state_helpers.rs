use iced::Theme;
use std::time::{Duration, Instant};

use super::messages::SpeedTier;
use super::state::DesktopApp;
use super::status::StatusKind;
use crate::i18n::Key;

impl DesktopApp {
    pub(crate) fn theme(&self, _window: iced::window::Id) -> Option<Theme> {
        Some(Theme::TokyoNight)
    }

    pub(crate) fn set_status(&mut self, kind: StatusKind) {
        if let Some(rendered) = kind.render(self.lang) {
            self.status = rendered;
            self.status_kind = kind;
        }
    }

    pub(crate) fn set_status_custom(&mut self, text: String) {
        self.status = text;
        self.status_kind = StatusKind::Custom;
    }

    pub(crate) fn refresh_localized_status(&mut self) {
        if let Some(rendered) = self.status_kind.render(self.lang) {
            self.status = rendered;
        }
    }

    pub(crate) fn clear_error_notice(&mut self) {
        self.error_notice = None;
        self.error_notice_dismiss_at = None;
    }

    pub(crate) fn clear_halt_notice(&mut self) {
        self.halt_notice = None;
        self.halt_notice_dismiss_at = None;
    }

    pub(crate) fn raise_halt_notice(&mut self) {
        self.halt_notice = Some(self.lang.t(Key::HaltNotice).to_owned());
        self.halt_notice_dismiss_at = Some(Instant::now() + Duration::from_secs(8));
        self.run_blocked_after_halt = true;
    }

    pub(crate) fn run_new_file(&mut self) {
        self.dispatch(crate::backend::AppCommand::ResetRam);
        self.dispatch(crate::backend::AppCommand::ResetCpu);
        self.running = false;
        self.current_snapshot_path = None;
        self.undo_stack.clear();
        self.mark_saved();
        self.speed_tier = self.default_speed;
        self.set_status(StatusKind::NewFile);
    }

    pub(crate) fn mark_saved(&mut self) {
        self.dirty = false;
        self.saved_cpu = self.snapshot.cpu.clone();
    }

    pub(crate) fn recompute_dirty(&mut self) {
        self.dirty = self.snapshot.cpu != self.saved_cpu;
    }

    pub(crate) fn apply_speed_tier(&mut self, tier: SpeedTier) {
        self.speed_tier = tier;
        let hz = super::tier_hz(tier);
        let interval = Duration::from_micros(1_000_000 / u64::from(hz.max(1)));
        self.dispatch(crate::backend::AppCommand::SetStepInterval(interval));
        let mode = match tier {
            SpeedTier::Max => crate::backend::RunMode::Burst {
                slice: Duration::from_millis(16),
            },
            _ => crate::backend::RunMode::Paced,
        };
        self.dispatch(crate::backend::AppCommand::SetRunMode(mode));
    }
}
