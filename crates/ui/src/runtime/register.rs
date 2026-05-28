//! Register editor helpers attached to `DesktopApp`.

use crate::app::{
    DesktopApp, Message, REGISTER_INLINE_INPUT_ID, REGISTER_NAME_INPUT_ID, REGISTER_ORDER,
    REGISTER_VALUE_INPUT_ID, RegisterInlineTarget, RegisterMove, parse_register_name,
    register_name,
};
use iced::Task;
use iced::widget::operation;
use k580_app::AppCommand;
use k580_core::RegisterName;

use super::parse::{
    bounded_hex_input, bounded_register_input, parse_hex_u8, register_index, saturating_step_u8,
};

impl DesktopApp {
    pub(crate) fn select_register(&mut self, register: RegisterName) {
        self.selected_register = register;
        self.register_name_input = register_name(register).to_owned();
        self.register_value_input = format!("{:02X}", self.snapshot.cpu.registers.get(register));
        self.active_register_target = None;
        self.inline_register_target = None;
    }

    pub(crate) fn select_register_target(&mut self, target: RegisterInlineTarget) {
        self.selected_register = target.register();
        self.register_name_input = register_name(self.selected_register).to_owned();
        self.register_value_input = format!(
            "{:02X}",
            self.snapshot.cpu.registers.get(self.selected_register)
        );
        self.active_register_target = Some(target);
        self.inline_register_target = None;
        self.focused_input = None;
    }

    pub(crate) fn enter_inline_register(&mut self, target: RegisterInlineTarget) {
        self.selected_register = target.register();
        self.register_name_input = register_name(self.selected_register).to_owned();
        self.register_value_input = format!(
            "{:02X}",
            self.snapshot.cpu.registers.get(self.selected_register)
        );
        self.active_register_target = Some(target);
        self.inline_register_target = Some(target);
    }

    pub(crate) fn change_register_name(&mut self, value: String) {
        let Some(value) = bounded_register_input(&value) else {
            return;
        };

        let before = self.register_name_input.clone();
        self.register_name_input = value;
        self.undo_stack.push_text(
            REGISTER_NAME_INPUT_ID,
            before,
            self.register_name_input.clone(),
        );
        if let Some(register) = parse_register_name(&self.register_name_input) {
            self.selected_register = register;
            self.register_value_input =
                format!("{:02X}", self.snapshot.cpu.registers.get(register));
        }
    }

    pub(crate) fn change_register_value(&mut self, value: String) {
        if let Some(value) = bounded_hex_input(&value, 2) {
            let before = self.register_value_input.clone();
            self.register_value_input = value;
            self.undo_stack.push_text(
                REGISTER_VALUE_INPUT_ID,
                before,
                self.register_value_input.clone(),
            );
        }
    }

    pub(crate) fn change_inline_register_value(
        &mut self,
        target: RegisterInlineTarget,
        value: String,
    ) {
        if self.inline_register_target != Some(target) {
            self.enter_inline_register(target);
        }
        self.change_register_value(value);
    }

    pub(crate) fn display_register_value(&self, register: RegisterName) -> String {
        if parse_register_name(&self.register_name_input) == Some(register) {
            self.register_value_input.clone()
        } else {
            format!("{:02X}", self.snapshot.cpu.registers.get(register))
        }
    }

    pub(crate) fn apply_inline_register_value(
        &mut self,
        target: RegisterInlineTarget,
        backward: bool,
    ) -> Task<Message> {
        self.selected_register = target.register();
        self.register_name_input = register_name(self.selected_register).to_owned();
        self.inline_register_target = Some(target);
        let next = target.adjacent(backward);
        if let Some(next) = next {
            self.apply_register_with_step_selection(next.register());
            self.enter_inline_register(next);
            self.focused_input = Some(REGISTER_INLINE_INPUT_ID);
            operation::focus(REGISTER_INLINE_INPUT_ID)
        } else {
            self.apply_register_inner(None);
            self.select_register_target(target);
            self.focused_input = None;
            Task::none()
        }
    }

    pub(crate) fn cancel_inline_register_edit(&mut self) -> Task<Message> {
        if let Some(target) = self.inline_register_target {
            let register = target.register();
            self.selected_register = register;
            self.register_name_input = register_name(register).to_owned();
            self.register_value_input =
                format!("{:02X}", self.snapshot.cpu.registers.get(register));
            self.active_register_target = Some(target);
        }
        self.inline_register_target = None;
        self.focused_input = None;
        Task::none()
    }

    pub(crate) fn navigate_active_register_target(&mut self, direction: RegisterMove) {
        let Some(target) = self.active_register_target else {
            return;
        };
        if let Some(next) = target.navigate(direction) {
            self.select_register_target(next);
        }
    }

    pub(crate) fn navigate_inline_register_target(
        &mut self,
        direction: RegisterMove,
    ) -> Task<Message> {
        if self.focused_input != Some(REGISTER_INLINE_INPUT_ID) {
            return Task::none();
        }

        let Some(target) = self.inline_register_target else {
            return Task::none();
        };

        let Some(next) = target.navigate(direction) else {
            return operation::focus(REGISTER_INLINE_INPUT_ID);
        };

        self.enter_inline_register(next);
        self.focused_input = Some(REGISTER_INLINE_INPUT_ID);
        operation::focus(REGISTER_INLINE_INPUT_ID)
    }

