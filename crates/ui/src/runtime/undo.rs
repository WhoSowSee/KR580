use crate::app::{
    DesktopApp, MEMORY_ADDRESS_INPUT_ID, MEMORY_INLINE_INPUT_ID, MEMORY_VALUE_INPUT_ID, Message,
    OPCODE_SEARCH_INPUT_ID, REGISTER_NAME_INPUT_ID, REGISTER_VALUE_INPUT_ID, UndoEntry,
};
use iced::Task;
use iced::widget::operation;
use k580_app::AppCommand;
use k580_core::{Cpu8080State, RegisterName};

impl DesktopApp {
    pub(crate) fn apply_undo(&mut self) -> Task<Message> {
        let Some(entry) = self.undo_stack.pop_undo() else {
            self.set_status(crate::app::StatusKind::NothingToUndo);
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

    pub(crate) fn apply_redo(&mut self) -> Task<Message> {
        let Some(entry) = self.undo_stack.pop_redo() else {
            self.set_status(crate::app::StatusKind::NothingToRedo);
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

    /// Bypasses `dispatch_with_undo`: the apply-undo path *is* the
    /// rewind, pushing another `Cpu` entry would loop.
    fn replay_cpu_state(
        &mut self,
        state: Cpu8080State,
        register_selection: Option<RegisterName>,
    ) -> Task<Message> {
        self.running = false;
        self.dispatch_sync(AppCommand::ApplyCpuState(Box::new(state)));
        self.recompute_dirty();
        let memory_task = self.follow_pc_into_memory_list();
        if let Some(register) = register_selection {
            self.select_register(register);
            self.focused_input = Some(REGISTER_VALUE_INPUT_ID);
            return Task::batch([memory_task, operation::focus(REGISTER_VALUE_INPUT_ID)]);
        }
        memory_task
    }
}
