use std::time::{Duration, Instant};

use crate::i18n::Key;

const SETTINGS_NOTICE_DURATION: Duration = Duration::from_secs(2);
const ENTER_DURATION: Duration = Duration::from_millis(180);
const EXIT_DURATION: Duration = Duration::from_millis(240);
const REFRESH_PULSE_SPLIT: f32 = 0.4;

#[derive(Clone, Copy, Debug)]
pub(crate) struct SettingsNotice {
    message_key: Key,
    started_at: Instant,
    entrance_from: SettingsNoticePresentation,
    refreshed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SettingsNoticePresentation {
    pub(crate) opacity: f32,
    pub(crate) offset_y: f32,
}

const INITIAL_PRESENTATION: SettingsNoticePresentation = SettingsNoticePresentation {
    opacity: 0.2,
    offset_y: -10.0,
};
const REFRESH_PULSE_PRESENTATION: SettingsNoticePresentation = SettingsNoticePresentation {
    opacity: 0.92,
    offset_y: -2.0,
};
const VISIBLE_PRESENTATION: SettingsNoticePresentation = SettingsNoticePresentation {
    opacity: 1.0,
    offset_y: 0.0,
};

impl SettingsNotice {
    pub(crate) fn new(message_key: Key, started_at: Instant) -> Self {
        Self {
            message_key,
            started_at,
            entrance_from: INITIAL_PRESENTATION,
            refreshed: false,
        }
    }

    pub(crate) fn restarted(self, message_key: Key, started_at: Instant) -> Self {
        Self {
            message_key,
            started_at,
            entrance_from: self.presentation(started_at),
            refreshed: true,
        }
    }

    pub(crate) fn message_key(self) -> Key {
        self.message_key
    }

    pub(crate) fn is_expired(self, now: Instant) -> bool {
        now.saturating_duration_since(self.started_at) >= SETTINGS_NOTICE_DURATION
    }

    pub(crate) fn presentation(self, now: Instant) -> SettingsNoticePresentation {
        let elapsed = now.saturating_duration_since(self.started_at);
        if elapsed < ENTER_DURATION {
            let progress = elapsed.as_secs_f32() / ENTER_DURATION.as_secs_f32();
            return self.entrance_presentation(progress);
        }

        let exit_started_at = SETTINGS_NOTICE_DURATION - EXIT_DURATION;
        if elapsed >= exit_started_at {
            let progress = ((elapsed - exit_started_at).as_secs_f32()
                / EXIT_DURATION.as_secs_f32())
            .clamp(0.0, 1.0);
            let eased = progress.powi(3);
            return SettingsNoticePresentation {
                opacity: 1.0 - eased,
                offset_y: -4.0 * eased,
            };
        }

        VISIBLE_PRESENTATION
    }

    fn entrance_presentation(self, progress: f32) -> SettingsNoticePresentation {
        if !self.refreshed {
            return interpolate_presentation(
                self.entrance_from,
                VISIBLE_PRESENTATION,
                ease_out_cubic(progress),
            );
        }

        if progress < REFRESH_PULSE_SPLIT {
            return interpolate_presentation(
                self.entrance_from,
                REFRESH_PULSE_PRESENTATION,
                smoothstep(progress / REFRESH_PULSE_SPLIT),
            );
        }

        interpolate_presentation(
            REFRESH_PULSE_PRESENTATION,
            VISIBLE_PRESENTATION,
            smoothstep((progress - REFRESH_PULSE_SPLIT) / (1.0 - REFRESH_PULSE_SPLIT)),
        )
    }
}

fn interpolate_presentation(
    from: SettingsNoticePresentation,
    to: SettingsNoticePresentation,
    progress: f32,
) -> SettingsNoticePresentation {
    SettingsNoticePresentation {
        opacity: interpolate(from.opacity, to.opacity, progress),
        offset_y: interpolate(from.offset_y, to.offset_y, progress),
    }
}

fn interpolate(from: f32, to: f32, progress: f32) -> f32 {
    from + (to - from) * progress
}

fn ease_out_cubic(progress: f32) -> f32 {
    1.0 - (1.0 - progress).powi(3)
}

fn smoothstep(progress: f32) -> f32 {
    progress * progress * (3.0 - 2.0 * progress)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presentation_enters_holds_and_expires_in_two_seconds() {
        let started_at = Instant::now();
        let notice = SettingsNotice::new(Key::SettingsSavedNotice, started_at);

        let entering = notice.presentation(started_at);
        assert_eq!(entering.opacity, 0.2);
        assert_eq!(entering.offset_y, -10.0);

        let holding = notice.presentation(started_at + Duration::from_secs(1));
        assert_eq!(holding.opacity, 1.0);
        assert_eq!(holding.offset_y, 0.0);

        let exiting = notice.presentation(started_at + Duration::from_millis(1_900));
        assert!(exiting.opacity < 1.0);
        assert!(exiting.offset_y < 0.0);
        assert!(!notice.is_expired(started_at + Duration::from_millis(1_999)));
        assert!(notice.is_expired(started_at + SETTINGS_NOTICE_DURATION));
    }

    #[test]
    fn restarting_notice_preserves_current_frame_and_full_lifetime() {
        let first_start = Instant::now();
        let replacement_start = first_start + Duration::from_secs(1);
        let first = SettingsNotice::new(Key::SettingsSavedNotice, first_start);
        let replacement = first.restarted(Key::SettingsResetNotice, replacement_start);

        assert_eq!(
            replacement.presentation(replacement_start),
            first.presentation(replacement_start)
        );
        assert!(!replacement.is_expired(first_start + SETTINGS_NOTICE_DURATION));
        assert!(replacement.is_expired(replacement_start + SETTINGS_NOTICE_DURATION));
        assert_eq!(replacement.message_key(), Key::SettingsResetNotice);
    }

    #[test]
    fn restart_from_visible_state_uses_a_subtle_pulse() {
        let started_at = Instant::now();
        let notice = SettingsNotice::new(Key::SettingsSavedNotice, started_at);
        let restart_at = started_at + Duration::from_secs(1);
        let restarted = notice.restarted(Key::SettingsSavedNotice, restart_at);

        assert_eq!(restarted.presentation(restart_at), VISIBLE_PRESENTATION);
        let pulse = restarted.presentation(restart_at + Duration::from_millis(90));
        assert!((0.91..0.95).contains(&pulse.opacity));
        assert!((-2.1..-1.0).contains(&pulse.offset_y));
        assert_eq!(
            restarted.presentation(restart_at + ENTER_DURATION),
            VISIBLE_PRESENTATION
        );
    }

    #[test]
    fn repeated_restarts_remain_continuous_during_entry() {
        let started_at = Instant::now();
        let first = SettingsNotice::new(Key::SettingsSavedNotice, started_at);
        let first_restart_at = started_at + Duration::from_millis(60);
        let first_restarted = first.restarted(Key::SettingsSavedNotice, first_restart_at);
        let second_restart_at = first_restart_at + Duration::from_millis(40);
        let before_second_restart = first_restarted.presentation(second_restart_at);
        let second_restarted =
            first_restarted.restarted(Key::SettingsShortcutsResetNotice, second_restart_at);

        assert_eq!(
            second_restarted.presentation(second_restart_at),
            before_second_restart
        );
        assert_eq!(
            second_restarted.presentation(second_restart_at + ENTER_DURATION),
            SettingsNoticePresentation {
                opacity: 1.0,
                offset_y: 0.0,
            }
        );
    }
}
