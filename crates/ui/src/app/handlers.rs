use iced::Task;
#[cfg(test)]
use iced::keyboard;
use std::time::{Duration, Instant};

use super::constants::{
    MEMORY_ADDRESS_INPUT_ID, MEMORY_INLINE_INPUT_ID, MEMORY_VALUE_INPUT_ID, OPCODE_SEARCH_INPUT_ID,
    REGISTER_INLINE_INPUT_ID, REGISTER_NAME_INPUT_ID, REGISTER_VALUE_INPUT_ID,
};
use super::help::run_help_search;
use super::messages::{Message, SpeedTier};
use super::speed::tier_hz;
use super::state::DesktopApp;

impl DesktopApp {
    pub(crate) fn handle_tick(&mut self) -> Task<Message> {
        self.pull_events();
        let now = Instant::now();
        let help_search_task = self.due_help_search_task(now);
        let registered = k580_ui::file_assoc::is_registered();
        if registered != self.file_association_last_registered {
            self.file_association_last_registered = registered;
            self.file_association_toggle_revision =
                self.file_association_toggle_revision.wrapping_add(1);
        }
        self.memory_scroll_visible_ticks = self.memory_scroll_visible_ticks.saturating_sub(1);
        self.opcode_scroll_visible_ticks = self.opcode_scroll_visible_ticks.saturating_sub(1);
        self.monitor_hex_scroll_visible_ticks =
            self.monitor_hex_scroll_visible_ticks.saturating_sub(1);
        self.import_target_scroll_visible_ticks =
            self.import_target_scroll_visible_ticks.saturating_sub(1);
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
        if self
            .settings_saved_notice
            .is_some_and(|notice| notice.is_expired(now))
        {
            self.settings_saved_notice = None;
        }
        // `pending_follow_pc` covers a fast run that auto-paused
        // inside one tick: by the time we read `running` here it's
        // already false.
        if self.running || self.pending_follow_pc {
            let was_pending = self.pending_follow_pc;
            self.pending_follow_pc = false;
            if was_pending {
                return batch_optional(help_search_task, self.follow_pc_during_run());
            }
            if self.follow_pc {
                return batch_optional(help_search_task, self.follow_pc_during_run());
            }
            self.track_pc_in_place();
        }
        help_search_task.unwrap_or_else(Task::none)
    }

    fn due_help_search_task(&mut self, now: Instant) -> Option<Task<Message>> {
        let request = self
            .help_dialog
            .as_mut()?
            .take_due_search_request(self.lang, now)?;
        Some(Task::perform(
            run_help_search(request),
            Message::HelpSearchFinished,
        ))
    }

