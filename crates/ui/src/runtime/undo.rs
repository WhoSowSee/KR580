//! Apply-side of the undo/redo timeline.
//!
//! The storage and coalescing rules live in `app::undo`; this module
//! is the bridge between the popped `UndoEntry` and the world it has
//! to mutate. Two responsibilities:
//!
//! 1. **Text entries** — restore the matching `DesktopApp::*_input`
//!    field. Pure cosmetic state, no worker involvement.
//! 2. **CPU entries** — replay the snapshot through
//!    `AppCommand::ApplyCpuState`, which the worker treats as a stop
//!    + state-replace. The handler also re-derives the spinner /
//!      inline editor against the rewound `cpu.pc` so the visible
//!      address column lines up with the restored state — without
//!      that the highlight would still point at the post-mutation row
//!      even though memory/registers had moved underneath.
//!
//! `apply_undo` and `apply_redo` are deliberately symmetrical: the
//! only difference is which stack they pop and which half of the
//! entry (`before` vs `after`) they apply.

use crate::app::{
    DesktopApp, MEMORY_ADDRESS_INPUT_ID, MEMORY_INLINE_INPUT_ID, MEMORY_VALUE_INPUT_ID,
    Message, OPCODE_SEARCH_INPUT_ID, REGISTER_NAME_INPUT_ID, REGISTER_VALUE_INPUT_ID, UndoEntry,
};
use iced::Task;
use iced::widget::operation;
use k580_app::AppCommand;
use k580_core::{Cpu8080State, RegisterName};

impl DesktopApp {
    /// Pops the top undo entry and applies it. Ctrl+Z handler.
    /// `None` on the empty stack — the user gets a status-bar hint
    /// rather than a silent no-op so they can tell the press was
    /// received but had nothing to roll back.
    pub(crate) fn apply_undo(&mut self) -> Task<Message> {
        let Some(entry) = self.undo_stack.pop_undo() else {
            self.status = "Нечего отменять".to_owned();
            return Task::none();
        };
        match entry {
            UndoEntry::Text { field, before, .. } => {
                self.restore_text_field(field, before);
                Task::none()
            }
            UndoEntry::Cpu {
                before,
                register_selection,
                ..
            } => {
                let selection = register_selection.map(|(before_reg, _)| before_reg);
                self.replay_cpu_state(*before, selection)
            }
        }
    }

    /// Pops the top redo entry and applies it. Ctrl+Shift+Z handler.
    pub(crate) fn apply_redo(&mut self) -> Task<Message> {
        let Some(entry) = self.undo_stack.pop_redo() else {
            self.status = "Нечего вернуть".to_owned();
            return Task::none();
        };
        match entry {
            UndoEntry::Text { field, after, .. } => {
                self.restore_text_field(field, after);
                Task::none()
            }
            UndoEntry::Cpu {
                after,
                register_selection,
                ..
            } => {
                let selection = register_selection.map(|(_, after_reg)| after_reg);
                self.replay_cpu_state(*after, selection)
            }
        }
    }

    /// Writes `value` into whichever `DesktopApp` text field the
    /// `field` identifier maps to. Identifiers come from the static
    /// strings declared in `app::constants`, so a stable match is
    /// safe; an unknown id is a programming error and we silently
    /// drop the entry rather than panic — better to lose one undo
    /// step than to bring the app down.
    fn restore_text_field(&mut self, field: &'static str, value: String) {
        match field {
            MEMORY_ADDRESS_INPUT_ID => self.memory_address_input = value,
            MEMORY_VALUE_INPUT_ID => self.memory_value_input = value,
            MEMORY_INLINE_INPUT_ID => self.memory_inline_value_input = value,
            REGISTER_NAME_INPUT_ID => self.register_name_input = value,
            REGISTER_VALUE_INPUT_ID => self.register_value_input = value,
            OPCODE_SEARCH_INPUT_ID => self.opcode_search_input = value,
            _ => {}
        }
    }

    /// Sends `state` to the worker via `AppCommand::ApplyCpuState`
    /// and reconciles the UI side. The blocking `dispatch_sync`
    /// guarantees the snapshot we read for the spinner reflow is
    /// the rewound state, not whatever the worker had a millisecond
    /// ago. We deliberately bypass the `dispatch_with_undo` helper
    /// here: the apply-undo path *is* the rewind, pushing another
    /// `Cpu` entry would create a self-referential loop where every
    /// Ctrl+Z grew the redo stack by one entry pointing at itself.
    ///
    /// `register_selection` is `Some(register)` for entries pushed
    /// by `apply_register_and_step` — Ctrl+Z then re-points the
    /// register editor at the register the user was editing
    /// (`select_register` reloads name/value buffers from the
    /// rewound CPU) and pulls focus back into the value field so
    /// the user can keep typing right where they left off. For
    /// every other entry kind the selection is `None` and the
    /// register panel is left alone — memory edits and resets do
    /// not move the register cursor.
    fn replay_cpu_state(
        &mut self,
        state: Cpu8080State,
        register_selection: Option<RegisterName>,
    ) -> Task<Message> {
        // The worker's `ApplyCpuState` handler clears its `running`
        // flag and emits `Stopped`; mirror the cosmetic side here so
        // the play/pause icon does not lag the worker by one tick.
        self.running = false;
        self.dispatch_sync(AppCommand::ApplyCpuState(Box::new(state)));
        // Re-derive the spinner / inline editor against the restored
        // PC, then scroll the memory list so the highlight ends up
        // visible. `follow_pc_into_memory_list` already handles the
        // viewport math, the inline buffer refresh, and the scroll
        // task — exactly the post-step bookkeeping we want here.
        let memory_task = self.follow_pc_into_memory_list();
        // Re-point the register editor at the register the user
        // was on at the moment of the gesture. `select_register`
        // refreshes both the name and value input buffers from the
        // freshly rewound snapshot, so the visible fields match
        // the byte the worker just restored. Without this the
        // rewind would put the byte back into A while the visible
        // name field still showed B (the post-step register), and
        // the user would have no obvious cue that the undo landed.
        if let Some(register) = register_selection {
            self.select_register(register);
            // Pull focus back into the value field so the next
            // keystroke continues editing the rewound register —
            // the same input the user pressed Enter from. The
            // memory-list scroll task is preserved; iced merges
            // the two operations on the same frame.
            self.focused_input = Some(REGISTER_VALUE_INPUT_ID);
            return Task::batch([memory_task, operation::focus(REGISTER_VALUE_INPUT_ID)]);
        }
        memory_task
    }
}
