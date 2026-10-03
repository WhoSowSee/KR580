use crate::app::{
    DesktopApp, MEMORY_ADDRESS_INPUT_ID, MEMORY_INLINE_INPUT_ID, MEMORY_VALUE_INPUT_ID, Message,
    OPCODE_SEARCH_INPUT_ID, REGISTER_NAME_INPUT_ID, REGISTER_VALUE_INPUT_ID, UndoReplay,
};
use crate::backend::AppCommand;
use iced::Task;
use iced::widget::operation;
use k580_core::{CpuMetadata, RegisterName};

impl DesktopApp {
    pub(crate) fn apply_undo(&mut self) -> Task<Message> {
        let Some(entry) = self.undo_stack.pop_undo() else {
            self.set_status(crate::app::StatusKind::NothingToUndo);
            return Task::none();
        };
        match entry {
            UndoReplay::Text { field, value } => {
                self.restore_text_field(field, value);
                Task::none()
            }
            UndoReplay::Cpu {
                metadata,
                memory,
                register_selection,
                ..
            } => self.replay_cpu_state(metadata, memory, register_selection),
        }
    }

    pub(crate) fn apply_redo(&mut self) -> Task<Message> {
        let Some(entry) = self.undo_stack.pop_redo() else {
            self.set_status(crate::app::StatusKind::NothingToRedo);
            return Task::none();
        };
        match entry {
            UndoReplay::Text { field, value } => {
                self.restore_text_field(field, value);
                Task::none()
            }
            UndoReplay::Cpu {
                metadata,
                memory,
                register_selection,
                ..
            } => self.replay_cpu_state(metadata, memory, register_selection),
        }
    }

    fn restore_text_field(&mut self, field: &'static str, value: String) {
        match field {
            MEMORY_ADDRESS_INPUT_ID => self.memory_address_input = value,
            MEMORY_VALUE_INPUT_ID => {
                self.memory_value_input = value;
                self.memory_inline_value_input = self.memory_value_input.clone();
            }
            MEMORY_INLINE_INPUT_ID => self.memory_inline_value_input = value,
            REGISTER_NAME_INPUT_ID => self.register_name_input = value,
            REGISTER_VALUE_INPUT_ID => self.register_value_input = value,
            OPCODE_SEARCH_INPUT_ID => self.opcode_search_input = value,
            _ => {}
        }
    }

    /// Undo replay must not create another undo entry.
    fn replay_cpu_state(
        &mut self,
        metadata: CpuMetadata,
        memory: crate::backend::MemoryUpdate,
        register_selection: Option<RegisterName>,
    ) -> Task<Message> {
        self.running = false;
        self.dispatch_sync(AppCommand::ApplyCpuDelta { metadata, memory });
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
