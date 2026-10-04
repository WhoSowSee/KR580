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
        #[cfg(target_os = "macos")]
        if let Some(path) = crate::platform::take_pending_path() {
            self.open_associated_file(path);
        }
        self.pull_events();
        let image_task = self.refresh_open_image_contents();
        let now = Instant::now();
        let help_search_task = self.due_help_search_task(now);
        let background_task = match help_search_task {
            Some(task) => Task::batch([image_task, task]),
            None => image_task,
        };
        #[cfg(target_os = "windows")]
        if !self.preferences.file_association_pending
            && let Some(dialog) = self.preferences.settings_dialog.as_mut()
        {
            dialog.file_association_registered = k580_ui::file_assoc::is_registered();
        }
        self.memory.memory_scroll_visible_ticks =
            self.memory.memory_scroll_visible_ticks.saturating_sub(1);
        self.memory.opcode_scroll_visible_ticks =
            self.memory.opcode_scroll_visible_ticks.saturating_sub(1);
        self.panels.monitor_hex_scroll_visible_ticks = self
            .panels
            .monitor_hex_scroll_visible_ticks
            .saturating_sub(1);
        if let Some(deadline) = self.shell.error_notice_dismiss_at
            && now >= deadline
        {
            self.clear_error_notice();
        }
        if let Some(deadline) = self.execution.halt_notice_dismiss_at
            && now >= deadline
        {
            self.clear_halt_notice();
        }
        if self
            .preferences
            .settings_notice
            .is_some_and(|notice| notice.is_expired(now))
        {
            self.preferences.settings_notice = None;
        }
        // A fast run can auto-pause before Tick reads running; its final PC still needs following.
        if self.execution.running || self.execution.pending_follow_pc {
            let was_pending = self.execution.pending_follow_pc;
            self.execution.pending_follow_pc = false;
            if was_pending {
                return Task::batch([background_task, self.follow_pc_during_run()]);
            }
            if self.preferences.follow_pc {
                return Task::batch([background_task, self.follow_pc_during_run()]);
            }
            self.track_pc_in_place();
        }
        background_task
    }

    fn due_help_search_task(&mut self, now: Instant) -> Option<Task<Message>> {
        let request = self
            .shell
            .help_dialog
            .as_mut()?
            .take_due_search_request(self.preferences.lang, now)?;
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

        if generation != self.interaction.mouse_press_generation {
            return Task::none();
        }

        self.document.undo_stack.break_coalescing();

        let resolved = hit.as_ref().and_then(|id| {
            TRACKED
                .into_iter()
                .find(|known| *id == iced::widget::Id::new(known))
        });

        if let Some((guard_generation, input)) = self.interaction.replacement_reconcile_guard.take()
            && generation == guard_generation
        {
            self.interaction.focused_input = Some(input);
            return iced::widget::operation::focus(input);
        }

        if resolved != self.interaction.focused_input {
            self.finish_replacement();
        }

        if self.register.inline_register_just_entered {
            self.register.inline_register_just_entered = false;
            if let Some(id) = hit {
                self.interaction.focused_input = resolved;
                return iced::advanced::widget::operate(crate::runtime::unfocus_except(id))
                    .discard();
            }
            return iced::advanced::widget::operate(crate::runtime::find_focused_optional())
                .map(Message::ResolveFocusedTracker);
        }

        if self.register.inline_register_target.is_some()
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
            self.interaction.focused_input = resolved;
            return iced::advanced::widget::operate(crate::runtime::unfocus_except(id)).discard();
        }
        iced::advanced::widget::operate(crate::runtime::find_focused_optional())
            .map(Message::ResolveFocusedTracker)
    }

    pub(crate) fn handle_esc(&mut self) -> Task<Message> {
        self.document.undo_stack.break_coalescing();
        if self.shell.help_dialog.is_some() {
            self.shell.help_dialog = None;
            return Task::none();
        }
        if self.preferences.settings_dialog.is_some() {
            self.preferences.settings_dialog = None;
            return Task::none();
        }
        if self.shell.about_dialog_open {
            self.shell.about_dialog_open = false;
            return Task::none();
        }
        if self.panels.monitor_open {
            if self.panels.monitor_hex_popup {
                self.panels.monitor_hex_popup = false;
            } else {
                return self.close_monitor();
            }
            return Task::none();
        }
        if self.panels.network_settings_open {
            self.panels.network_settings_open = false;
            self.panels.network_settings_error = None;
            return Task::none();
        }
        if self.panels.network_open {
            return self.close_network();
        }
        if self.panels.printer_open {
            return self.close_printer();
        }
        if self.panels.hdd_open {
            return self.close_hdd();
        }
        if self.panels.floppy_open {
            return self.close_floppy();
        }
        if self.shell.error_notice.is_some() {
            self.clear_error_notice();
            return Task::none();
        }
        if self.execution.halt_notice.is_some() {
            self.clear_halt_notice();
            return Task::none();
        }
        if self.shell.open_menu.is_some() || self.shell.top_menu_focus.is_some() {
            self.close_top_menu();
            return Task::none();
        }
        let resolve = iced::advanced::widget::operate(crate::runtime::find_focused_optional())
            .map(Message::ResolveFocusedTracker);
        if self.interaction.focused_input == Some(REGISTER_INLINE_INPUT_ID) {
            return self.cancel_inline_register_edit().chain(resolve);
        }
        if self.interaction.focused_input == Some(MEMORY_INLINE_INPUT_ID) {
            return self.cancel_inline_memory_edit().chain(resolve);
        }
        if matches!(
            self.interaction.focused_input,
            Some(REGISTER_NAME_INPUT_ID | REGISTER_VALUE_INPUT_ID)
        ) {
            self.finish_replacement();
            self.register.active_register_target = None;
            self.register.inline_register_target = None;
            self.register.register_name_input.clear();
            self.register.register_value_input.clear();
            self.interaction.focused_input = None;
            return resolve;
        }
        if self.memory.view.is_stack() {
            self.disable_stack_view();
            return Task::none();
        }
        self.finish_replacement();
        if self.register.active_register_target.is_some() {
            self.register.active_register_target = None;
            self.register.inline_register_target = None;
            self.register.register_name_input.clear();
            self.register.register_value_input.clear();
            return resolve;
        }
        if self.memory.opcode_dropdown_address.is_some() {
            self.hide_opcode_dropdown();
            self.interaction.focused_input = None;
            return Task::none();
        }
        if self.selected_memory_address().is_some() {
            self.memory.memory_address_input.clear();
            self.memory.memory_value_input.clear();
            self.memory.memory_inline_value_input.clear();
            self.memory.opcode_dropdown_address = None;
            self.memory.opcode_search_input.clear();
            return resolve;
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