    /// Bumps the register value buffered in the editor by `delta`,
    /// saturating at `0x00`/`0xFF`. The byte is *not* written to the CPU
    /// here — applying still requires Enter, the same contract the user
    /// gets when typing the value manually. This way ArrowUp on `00` or
    /// ArrowDown on `FF` simply does nothing instead of silently
    /// committing a wrap-around write.
    pub(crate) fn step_register_value_input(&mut self, delta: i32) {
        let current = parse_hex_u8(&self.register_value_input).unwrap_or(0);
        let next = saturating_step_u8(current, delta);
        self.register_value_input = format!("{next:02X}");
    }

    pub(crate) fn step_register(&mut self, delta: i32) {
        let index = register_index(self.selected_register);
        let len = REGISTER_ORDER.len() as i32;
        let next = (index as i32 + delta).rem_euclid(len) as usize;
        self.select_register(REGISTER_ORDER[next]);
    }

    /// Same as the textbook "apply" gesture (parse name + value,
    /// dispatch `SetRegister`, push the resulting `Cpu` undo
    /// entry), but tags the entry with a `(register_before,
    /// register_after)` selection so Ctrl+Z later restores the
    /// editor to whichever register the user was *editing* (the
    /// one they pressed Enter from), not whichever register the
    /// follow-on step walked to. Used by `apply_register_and_step`;
    /// the redo direction restores the post-step register.
    fn apply_register_with_step_selection(&mut self, register_after: RegisterName) {
        let register_before = self.selected_register;
        // `parse_register_name` inside `apply_register_inner` may
        // re-point `selected_register` if the name field was edited
        // before Enter — e.g. user typed "C" into the name input,
        // hit Enter without explicitly clicking the value field.
        // We capture the "before" selection *after* that
        // reconciliation so the rewound state matches what the user
        // actually saw at the moment of Enter, not whatever
        // selection was current before they retyped the name.
        self.apply_register_inner(Some((register_before, register_after)));
    }

    /// Internal worker shared by every "Enter in the register
    /// editor" path. The dispatch is inlined here (rather than
    /// going through `dispatch_with_undo`) because we need to push
    /// a *register-aware* `Cpu` entry: `dispatch_with_undo` always
    /// pushes the plain variant, which loses the selection. The
    /// before/after capture and `push_cpu_with_register_selection`
    /// match what `dispatch_with_undo` does, just with the extra
    /// selection field threaded through.
    ///
    /// `register_selection` is `Some((before, after))` when the
    /// caller wants Ctrl+Z to teleport the register panel back to
    /// `before` (and Ctrl+Shift+Z to push it forward to `after`).
    /// Currently only `apply_register_with_step_selection` uses
    /// that path; pass `None` from any future site where the
    /// register cursor does not move as part of the gesture.
    fn apply_register_inner(&mut self, register_selection: Option<(RegisterName, RegisterName)>) {
        if let Some(register) = parse_register_name(&self.register_name_input) {
            self.selected_register = register;
        } else {
            self.register_name_input = register_name(self.selected_register).to_owned();
        }

        match parse_hex_u8(&self.register_value_input) {
            Ok(value) => {
                // Pressing Enter is the moment the byte hits the CPU,
                // so this is where the worker mutation enters the undo
                // timeline. Break text coalescing first — the next
                // keystroke is logically a fresh edit, not a
                // continuation of whatever produced this commit.
                self.undo_stack.break_coalescing();
                let before = self.snapshot.cpu.clone();
                self.dispatch_sync(AppCommand::SetRegister(self.selected_register, value));
                let after = self.snapshot.cpu.clone();
                // Mirror the `dirty` flip from `dispatch_with_undo`:
                // the inlined dispatch path bypasses that helper so
                // it has to keep the dirty bookkeeping in sync on
                // its own. Same gate (a no-op write must not flip
                // the bit) and same observable effect — Ctrl+S is
                // offered the moment the byte actually changes.
                if before != after {
                    self.dirty = true;
                }
                // With no follow-on step, fall through to the
                // plain `push_cpu` path. Otherwise tag the entry
                // with the (before, after) register pair so
                // undo/redo also rewind the selection.
                match register_selection {
                    Some(selection) => self.undo_stack.push_cpu_with_register_selection(
                        before,
                        after,
                        Some(selection),
                    ),
                    None => self.undo_stack.push_cpu(before, after),
                }
            }
            Err(error) => self.status = error,
        }
    }

    /// Plain Enter handler for the register editor: applies the typed
    /// register value, cycles to the next/previous register, and keeps
    /// focus inside the register editor (whichever of the two fields the
    /// user was in). The memory list and the memory editor are not
    /// touched.
    pub(crate) fn apply_register_and_step(&mut self, backward: bool) -> Task<Message> {
        let stay_on_value = self.focused_input == Some(REGISTER_VALUE_INPUT_ID);
        // Compute the post-step register *before* dispatching the
        // SetRegister write so the undo entry's `after` selection
        // matches the register the user is about to land on. Doing
        // it after `apply_register_inner` would be just as correct
        // numerically (selected_register has not moved yet) but
        // muddies the data flow — the entry needs to be pushed
        // with the post-step register baked in.
        let delta = if backward { -1 } else { 1 };
        let register_before = self.selected_register;
        let index = register_index(register_before);
        let len = REGISTER_ORDER.len() as i32;
        let next = (index as i32 + delta).rem_euclid(len) as usize;
        let register_after = REGISTER_ORDER[next];

        self.apply_register_with_step_selection(register_after);
        // `step_register` walks `selected_register` and refreshes
        // the name/value buffers from the freshly written CPU. We
        // pass the same `register_after` we baked into the undo
        // entry so the visible state matches what redo would
        // restore.
        self.select_register(register_after);
        let target = if stay_on_value {
            REGISTER_VALUE_INPUT_ID
        } else {
            REGISTER_NAME_INPUT_ID
        };
        self.focused_input = Some(target);
        operation::focus(target)
    }
}
