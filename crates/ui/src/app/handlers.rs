use iced::{Task, keyboard};
use std::time::{Duration, Instant};

use super::constants::{
    MEMORY_ADDRESS_INPUT_ID, MEMORY_INLINE_INPUT_ID, MEMORY_VALUE_INPUT_ID, OPCODE_SEARCH_INPUT_ID,
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
        self.import_target_scroll_visible_ticks =
            self.import_target_scroll_visible_ticks.saturating_sub(1);
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
        // `pending_follow_pc` covers a fast run that auto-paused
        // inside one tick: by the time we read `running` here it's
        // already false.
        if self.running || self.pending_follow_pc {
            let was_pending = self.pending_follow_pc;
            self.pending_follow_pc = false;
            if was_pending {
                return self.follow_pc_during_run();
            }
            if self.follow_pc {
                return self.follow_pc_during_run();
            }
            self.track_pc_in_place();
        }
        Task::none()
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
        return Some(Message::RegisterArrowKey(direction));
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
        ('a', false, false) => Some(Message::OpenNetwork),
        ('d', false, false) => Some(Message::OpenHdd),
        ('f', false, false) => Some(Message::OpenFloppy),
        ('p', false, false) => Some(Message::OpenPrinter),
        ('r', false, false) => Some(Message::ToggleRun),
        ('t', false, false) => Some(Message::StepInstruction),
        ('y', false, false) => Some(Message::StepTact),
        ('r', true, false) => Some(Message::ResetRam),
        ('g', true, false) => Some(Message::ResetCpu),
        ('h', false, false) => Some(Message::OpenHelp),
        ('h', true, false) => Some(Message::ClearHalt),
        ('m', false, false) => Some(Message::OpenMonitor),
        ('z', false, false) => Some(Message::Undo),
        ('z', true, false) => Some(Message::Redo),
        _ => None,
    }
}

pub(crate) fn plain_shortcut(
    key: &keyboard::Key,
    physical_key: keyboard::key::Physical,
    modifiers: keyboard::Modifiers,
) -> Option<Message> {
    if modifiers.command() || modifiers.alt() {
        return None;
    }
    match key.to_latin(physical_key)?.to_ascii_lowercase() {
        'e' => Some(Message::OpenOpcodePicker),
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

#[cfg(test)]
mod tests {
    use super::{ctrl_shortcut, plain_shortcut};
    use crate::app::Message;
    use iced::keyboard;
    use iced::keyboard::key::{Code, Physical};
    use std::mem::discriminant;

    fn char_key(value: &str) -> keyboard::Key {
        keyboard::Key::Character(value.into())
    }

    fn physical(code: Code) -> Physical {
        Physical::Code(code)
    }

    fn assert_message(actual: Option<Message>, expected: Message) {
        let actual = actual.expect("shortcut should resolve");
        assert_eq!(discriminant(&actual), discriminant(&expected));
    }

    #[test]
    fn plain_shortcuts_use_physical_key_for_russian_layout() {
        assert_message(
            plain_shortcut(
                &char_key("e"),
                physical(Code::KeyE),
                keyboard::Modifiers::NONE,
            ),
            Message::OpenOpcodePicker,
        );
        assert_message(
            plain_shortcut(
                &char_key("у"),
                physical(Code::KeyE),
                keyboard::Modifiers::NONE,
            ),
            Message::OpenOpcodePicker,
        );
    }

    #[test]
    fn ctrl_shortcuts_use_physical_key_for_russian_layout() {
        let ctrl = keyboard::Modifiers::COMMAND;

        for (typed, code, expected) in [
            ("т", Code::KeyN, Message::NewFile),
            ("щ", Code::KeyO, Message::OpenSnapshot),
            ("ы", Code::KeyS, Message::SaveSnapshot),
            ("ш", Code::KeyI, Message::Import),
            ("у", Code::KeyE, Message::Export),
            ("к", Code::KeyR, Message::ToggleRun),
            ("е", Code::KeyT, Message::StepInstruction),
            ("н", Code::KeyY, Message::StepTact),
            ("р", Code::KeyH, Message::OpenHelp),
            ("в", Code::KeyD, Message::OpenHdd),
            ("а", Code::KeyF, Message::OpenFloppy),
            ("ф", Code::KeyA, Message::OpenNetwork),
            ("з", Code::KeyP, Message::OpenPrinter),
            ("я", Code::KeyZ, Message::Undo),
            ("ь", Code::KeyM, Message::OpenMonitor),
        ] {
            assert_message(
                ctrl_shortcut(&char_key(typed), physical(code), ctrl),
                expected,
            );
        }
    }

    #[test]
    fn shifted_and_alt_ctrl_shortcuts_use_physical_key_for_russian_layout() {
        assert_message(
            ctrl_shortcut(
                &char_key("Ы"),
                physical(Code::KeyS),
                keyboard::Modifiers::COMMAND | keyboard::Modifiers::SHIFT,
            ),
            Message::SaveSnapshotAs,
        );
        assert_message(
            ctrl_shortcut(
                &char_key("К"),
                physical(Code::KeyR),
                keyboard::Modifiers::COMMAND | keyboard::Modifiers::SHIFT,
            ),
            Message::ResetRam,
        );
        assert_message(
            ctrl_shortcut(
                &char_key("П"),
                physical(Code::KeyG),
                keyboard::Modifiers::COMMAND | keyboard::Modifiers::SHIFT,
            ),
            Message::ResetCpu,
        );
        assert_message(
            ctrl_shortcut(
                &char_key("Р"),
                physical(Code::KeyH),
                keyboard::Modifiers::COMMAND | keyboard::Modifiers::SHIFT,
            ),
            Message::ClearHalt,
        );
        assert_message(
            ctrl_shortcut(
                &char_key("Я"),
                physical(Code::KeyZ),
                keyboard::Modifiers::COMMAND | keyboard::Modifiers::SHIFT,
            ),
            Message::Redo,
        );
        assert_message(
            ctrl_shortcut(
                &char_key("б"),
                physical(Code::Comma),
                keyboard::Modifiers::COMMAND,
            ),
            Message::OpenSettings,
        );
    }
}