    pub(crate) fn handle_focus_reconciled(
        &mut self,
        generation: u64,
        hit: Option<iced::widget::Id>,
    ) -> Task<Message> {
        const TRACKED: [&str; 7] = [
            MEMORY_ADDRESS_INPUT_ID,
            MEMORY_VALUE_INPUT_ID,
            REGISTER_NAME_INPUT_ID,
            REGISTER_VALUE_INPUT_ID,
            REGISTER_INLINE_INPUT_ID,
            MEMORY_INLINE_INPUT_ID,
            OPCODE_SEARCH_INPUT_ID,
        ];

        if generation != self.mouse_press_generation {
            return Task::none();
        }

        self.undo_stack.break_coalescing();

        let resolved = hit.as_ref().and_then(|id| {
            TRACKED
                .into_iter()
                .find(|known| *id == iced::widget::Id::new(known))
        });

        if let Some((guard_generation, input)) = self.replacement_reconcile_guard.take()
            && generation == guard_generation
        {
            self.focused_input = Some(input);
            return iced::widget::operation::focus(input);
        }

        if resolved != self.focused_input {
            self.finish_replacement();
        }

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
        if self.help_dialog.is_some() {
            self.help_dialog = None;
            return Task::none();
        }
        if self.settings_dialog.is_some() {
            self.settings_dialog = None;
            return Task::none();
        }
        if self.about_dialog_open {
            self.about_dialog_open = false;
            return Task::none();
        }
        if self.monitor_open {
            if self.monitor_hex_popup {
                self.monitor_hex_popup = false;
            } else {
                return self.close_monitor();
            }
            return Task::none();
        }
        if self.network_settings_open {
            self.network_settings_open = false;
            self.network_settings_error = None;
            return Task::none();
        }
        if self.network_open {
            return self.close_network();
        }
        if self.printer_open {
            return self.close_printer();
        }
        if self.hdd_open {
            return self.close_hdd();
        }
        if self.floppy_open {
            return self.close_floppy();
        }
        if self.error_notice.is_some() {
            self.clear_error_notice();
            return Task::none();
        }
        if self.halt_notice.is_some() {
            self.clear_halt_notice();
            return Task::none();
        }
        if self.open_menu.is_some() || self.top_menu_focus.is_some() {
            self.close_top_menu();
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
        if matches!(
            self.focused_input,
            Some(REGISTER_NAME_INPUT_ID | REGISTER_VALUE_INPUT_ID)
        ) {
            self.finish_replacement();
            self.active_register_target = None;
            self.inline_register_target = None;
            self.register_name_input.clear();
            self.register_value_input.clear();
            self.focused_input = None;
            return resolve;
        }
        if self.stack_view {
            self.disable_stack_view();
            return Task::none();
        }
        self.finish_replacement();
        if self.active_register_target.is_some() {
            self.active_register_target = None;
            self.inline_register_target = None;
            self.register_name_input.clear();
            self.register_value_input.clear();
            return resolve;
        }
        if self.selected_memory_address().is_some() {
            self.memory_address_input.clear();
            self.memory_value_input.clear();
            self.memory_inline_value_input.clear();
            self.opcode_dropdown_address = None;
            self.opcode_search_input.clear();
            return resolve;
        }
        self.hide_opcode_dropdown();
        resolve
    }
}

fn batch_optional(optional: Option<Task<Message>>, task: Task<Message>) -> Task<Message> {
    match optional {
        Some(optional) => Task::batch([optional, task]),
        None => task,
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
#[cfg(test)]
pub(crate) fn ctrl_shortcut(
    key: &keyboard::Key,
    physical_key: keyboard::key::Physical,
    modifiers: keyboard::Modifiers,
) -> Option<Message> {
    if let Some(direction) = super::register_inline::ctrl_arrow_move(key, modifiers) {
        return Some(Message::RegisterArrowKey(direction));
    }
    if let keyboard::Key::Named(keyboard::key::Named::Tab) = key {
        return Some(Message::SettingsSectionCycle {
            backward: modifiers.shift(),
        });
    }
    crate::app::shortcuts::shortcut_message(
        &crate::persistence::ShortcutSettings::default(),
        physical_key,
        modifiers,
    )
}

#[cfg(test)]
pub(crate) fn alt_shortcut(
    _key: &keyboard::Key,
    physical_key: keyboard::key::Physical,
    modifiers: keyboard::Modifiers,
) -> Option<Message> {
    if modifiers.command() || modifiers.shift() || !modifiers.alt() {
        return None;
    }
    crate::app::shortcuts::shortcut_message(
        &crate::persistence::ShortcutSettings::default(),
        physical_key,
        modifiers,
    )
}

#[cfg(test)]
pub(crate) fn plain_shortcut(
    _key: &keyboard::Key,
    physical_key: keyboard::key::Physical,
    modifiers: keyboard::Modifiers,
) -> Option<Message> {
    if modifiers.command() || modifiers.alt() {
        return None;
    }
    crate::app::shortcuts::shortcut_message(
        &crate::persistence::ShortcutSettings::default(),
        physical_key,
        modifiers,
    )
}
