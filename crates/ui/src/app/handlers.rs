use iced::{Task, keyboard};
use std::time::{Duration, Instant};

use super::constants::{
    MEMORY_ADDRESS_INPUT_ID, MEMORY_INLINE_INPUT_ID, MEMORY_VALUE_INPUT_ID,
    REGISTER_INLINE_INPUT_ID, REGISTER_NAME_INPUT_ID, REGISTER_VALUE_INPUT_ID,
};
use super::messages::{Message, SpeedTier};
use super::speed::tier_hz;
use super::state::DesktopApp;

impl DesktopApp {
    pub(crate) fn handle_tick(&mut self) -> Task<Message> {
        self.pull_events();
        self.memory_scroll_visible_ticks = self.memory_scroll_visible_ticks.saturating_sub(1);
        self.opcode_scroll_visible_ticks = self.opcode_scroll_visible_ticks.saturating_sub(1);
        self.monitor_hex_scroll_visible_ticks =
            self.monitor_hex_scroll_visible_ticks.saturating_sub(1);
        let now = Instant::now();
        if let Some(deadline) = self.error_notice_dismiss_at
            && now >= deadline
        {
            self.clear_error_notice();
        }
        if let Some(deadline) = self.halt_notice_dismiss_at
            && now >= deadline
        {
            self.clear_halt_notice();
        }
        if let Some(deadline) = self.info_notice_dismiss_at
            && now >= deadline
        {
            self.clear_info_notice();
        }
        // `pending_follow_pc` covers a fast run that auto-paused
        // inside one tick: by the time we read `running` here it's
        // already false.
        if self.running || self.pending_follow_pc {
            self.pending_follow_pc = false;
            return self.follow_pc_during_run();
        }
        Task::none()
    }

    pub(crate) fn handle_focus_reconciled(
        &mut self,
        hit: Option<iced::widget::Id>,
    ) -> Task<Message> {
        const TRACKED: [&str; 6] = [
            MEMORY_ADDRESS_INPUT_ID,
            MEMORY_VALUE_INPUT_ID,
            REGISTER_NAME_INPUT_ID,
            REGISTER_VALUE_INPUT_ID,
            REGISTER_INLINE_INPUT_ID,
            MEMORY_INLINE_INPUT_ID,
        ];

        self.undo_stack.break_coalescing();

        let resolved = hit.as_ref().and_then(|id| {
            TRACKED
                .into_iter()
                .find(|known| *id == iced::widget::Id::new(known))
        });

        if self.inline_register_just_entered {
            self.inline_register_just_entered = false;
            if let Some(id) = hit {
                self.focused_input = resolved;
                return iced::advanced::widget::operate(crate::runtime::unfocus_except(id))
                    .discard();
            }
            return iced::advanced::widget::operate(crate::runtime::find_focused_optional())
                .map(Message::ResolveFocusedTracker);
        }

        if self.inline_register_target.is_some()
            && !matches!(
                resolved,
                Some(REGISTER_INLINE_INPUT_ID)
                    | Some(MEMORY_INLINE_INPUT_ID)
                    | Some(MEMORY_ADDRESS_INPUT_ID)
                    | Some(MEMORY_VALUE_INPUT_ID)
                    | Some(REGISTER_NAME_INPUT_ID)
                    | Some(REGISTER_VALUE_INPUT_ID)
            )
        {
            return self.cancel_inline_register_edit();
        }

        if let Some(id) = hit {
            self.focused_input = resolved;
            return iced::advanced::widget::operate(crate::runtime::unfocus_except(id)).discard();
        }
        // Miss = dead-space click or layout-race false negative;
        // poll iced for the ground truth.
        iced::advanced::widget::operate(crate::runtime::find_focused_optional())
            .map(Message::ResolveFocusedTracker)
    }

    pub(crate) fn handle_esc(&mut self) -> Task<Message> {
        self.undo_stack.break_coalescing();
        if self.settings_dialog.is_some() {
            self.settings_dialog = None;
            return Task::none();
        }
        if self.monitor_open {
            if self.monitor_hex_popup {
                self.monitor_hex_popup = false;
            } else {
                self.monitor_open = false;
            }
            return Task::none();
        }
        if self.error_notice.is_some() {
            self.clear_error_notice();
            return Task::none();
        }
        if self.halt_notice.is_some() {
            self.clear_halt_notice();
            return Task::none();
        }
        if self.info_notice.is_some() {
            self.clear_info_notice();
            return Task::none();
        }
        if self.open_menu.is_some() {
            self.open_menu = None;
            return Task::none();
        }
        let resolve = iced::advanced::widget::operate(crate::runtime::find_focused_optional())
            .map(Message::ResolveFocusedTracker);
        if self.focused_input == Some(REGISTER_INLINE_INPUT_ID) {
            return self.cancel_inline_register_edit().chain(resolve);
        }
        if self.focused_input == Some(MEMORY_INLINE_INPUT_ID) {
            return self.cancel_inline_memory_edit().chain(resolve);
        }
        self.hide_opcode_dropdown();
        resolve
    }
}

pub(crate) fn tick_interval(running: bool, tier: SpeedTier) -> Duration {
    if running {
        let hz = u64::from(tier_hz(tier).max(1));
        let raw_ms = (1000_u64 / hz).max(16);
        Duration::from_millis(raw_ms.min(100))
    } else {
        Duration::from_millis(100)
    }
}

/// `to_latin(physical_key)` makes Russian-layout keys resolve to the same
/// shortcut as the QWERTY positions.
pub(crate) fn ctrl_shortcut(
    key: &keyboard::Key,
    physical_key: keyboard::key::Physical,
    modifiers: keyboard::Modifiers,
) -> Option<Message> {
    if let Some(direction) = super::register_inline::ctrl_arrow_move(key, modifiers) {
        return Some(Message::RegisterCtrlArrowKey(direction));
    }
    if let keyboard::Key::Named(keyboard::key::Named::Tab) = key {
        return Some(Message::SettingsSectionCycle {
            backward: modifiers.shift(),
        });
    }
    if !modifiers.shift() && !modifiers.alt() && is_comma_key(key, physical_key) {
        return Some(Message::OpenSettings);
    }
    let latin = key.to_latin(physical_key)?;
    let alt = modifiers.alt();
    match (latin, modifiers.shift(), alt) {
        ('n', false, false) => Some(Message::NewFile),
        ('o', false, false) => Some(Message::OpenSnapshot),
        ('s', false, false) => Some(Message::SaveSnapshot),
        ('s', true, false) => Some(Message::SaveSnapshotAs),
        ('i', false, false) => Some(Message::Import),
        ('e', false, false) => Some(Message::Export),
        ('s', false, true) => Some(Message::SaveLegacySnapshot),
        ('o', false, true) => Some(Message::OpenLegacySnapshot),
        ('r', false, false) => Some(Message::ToggleRun),
        ('t', false, false) => Some(Message::StepInstruction),
        ('y', false, false) => Some(Message::StepTact),
        ('r', true, false) => Some(Message::ResetRam),
        ('g', true, false) => Some(Message::ResetCpu),
        ('h', true, false) => Some(Message::ClearHalt),
        ('z', false, false) => Some(Message::Undo),
        ('z', true, false) => Some(Message::Redo),
        _ => None,
    }
}

fn is_comma_key(key: &keyboard::Key, physical_key: keyboard::key::Physical) -> bool {
    if let keyboard::Key::Character(c) = key
        && c.as_str() == ","
    {
        return true;
    }
    matches!(
        physical_key,
        keyboard::key::Physical::Code(keyboard::key::Code::Comma)
    )
}
